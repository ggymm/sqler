# Sqler SQL 任务支持方案

## 1. 设计目标

### 1.1 核心原则

1. **简单优先**：只支持 SQL 文件，验证架构可行性
2. **进程隔离**：SQL 执行在独立进程，避免阻塞主界面
3. **安全第一**：数据源凭证不写入任务配置文件
4. **实时反馈**：通过 stdout 流式输出进度和结果
5. **可靠性**：支持检查点和错误恢复

### 1.2 功能范围

**Phase 1（本文档）：SQL 文件任务**
- ✅ 运行 SQL 文件：读取 .sql 文件并执行
- ✅ 转储表为 SQL：导出表数据为 INSERT 语句

**特点**：
- 输入和输出都是 SQL 文件
- 不支持 CSV/JSON 格式
- 专注于 SQL 工作流

---

## 2. 整体架构

### 2.1 架构图

```
┌─────────────────────────────────────────────────────────────┐
│                    sqler-app (主进程)                         │
│                                                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   UI 窗口     │  │  任务管理器   │  │  数据源缓存   │      │
│  │  (右键菜单)   │  │ TaskManager  │  │  AppCache    │      │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘      │
│         │                  │                  │              │
│    用户触发：           创建任务配置     读取数据源信息       │
│    - 右键 .sql 文件        │                  │              │
│    - 右键表 → 转储         │                  │              │
│         │                  │                  │              │
│         └──────────────────┼──────────────────┘              │
│                            │                                 │
│                            ▼                                 │
│                   创建任务目录和配置                          │
│              ~/.sqler/tasks/{task_id}/                       │
│                      config.json                             │
│              (仅包含数据源ID + SQL文件路径)                    │
│                            │                                 │
│                            ▼                                 │
│                   启动子进程并捕获stdout                      │
│              Command::spawn() + pipe                         │
│                            │                                 │
└────────────────────────────┼─────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────┐
│              sqler-task (任务执行器子进程)                     │
│                                                               │
│  命令：sqler-task run-sql --task-dir ~/.sqler/tasks/{id}    │
│  命令：sqler-task dump-table --task-dir ~/.sqler/tasks/{id} │
│                                                               │
│  ┌──────────────────────────────────────────────┐           │
│  │          1. 读取任务配置                      │           │
│  │     config.json → data_source_id + sql_path  │           │
│  └───────────────────┬──────────────────────────┘           │
│                      │                                       │
│                      ▼                                       │
│  ┌──────────────────────────────────────────────┐           │
│  │    2. 从主进程缓存读取完整数据源配置          │           │
│  │    ~/.sqler/cache/sources.db (加密存储)      │           │
│  └───────────────────┬──────────────────────────┘           │
│                      │                                       │
│                      ▼                                       │
│  ┌──────────────────────────────────────────────┐           │
│  │         3. 建立数据库连接                     │           │
│  └───────────────────┬──────────────────────────┘           │
│                      │                                       │
│                      ▼                                       │
│  ┌──────────────────────────────────────────────┐           │
│  │    4a. 运行 SQL：逐行读取并执行               │           │
│  │    4b. 转储表：生成 INSERT 语句写入文件       │           │
│  │    - 流式处理数据                             │           │
│  │    - 定期报告进度                             │           │
│  │    - 保存检查点                               │           │
│  └───────────────────┬──────────────────────────┘           │
│                      │                                       │
│                      ▼                                       │
│         双通道输出：stdout (实时) + 文件 (持久化)             │
│                                                               │
└───────────────────────────────────────────────────────────────┘
```

---

## 3. 安全的配置传递方案

### 3.1 配置文件结构

**运行 SQL 文件配置**
```json
{
  "task_id": "550e8400-e29b-41d4-a716-446655440000",
  "operation": "run_sql",
  "created_at": "2025-12-20T17:30:00Z",

  "data_source_id": "datasource-uuid",

  "sql_file": {
    "path": "/Users/xxx/query.sql",
    "encoding": "utf-8"
  },

  "options": {
    "timeout_seconds": 3600,
    "checkpoint_interval": 100
  }
}
```

