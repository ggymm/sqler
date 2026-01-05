# 贡献指南

欢迎为 Sqler 项目做出贡献！本文档提供了参与项目开发所需的规范和指南。

---

## 代码规范

### 1. 导入顺序

所有 Rust 文件的导入语句必须遵循以下顺序：

```
// 1. std 库
use std::sync::Arc;

// 2. 外部 crate 导入（按字母顺序）
use gpui::{prelude::*, *};
use serde::{Deserialize, Serialize};

// 3. Workspace crate 导入
use sqler_core::{
    DatabaseDriver, DriverError,
    DataSource, DataSourceKind,
};

// 4. 当前 crate 导入（按模块分组）
use crate::{
    app::comps::DataTable,
    workspace::CommonWorkspace,
};
```

### 2. 数据源排序标准

所有涉及 `DataSourceKind` 的 match 语句必须遵循标准顺序：

**MySQL → SQLite → Postgres → Oracle → SQLServer → Redis → MongoDB**

示例：

```
match source.kind {
    DataSourceKind::MySQL => { /* ... */ }
    DataSourceKind::SQLite => { /* ... */ }
    DataSourceKind::Postgres => { /* ... */ }
    DataSourceKind::Oracle => { /* ... */ }
    DataSourceKind::SQLServer => { /* ... */ }
    DataSourceKind::Redis => { /* ... */ }
    DataSourceKind::MongoDB => { /* ... */ }
}
```

这种统一的顺序可以提高代码一致性和可维护性。

### 3. 错误处理

- 优先使用 `Result<T, DriverError>` 而非 panic
- 对于可恢复的错误，返回 `Err(...)`
- 对于不可恢复的错误，使用 `expect()` 或 `unwrap()` 并附带清晰的错误信息

示例：

```
// 推荐
fn create_connection(opts: &DataSourceOptions) -> Result<Box<dyn DatabaseSession>, DriverError> {
    match opts {
        DataSourceOptions::MySQL(cfg) => {
            let conn = mysql::Conn::new(&cfg.endpoint())
                .map_err(|e| DriverError::ConnectionFailed(e.to_string()))?;
            Ok(Box::new(MySQLSession { conn }))
        }
        // ...
    }
}

// 避免
fn create_connection(opts: &DataSourceOptions) -> Box<dyn DatabaseSession> {
    let conn = mysql::Conn::new(&opts.endpoint()).unwrap();  // 不推荐
    Box::new(MySQLSession { conn })
}
```

### 4. 命名约定

- **结构体**: 大驼峰命名 (PascalCase)
  - 例：`DataSource`, `MySQLOptions`, `CommonWorkspace`
- **函数/变量**: 蛇形命名 (snake_case)
  - 例：`create_connection`, `active_session`, `table_name`
- **常量**: 全大写蛇形命名 (UPPER_SNAKE_CASE)
  - 例：`PAGE_SIZE`, `ENCRYPTION_KEY`, `BUFFER_SIZE`
- **类型参数**: 单个大写字母或大驼峰
  - 例：`T`, `Config`, `Session`

### 5. 格式化

项目使用 `rustfmt` 进行代码格式化。提交前请运行：

```bash
cargo fmt --all
```

### 6. 文档注释

公共 API 必须包含文档注释：

```
/// 创建数据库连接
///
/// # 参数
///
/// * `opts` - 数据源配置
///
/// # 返回
///
/// 返回数据库会话实例，失败时返回 `DriverError`
///
/// # 示例
///
/// ```
/// let opts = DataSourceOptions::MySQL(MySQLOptions::default());
/// let session = create_connection(&opts)?;
/// ```
pub fn create_connection(opts: &DataSourceOptions) -> Result<Box<dyn DatabaseSession>, DriverError> {
    // ...
}
```

---

## 测试

### 测试数据

- 测试数据脚本位于 `docs/testdata/` 目录
- 每个数据库至少 10 张表，每表 ≥1000 行数据
- 覆盖常见数据类型和关系

### 运行测试

```bash
# 运行所有测试
cargo test --all

# 运行特定 crate 的测试
cargo test -p sqler-core

# 运行特定测试
cargo test test_mysql_connection
```

### 测试覆盖率

项目目标是保持 60% 以上的测试覆盖率。核心模块（sqler-core）应达到 80% 以上。

---

## 提交规范

### 提交消息格式

使用以下格式编写提交消息：

```
<类型>(<范围>): <简短描述>