**转储表为 SQL 配置**
```json
{
  "task_id": "...",
  "operation": "dump_table",
  "created_at": "...",

  "data_source_id": "datasource-uuid",

  "dump_config": {
    "table": "orders",
    "where": "created_at >= '2025-01-01'",
    "order_by": "id ASC"
  },

  "output_file": {
    "path": "/Users/xxx/Downloads/orders.sql",
    "encoding": "utf-8"
  },

  "options": {
    "batch_size": 1000,
    "checkpoint_interval": 5000
  }
}
```

### 3.2 数据源读取实现

**子进程端（sqler-task）**
```rust
use sqler_core::{AppCache, DataSource};

fn load_data_source(data_source_id: &str) -> Result<DataSource> {
    let cache = AppCache::init()?;
    let sources = cache.read().unwrap();

    sources.sources()
        .iter()
        .find(|s| s.id == data_source_id)
        .cloned()
        .ok_or(Error::DataSourceNotFound)
}

fn main() -> Result<()> {
    let config = read_config(&task_dir)?;
    let data_source = load_data_source(&config.data_source_id)?;
    let mut session = create_connection(&data_source)?;

    match config.operation.as_str() {
        "run_sql" => run_sql_file(&mut session, &config)?,
        "dump_table" => dump_table_to_sql(&mut session, &config)?,
        _ => return Err(Error::UnknownOperation),
    }

    Ok(())
}
```

---

## 4. 任务目录结构

```
~/.sqler/tasks/
├── {task_id}/
│   ├── config.json       # 任务配置（主进程写入，无密码）
│   ├── progress.json     # 实时进度（子进程写入）
│   ├── status.json       # 任务状态（子进程写入）
│   ├── checkpoint.json   # 检查点（子进程定期写入）
│   └── errors.log        # 错误日志（如有）
```

**progress.json**
```json
{
  "status": "running",
  "processed_statements": 42,
  "total_statements": 100,
  "processed_rows": 45230,
  "total_rows": 100000,
  "percentage": 45.23,
  "speed": 1520.5,
  "elapsed_seconds": 29.7,
  "estimated_seconds": 36.1,
  "last_update": "2025-12-20T17:30:29Z"
}
```

**status.json**
```json
{
  "status": "running",
  "pid": 12345,
  "started_at": "2025-12-20T17:30:00Z",
  "updated_at": "2025-12-20T17:30:29Z",
  "completed_at": null
}
```

**checkpoint.json**
```json
{
  "last_statement": 42,
  "last_row": 45230,
  "timestamp": "2025-12-20T17:30:29Z"
}
```

---

## 5. 进度输出协议

### 5.1 JSON Lines 格式

**进度更新**
```json
{"type":"progress","data":{"processed_statements":10,"total_statements":100,"percentage":10.0,"elapsed_seconds":0.65}}
{"type":"progress","data":{"processed_rows":1000,"total_rows":100000,"percentage":1.0,"speed":1520.5}}
```

**状态变更**
```json
{"type":"status","data":{"status":"running","message":"开始执行 SQL 文件"}}
{"type":"status","data":{"status":"completed","message":"执行完成"}}
```

**错误消息**
```json
{"type":"error","data":{"severity":"error","line":15,"code":"SQL_ERROR","message":"Table 'users' doesn't exist"}}
```

**完成消息**
```json
{
  "type":"completed",
  "data":{
    "status":"success",
    "processed_statements":100,
    "affected_rows":1234,
    "elapsed_seconds":65.7
  }
}
```

---

## 6. 功能实现

### 6.1 运行 SQL 文件

#### 6.1.1 功能说明

- 读取 .sql 文件（可能包含多条 SQL 语句）
- 按分号 `;` 分割语句并逐条执行
- 支持 DDL、DML、查询语句
- 显示每条语句的执行结果

#### 6.1.2 实现流程