<详细描述>（可选）

<关联的 Issue 或 PR>（可选）
```

#### 类型

- `feat`: 新功能
- `fix`: Bug 修复
- `refactor`: 重构（既不是新功能也不是 Bug 修复）
- `docs`: 文档更新
- `style`: 代码格式调整（不影响功能）
- `test`: 添加或修改测试
- `chore`: 构建、依赖或配置相关的更改
- `perf`: 性能优化

#### 范围

- `sqler-core`: 核心库
- `sqler-app`: 应用程序
- `sqler-task`: 任务执行器
- `driver`: 数据库驱动
- `cache`: 缓存系统
- `workspace`: 工作区
- `transfer`: 数据传输

#### 示例

```
feat(sqler-task): 实现 SQL 文件执行功能

- 新增 exec 模块，支持流式 SQL 文件执行
- 实现滑动窗口模式处理跨块 SQL 语句
- 支持 100MB 分块读取大文件

Closes #123
```

```
fix(driver): 修复 INF/NAN 字符串被错误解析为数字的问题

在 dump 模块中，INF 和 NAN 字符串被错误识别为数字类型，
导致导出的 SQL 文件中缺少引号。

修复方法：新增 ColumnKind::needs_quotes() 方法，
根据实际列类型判断是否需要引号。
```

---

## 分支管理

- `main`: 主分支，始终保持稳定
- `dev`: 开发分支（如果存在）
- `feat/xxx`: 功能分支
- `fix/xxx`: Bug 修复分支
- `refactor/xxx`: 重构分支

### 工作流程

1. 从 `main` 分支创建新分支
2. 进行开发和测试
3. 提交 Pull Request
4. 代码审查
5. 合并到 `main`

---

## Pull Request 规范

### PR 标题

使用与提交消息相同的格式：

```
feat(sqler-task): 实现 SQL 文件执行功能
```

### PR 描述

包含以下内容：

1. **变更内容**: 简要描述本 PR 的改动
2. **动机**: 为什么需要这个改动
3. **测试**: 如何测试这些改动
4. **相关 Issue**: 关联的 Issue 编号

示例：

```markdown
## 变更内容

实现了 SQL 文件执行功能，支持大文件流式处理。

## 动机

用户需要导入大型 SQL 脚本文件（如数据库备份），
直接在 UI 中执行会阻塞主线程。

## 测试

- 测试 100MB SQL 文件执行
- 测试跨块 SQL 语句处理
- 测试错误处理和日志输出

## 相关 Issue

Closes #123
```

---

## 开发环境设置

### 依赖

- Rust 1.75+
- 支持的数据库（用于测试）：
  - MySQL 5.7+ / MariaDB 10.3+
  - PostgreSQL 12+
  - SQLite 3.35+
  - Redis 6.0+
  - MongoDB 4.4+

### 构建

```bash
# 克隆仓库
git clone https://github.com/yourusername/sqler.git
cd sqler

# 构建所有 crate
cargo build --all

# 构建并运行主应用
cargo run -p sqler-app

# 构建任务执行器
cargo build -p sqler-task
```

### 开发工具

推荐安装以下工具：

```bash
# 代码格式化
rustup component add rustfmt

# 代码检查
rustup component add clippy

# 代码覆盖率
cargo install cargo-tarpaulin

# 依赖分析
cargo install cargo-outdated cargo-bloat
```

---

## 代码审查指南

### 审查重点

1. **正确性**: 代码是否实现了预期功能？
2. **安全性**: 是否存在安全漏洞（SQL 注入、内存泄漏等）？
3. **性能**: 是否存在性能问题（O(n²) 算法、不必要的克隆等）？
4. **可维护性**: 代码是否清晰易懂？是否遵循项目规范？
5. **测试**: 是否包含足够的测试？

### 审查流程

1. 检查提交消息和 PR 描述是否清晰
2. 检查代码是否遵循项目规范
3. 运行测试确保所有测试通过
4. 手动测试关键功能
5. 提出改进建议或批准 PR

---

## 发布流程

1. 更新版本号（Cargo.toml）
2. 更新 CHANGELOG.md
3. 更新 CODEMAP.md
4. 创建 Git 标签
5. 构建发布版本
6. 发布到 GitHub Releases

---

## 联系方式

如有任何问题，请：

- 提交 Issue: https://github.com/ggymm/sqler/issues
- 发起讨论: https://github.com/ggymm/sqler/discussions

感谢您的贡献！