```rust
fn run_sql_file(
    session: &mut Box<dyn DatabaseSession>,
    config: &TaskConfig,
) -> Result<()> {
    let sql_path = &config.sql_file.path;
    let start_time = Instant::now();

    // 1. 读取 SQL 文件
    let content = fs::read_to_string(sql_path)?;

    // 2. 分割 SQL 语句（按分号）
    let statements = split_sql_statements(&content);
    let total = statements.len();

    report_status("running", &format!("开始执行 {} 条 SQL 语句", total));

    // 3. 逐条执行
    let mut stats = ExecutionStats {
        total_statements: total,
        processed_statements: 0,
        success_count: 0,
        error_count: 0,
    };

    for (idx, sql) in statements.iter().enumerate() {
        let sql = sql.trim();
        if sql.is_empty() || sql.starts_with("--") {
            continue;
        }

        // 判断语句类型
        let is_query = sql.to_uppercase().starts_with("SELECT")
            || sql.to_uppercase().starts_with("SHOW")
            || sql.to_uppercase().starts_with("DESCRIBE");

        match if is_query {
            session.query(QueryReq {
                sql: sql.to_string(),
                params: vec![],
                paging: Paging { page: 1, page_size: 1000 },
            }).map(|_| ())
        } else {
            session.exec(ExecReq {
                sql: sql.to_string(),
                params: vec![],
            }).map(|_| ())
        } {
            Ok(_) => {
                stats.success_count += 1;
            }
            Err(e) => {
                stats.error_count += 1;
                log_error(idx + 1, sql, &e);

                // 如果错误过多，终止执行
                if stats.error_count > 10 {
                    return Err(Error::TooManyErrors);
                }
            }
        }

        stats.processed_statements += 1;

        // 报告进度
        if stats.processed_statements % 10 == 0 {
            report_progress(&Progress {
                processed_statements: stats.processed_statements,
                total_statements: total,
                percentage: (stats.processed_statements as f64 / total as f64) * 100.0,
                elapsed_seconds: start_time.elapsed().as_secs_f64(),
            });
        }

        // 保存检查点
        if stats.processed_statements % config.options.checkpoint_interval == 0 {
            save_checkpoint(&config.task_dir, stats.processed_statements, 0)?;
        }
    }

    // 4. 输出完成消息
    report_completed(json!({
        "status": if stats.error_count == 0 { "success" } else { "partial_success" },
        "processed_statements": stats.processed_statements,
        "success_count": stats.success_count,
        "error_count": stats.error_count,
        "elapsed_seconds": start_time.elapsed().as_secs_f64(),
    }));

    Ok(())
}

// 分割 SQL 语句（简单实现，不处理字符串中的分号）
fn split_sql_statements(content: &str) -> Vec<String> {
    content
        .split(';')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}
```

#### 6.1.3 UI 交互

**触发方式**
- 用户在文件树中右键点击 .sql 文件
- 选择"运行 SQL 文件"菜单项
- 弹出对话框选择目标数据源
- 确认后启动任务

**进度展示**
```
┌──────────────────────────────────────────────────┐
│ 运行 SQL：query.sql                               │
├──────────────────────────────────────────────────┤
│ 数据源：test_db                                   │
│                                                  │
│ 进度：[████████████████░░░░] 68/100 条           │
│                                                  │
│ 状态：运行中 ● 运行时间：00:12                    │
│ 成功：65 条                                       │
│ 失败：3 条                                        │
│                                                  │
│ [取消] [查看日志]                                 │
└──────────────────────────────────────────────────┘
```

### 6.2 转储表为 SQL

#### 6.2.1 功能说明

- 导出表数据为 INSERT 语句
- 支持 WHERE 条件筛选
- 流式处理，批量生成 INSERT
- 显示进度（行数、百分比、速度）

#### 6.2.2 实现流程

```rust
fn dump_table_to_sql(
    session: &mut Box<dyn DatabaseSession>,
    config: &TaskConfig,
) -> Result<()> {
    let dump_config = &config.dump_config;
    let output_path = &config.output_file.path;

    // 1. 构建查询 SQL
    let mut sql = format!("SELECT * FROM {}", dump_config.table);
    if let Some(where_clause) = &dump_config.where_ {
        sql.push_str(&format!(" WHERE {}", where_clause));
    }
    if let Some(order_by) = &dump_config.order_by {
        sql.push_str(&format!(" ORDER BY {}", order_by));
    }

    // 2. 查询总行数
    let count_sql = format!(
        "SELECT COUNT(*) as total FROM {}{}",
        dump_config.table,
        dump_config.where_
            .as_ref()
            .map(|w| format!(" WHERE {}", w))
            .unwrap_or_default()
    );
    let count_resp = session.query(QueryReq {
        sql: count_sql,
        params: vec![],
        paging: Paging { page: 1, page_size: 1 },
    })?;
    let total_rows = count_resp.data[0]
        .get("total")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    // 3. 创建输出文件
    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);

    // 4. 写入文件头
    writeln!(writer, "-- Dump of table: {}", dump_config.table)?;
    writeln!(writer, "-- Generated at: {}", Utc::now().to_rfc3339())?;
    writeln!(writer, "-- Total rows: {}\n", total_rows)?;

    // 5. 分页查询并生成 INSERT 语句
    let batch_size = config.options.batch_size;
    let mut processed = 0u64;
    let start_time = Instant::now();

    let mut page = 1;
    loop {
        let resp = session.query(QueryReq {
            sql: sql.to_string(),
            params: vec![],
            paging: Paging { page, page_size: batch_size },
        })?;

        if resp.data.is_empty() {
            break;
        }

        // 生成批量 INSERT 语句
        if !resp.data.is_empty() {
            // INSERT INTO table (col1, col2, ...) VALUES
            let columns = resp.columns.join(", ");
            write!(writer, "INSERT INTO {} ({}) VALUES\n", dump_config.table, columns)?;

            for (idx, row) in resp.data.iter().enumerate() {
                let values: Vec<String> = resp.columns
                    .iter()
                    .map(|col| format_sql_value(row.get(col)))
                    .collect();

                if idx == resp.data.len() - 1 {
                    writeln!(writer, "  ({});", values.join(", "))?;
                } else {
                    writeln!(writer, "  ({}),", values.join(", "))?;
                }
            }
            writeln!(writer)?;
        }

        processed += resp.data.len() as u64;

        // 报告进度
        report_progress(&Progress {
            processed_rows: processed,
            total_rows,
            percentage: (processed as f64 / total_rows as f64) * 100.0,
            speed: processed as f64 / start_time.elapsed().as_secs_f64(),
            elapsed_seconds: start_time.elapsed().as_secs_f64(),
        });

        // 保存检查点
        if processed % config.options.checkpoint_interval as u64 == 0 {
            save_checkpoint(&config.task_dir, 0, processed)?;
        }

        page += 1;
    }

    writer.flush()?;

    // 6. 输出完成消息
    report_completed(json!({
        "status": "success",
        "processed_rows": processed,
        "output_path": output_path,
        "elapsed_seconds": start_time.elapsed().as_secs_f64(),
    }));

    Ok(())
}

// 格式化 SQL 值（处理 NULL、字符串转义等）
fn format_sql_value(value: Option<&serde_json::Value>) -> String {
    match value {
        None => "NULL".to_string(),
        Some(v) => match v {
            serde_json::Value::Null => "NULL".to_string(),
            serde_json::Value::Bool(b) => if *b { "1" } else { "0" }.to_string(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::String(s) => {
                // 转义单引号
                format!("'{}'", s.replace("'", "''"))
            }
            _ => format!("'{}'", v.to_string().replace("'", "''")),
        },
    }
}
```

#### 6.2.3 UI 交互

**触发方式**
- 右键点击表 → "转储为 SQL"
- 弹出配置对话框：
  - WHERE 条件（可选）
  - 保存路径

**配置对话框**
```
┌──────────────────────────────────────────────────┐
│ 转储表：test_db.orders                            │
├──────────────────────────────────────────────────┤
│                                                  │
│ WHERE 条件（可选）：                              │
│ ┌──────────────────────────────────────────────┐ │
│ │ created_at >= '2025-01-01'                   │ │
│ └──────────────────────────────────────────────┘ │
│                                                  │
│ 保存路径：                                        │
│ ┌──────────────────────────────────────────────┐ │
│ │ ~/Downloads/orders.sql              [浏览...] │ │
│ └──────────────────────────────────────────────┘ │
│                                                  │
│                              [取消] [开始导出]    │
└──────────────────────────────────────────────────┘
```

**进度展示**
```
┌──────────────────────────────────────────────────┐
│ 转储表：test_db.orders                            │
├──────────────────────────────────────────────────┤
│ 状态：运行中 ● 运行时间：00:42                    │
│                                                  │
│ 进度：[████████████████░░░░] 68.4%               │
│       68,430 / 100,000 行                        │
│                                                  │
│ 速度：1,630 行/秒                                 │
│ 预计剩余：19 秒                                   │
│                                                  │
│ 输出：~/Downloads/orders.sql                     │
│                                                  │
│ [取消] [查看日志]                                 │
└──────────────────────────────────────────────────┘
```

---

## 7. 错误处理

### 7.1 错误分类

**致命错误（立即终止）**
- 数据源不存在
- 数据库连接失败
- SQL 文件无法读取
- 输出文件无法创建

**可恢复错误（记录并继续）**
- 单条 SQL 语句执行失败（运行 SQL 文件时）
- 单行数据转换失败（转储时）

### 7.2 错误日志

**errors.log**
```
[2025-12-20 17:30:15] [ERROR] Statement 15: Table 'users' doesn't exist
  SQL: SELECT * FROM users WHERE age > 18

[2025-12-20 17:30:20] [ERROR] Statement 23: Syntax error near 'FORM'
  SQL: SELECT * FORM orders
```

---

## 8. 检查点机制

### 8.1 检查点内容

```json
{
  "last_statement": 42,
  "last_row": 45230,
  "timestamp": "2025-12-20T17:30:29Z"
}
```

### 8.2 断点恢复

**运行 SQL 文件恢复**
- 跳过已执行的语句
- 从 `last_statement + 1` 继续

**转储表恢复**
- 使用 OFFSET 跳过已导出的行
- 从 `last_row` 继续查询

---

## 9. 性能优化

### 9.1 批量处理

- 转储时每 1000 行生成一条 INSERT 语句
- 减少文件写入次数

### 9.2 流式 I/O

- 使用 `BufWriter` 缓冲写入
- 内存占用恒定（~10MB）

### 9.3 生成的 SQL 格式

```sql
-- 批量 INSERT（每批 1000 行）
INSERT INTO orders (id, user_id, amount, created_at) VALUES
  (1, 100, 99.99, '2025-01-01 10:00:00'),
  (2, 101, 149.99, '2025-01-01 11:00:00'),
  ...
  (1000, 1099, 199.99, '2025-01-01 20:00:00');
```

---

## 10. 实现路线图

### Phase 1：基础架构（Week 1）
- [ ] 创建 sqler-task 项目结构
- [ ] 实现子进程启动和 stdout 捕获
- [ ] 实现数据源 ID 查找逻辑
- [ ] 实现进度输出协议
- [ ] 单元测试

### Phase 2：运行 SQL 文件（Week 2）
- [ ] 实现 `sqler-task run-sql` 子命令
- [ ] 实现 SQL 文件解析（分割语句）
- [ ] 实现逐条执行和错误处理
- [ ] 实现检查点和恢复
- [ ] UI 集成：右键菜单、进度展示

### Phase 3：转储表为 SQL（Week 3）
- [ ] 实现 `sqler-task dump-table` 子命令
- [ ] 实现 INSERT 语句生成
- [ ] 实现值转义和格式化
- [ ] 实现检查点和恢复
- [ ] UI 集成：右键菜单、配置对话框、进度展示

### Phase 4：测试和优化（Week 4）
- [ ] 集成测试：端到端流程
- [ ] 压力测试：大表导出（100 万行）
- [ ] 性能优化
- [ ] 错误处理完善
- [ ] 文档

---

## 11. UI 设计

### 11.1 右键菜单

**SQL 文件右键菜单**
```
┌─────────────────┐
│ 运行 SQL 文件... │  ← 新增
│ ─────────────── │
│ 打开            │
│ 重命名          │
│ 删除            │
└─────────────────┘
```

**表节点右键菜单**
```
┌─────────────────┐
│ 查看数据        │
│ 转储为 SQL...   │  ← 新增
│ ─────────────── │
│ 清空表          │
│ 删除表          │
└─────────────────┘
```

### 11.2 任务列表

```
┌─────────────────────────────────────────────────────┐
│ SQL 任务                                 [刷新] [清理]│
├─────────────────────────────────────────────────────┤
│ ● 转储表：orders → orders.sql                       │
│   68.4% | 1,630 行/秒 | 3 分钟前                    │
│   [查看详情] [取消]                                  │
├─────────────────────────────────────────────────────┤
│ ✓ 运行 SQL：query.sql                               │
│   68/100 条 | 5 分钟前                              │
│   [查看日志] [重新运行]                              │
├─────────────────────────────────────────────────────┤
│ ✗ 转储表：products → products.sql                   │
│   失败：连接超时 | 10 分钟前                         │
│   [查看错误] [重试]                                  │
└─────────────────────────────────────────────────────┘
```

---

## 12. 代码示例

### 12.1 主进程启动任务

```rust
impl TaskManager {
    pub fn create_run_sql_task(
        &mut self,
        data_source_id: String,
        sql_file_path: PathBuf,
    ) -> Result<TaskHandle> {
        let task_id = Uuid::new_v4().to_string();
        let task_dir = self.tasks_root.join(&task_id);
        fs::create_dir_all(&task_dir)?;

        let config = TaskConfig {
            task_id: task_id.clone(),
            operation: "run_sql".into(),
            created_at: Utc::now(),
            data_source_id,
            sql_file: SqlFile {
                path: sql_file_path.display().to_string(),
                encoding: "utf-8".into(),
            },
            options: TaskOptions {
                timeout_seconds: 3600,
                checkpoint_interval: 100,
            },
        };

        fs::write(
            task_dir.join("config.json"),
            serde_json::to_string_pretty(&config)?,
        )?;

        let mut child = Command::new("sqler-task")
            .arg("run-sql")
            .arg("--task-dir")
            .arg(&task_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.take().unwrap();
        let reader = BufReader::new(stdout);

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            for line in reader.lines().flatten() {
                if let Ok(msg) = serde_json::from_str::<ProgressMessage>(&line) {
                    tx.send(msg).ok();
                }
            }
        });

        Ok(TaskHandle {
            task_id,
            task_dir,
            child,
            progress_rx: rx,
            status: TaskStatus::Running,
            created_at: Utc::now(),
        })
    }

    pub fn create_dump_table_task(
        &mut self,
        data_source_id: String,
        table: String,
        output_path: PathBuf,
        where_clause: Option<String>,
    ) -> Result<TaskHandle> {
        let task_id = Uuid::new_v4().to_string();
        let task_dir = self.tasks_root.join(&task_id);
        fs::create_dir_all(&task_dir)?;

        let config = TaskConfig {
            task_id: task_id.clone(),
            operation: "dump_table".into(),
            created_at: Utc::now(),
            data_source_id,
            dump_config: Some(DumpConfig {
                table,
                where_: where_clause,
                order_by: Some("id ASC".into()),
            }),
            output_file: OutputFile {
                path: output_path.display().to_string(),
                encoding: "utf-8".into(),
            },
            options: TaskOptions {
                batch_size: 1000,
                checkpoint_interval: 5000,
            },
        };

        fs::write(
            task_dir.join("config.json"),
            serde_json::to_string_pretty(&config)?,
        )?;

        let mut child = Command::new("sqler-task")
            .arg("dump-table")
            .arg("--task-dir")
            .arg(&task_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdout = child.stdout.take().unwrap();
        let reader = BufReader::new(stdout);

        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            for line in reader.lines().flatten() {
                if let Ok(msg) = serde_json::from_str::<ProgressMessage>(&line) {
                    tx.send(msg).ok();
                }
            }
        });

        Ok(TaskHandle {
            task_id,
            task_dir,
            child,
            progress_rx: rx,
            status: TaskStatus::Running,
            created_at: Utc::now(),
        })
    }
}
```

---

## 13. 安全性说明

### 13.1 密码不落盘

- ✅ 任务配置文件不含密码
- ✅ 仅包含数据源 ID 引用
- ✅ 子进程从加密缓存读取密码

### 13.2 进程隔离

- ✅ SQL 执行在独立进程
- ✅ 崩溃不影响主界面
- ✅ 通过 stdout 单向传输数据

### 13.3 SQL 注入防护

- ✅ 表名通过白名单验证
- ✅ WHERE 条件由用户输入（用户自行负责）
- ✅ 生成的 SQL 值正确转义

---

## 14. 性能目标

**基准环境**
- CPU：4 核心
- 内存：8 GB
- 磁盘：SSD
- 数据库：本地 MySQL 8.0

**性能指标**
- 转储表（SQL）：>= 8,000 行/秒
- 内存占用：< 50 MB（恒定）
- 启动延迟：< 500ms

---

**文档版本**: v2.0
**最后更新**: 2025-12-20
**状态**: 设计完成 - 已简化为仅支持 SQL 文件
