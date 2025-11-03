# Sqler 代码地图

## 项目概览

### 基本信息

- **名称**: `sqler`
- **目标**: 桌面化多标签数据库管理器，可浏览数据源、维护连接并扩展查询能力

### 技术栈

- **核心框架**: Rust + gpui/gpui-component
- **数据库驱动**: 多数据库驱动 crate（mysql, postgres, rusqlite, mongodb, redis 等）

---

## 代码结构

### 1. 入口模块 (`src/main.rs`)

**职责**: 程序入口，应用初始化

**核心功能**:

1. 注册本地资源加载器 `FsAssets`
2. 设置基础主题字号
3. 在 Application 内打开主窗口
4. 将根视图挂载为 `SqlerApp`
5. `init_runtime()` 预留运行时初始化挂钩（当前为空实现）

---

### 2. 应用层 (`src/app/`)

**职责**: 核心 UI 逻辑和状态管理

#### 2.1 应用状态 (`mod.rs`)

**核心结构**: `SqlerApp`

**维护状态**:

1. `TabState` 列表: 所有打开的标签页
2. 活动标签 ID（String）: 首页为 `"home"`，工作区为数据源 UUID
3. 窗口句柄
4. `CacheApp` 缓存管理器

**标签 ID 设计**:

- **首页**: 使用固定字符串 `"home"` 作为 ID
- **工作区**: 直接使用数据源的 UUID 作为 ID
- **优势**: 消除了 TabId 包装类型和计数器，简化了查找逻辑

**核心功能**:

1. 标签页增删管理
2. 主题切换
3. 打开新建数据源窗口
4. `TabState`/`TabView` 差异化渲染（首页/工作区）

**数据源**:

- 当前使用 `seed_sources()` 生成默认 MySQL 示例
- TODO: 接入缓存系统的真实数据源

---

#### 2.2 公共组件 (`comps/`)

##### 组件工具 (`mod.rs`)

- 图标构造辅助函数
- 元素 ID 拼接工具
- 导出 `DataTable` 组件

##### 数据表格组件 (`table.rs`)

**核心结构**: `DataTable`

**实现接口**: `gpui-component::TableDelegate`

**字段设计**:

1. `col_defs: Vec<Column>`: 列定义对象
2. `cols: Vec<SharedString>`: 列名
3. `rows: Vec<Vec<SharedString>>`: 行数据

**核心方法**:

1. `new(cols, rows)`: 创建表格，动态生成列定义
2. `update_data(cols, rows)`: 更新数据，重新生成列定义
3. `build()`: 构建 Table Entity，配置表格属性

**表格配置**:

- 尺寸: Small
- 边框: 启用
- 列拖拽: 启用
- 列可调整大小: 启用
- 列/行选择: 启用
- 循环选择: 启用
- 滚动条: 显示

**动态列支持**:

- 通过 `update_data()` 更新数据
- 调用 `table.refresh(cx)` 重新准备列/行布局
- 支持从 0 列动态变更到任意列数

---

#### 2.3 数据源创建 (`create/`)

##### 创建窗口 (`mod.rs`)

**核心结构**: `CreateWindow`

**状态管理**: `CreateState`

**功能流程**:

1. 数据源类型选择页
2. 具体表单页（根据类型切换）
3. 底部操作按钮（测试连接/保存）

**当前状态**: 测试连接为占位事件，未实际执行

##### 表单实现

**支持类型**:

- `mysql.rs`: MySQL 表单
- `postgres.rs`: PostgreSQL 表单
- `sqlite.rs`: SQLite 表单
- `sqlserver.rs`: SQL Server 表单
- `oracle.rs`: Oracle 表单
- `redis.rs`: Redis 表单
- `mongodb.rs`: MongoDB 表单

**表单组成**:

- 主要由 `InputState` 组成
- 提供默认值/placeholder
- 支持连接参数输入

---

#### 2.4 工作区 (`workspace/`)

##### 工作区路由 (`mod.rs`)

**职责**: 根据数据源类型构造对应工作区

**路由策略**:

- MySQL: 使用 `MySQLWorkspace` 真实工作区
- 其他: 使用 `PlaceholderWorkspace` 占位视图

**WorkspaceState 设计**:

- **移除冗余**: 不再保存数据源 ID（直接使用 TabState.id）
- **简化结构**: 只保存 view（Entity）字段
- **类型区分**: MySQL 和 Placeholder 两种变体

**首页渲染**:

- 网格卡片展示所有数据源
- 双击卡片打开对应标签页

##### MySQL 工作区 (`mysql.rs`)

**核心结构**: `MySQLWorkspace`

**布局**: 左右分隔面板（表列表 + 内容区）

---

###### 连接管理

**策略**: 延迟建立 + 连接复用

**实现细节**:

1. `session: Option<Box<dyn DatabaseSession>>`: 连接实例
2. `active_session()`: 懒加载获取或创建连接
3. 后台任务通过 `session.take()` 移动连接到线程
4. 查询完成后通过 `session = Some(...)` 归还连接
5. 失败时设置 `session = None`，下次重新建立

**优点**:

- 避免重复创建连接
- 支持跨线程使用（MySQL 连接实现 `Send`）
- 失败自动重试

---

###### 标签页管理

**类型定义**:

- `TabContent::Overview`: 概览标签（不可关闭）
- `TabContent::Table`: 表格标签（可关闭）

**创建流程** (`create_table_tab`):

1. 检查标签页是否已存在
2. 创建空 `TableContent`（Table 用空数据初始化）
3. 设置为活动标签
4. 调用 `reload_table_tab` 加载实际数据

**设计优势**:

- 避免代码重复（创建和刷新共用加载逻辑）
- 用户立即看到标签页（无需等待数据加载）

---

###### 数据加载 (`reload_table_tab`)

**执行流程**:

**① 准备阶段**（主线程）:

1. 从 `table_content` 获取当前页码、页大小、总行数
2. 构建 `QueryConditions`（包含分页参数）
3. 通过 `active_session()` 获取连接
4. 使用 `session.take()` 移动连接到闭包

**② 后台查询**（后台线程）:

1. 查询列名: `SHOW COLUMNS FROM table`
2. 查询数据: `SELECT * FROM table LIMIT x OFFSET y`
3. 查询总数: `SELECT COUNT(*) FROM table`
4. 转换数据为 `Vec<Vec<SharedString>>`

**③ UI 更新**（主线程）:

1. 归还连接: `this.session = Some(session)`
2. 解构 `TablePage` 为独立变量（避免所有权冲突）
3. 更新 `data_tab` 的页码、总数、列名
4. 调用 `content.update()` 更新表格:
    - `delegate_mut().update_data(columns, rows)`: 更新数据
    - `refresh(cx)`: 重新准备列/行布局（关键！支持动态列）
    - `cx.notify()`: 触发重新渲染

**关键点**:

- `refresh(cx)` 必须调用，否则列结构不会更新
- 先解构 `TablePage` 避免闭包捕获冲突

---

###### 表格功能

**SQL 生成**:

- 调用 `driver::create_builder(DataSourceKind::MySQL)` 获取构建器
- 使用 `build_select_query()` 生成 SELECT 语句
- 使用 `build_count_query()` 生成 COUNT 语句

**已实现功能**:

1. 分页导航（上一页/下一页）
2. 显示当前页范围和总数
3. 筛选/排序规则 UI（添加/删除规则）
4. 列筛选按钮
5. 数据筛选开关

**TODO**:

- 筛选条件已收集但尚未写入 `QueryConditions`
- 排序规则已收集但尚未写入 `QueryConditions`
- 需要从 Dropdown 读取选中值并构建实际筛选/排序条件

---

##### 占位工作区 (`placeholder.rs`)

**用途**: 非 MySQL 数据源的占位展示

**提示**: 功能尚未实现

---

### 3. 缓存系统 (`src/cache/`)

**核心结构**: `CacheApp` (`mod.rs`)

**职责**: 本地加密存储数据源配置

#### 存储机制

**加密算法**: AES-256-GCM

**存储路径**: `~/.sqler/sources.enc`

**初始化流程**:

1. 确保目录存在
2. 尝试解密加载现有配置
3. 解密失败则使用空列表

#### 核心 API

**读取**:

- `sources()`: 获取数据源列表引用
- `sources_mut()`: 获取可变引用

**写入**:

- `sources_update()`: 加密并写入文件

**错误处理**:

- `CacheError` 枚举描述错误类型

#### 当前状态

**未被使用**: `SqlerApp` 尚未接入缓存系统

- UI 展示的仍是 `seed_sources()` 的演示数据
- 新建数据源窗口的保存逻辑未实现

---

### 4. 数据库驱动 (`src/driver/`)

**职责**: 统一数据库操作接口和 SQL 查询构建

#### 核心接口 (`mod.rs`)

**Trait 定义**:

- `DatabaseDriver`: 驱动工厂（创建连接、检查连接）
- `DatabaseSession`: 会话操作（查询、插入、更新、删除）
- `QueryBuilder`: SQL 查询构建器接口

**请求/响应类型**:

- `QueryReq`: 查询请求（SQL/Command/Document）；SQL 变体现包含 `params: Vec<Value>`，用于绑定占位符参数
- `QueryResp`: 查询响应（Rows/Value/Documents）
- `InsertReq`: 插入请求
- `UpdateReq`: 更新请求
- `DeleteReq`: 删除请求
- `WriteResp`: 写入响应

**SQL 构建器类型**:

- `Operator`: 查询操作符（等于、大于、小于、Like 等）
- `ConditionValue`: 条件值类型
- `FilterCondition`: 筛选条件
- `SortOrder`: 排序规则（升序/降序）
- `QueryConditions`: 查询条件集合

**路由函数**:

- `check_connection()`: 按类型分发连接检查
- `create_connection()`: 按类型分发连接创建
- `create_builder(kind: DataSourceKind)`: 根据数据库类型创建对应的查询构建器

---

#### MySQL 驱动 (`mysql.rs`)

**实现**: 基于 `mysql` crate

**核心功能**:

1. 连接管理: 支持字符集设置
2. 查询操作: 执行 SQL 并返回结果集（支持位置参数绑定）
3. 写操作: INSERT/UPDATE/DELETE
4. 类型转换: `mysql::Value` → `serde_json::Value`
5. 参数转换: `serde_json::Value` → `mysql::Value`

**SQL 构建器** (`MySQLBuilder`):

- 标识符转义: 使用反引号 `` ` ``
- 占位符格式: `?`
- WHERE/ORDER BY/LIMIT 拼接

**特性**:

- 支持连接池配置
- 自动处理字符集

---

#### PostgreSQL 驱动 (`postgres.rs`)

**实现**: 基于 `postgres` crate

**核心功能**:

1. 连接管理: 禁止 SSL 模式
2. 查询操作: 执行 SQL 并返回结果集（支持位置参数绑定）
3. 写操作: INSERT/UPDATE/DELETE
4. 类型转换: PostgreSQL 原生类型 → JSON
5. 参数转换: `serde_json::Value` → `Box<dyn ToSql + Sync>`

**SQL 构建器** (`PostgreSQLBuilder`):

- 标识符转义: 使用双引号 `"`
- 占位符格式: `$1, $2, $3...`
- WHERE/ORDER BY/LIMIT 拼接

**特性**:

- 支持常见类型映射
- 同步客户端

---

#### SQLite 驱动 (`sqlite.rs`)

**实现**: 基于 `rusqlite` crate

**核心功能**:

1. 连接管理: 支持只读/创建模式
2. 查询操作: 执行 SQL 并返回结果集（支持位置参数绑定）
3. 写操作: INSERT/UPDATE/DELETE
4. 类型转换: SQLite 类型 → JSON
5. 参数转换: `serde_json::Value` → SQLite 值

**SQL 构建器** (`SQLiteBuilder`):

- 标识符转义: 使用双引号 `"`
- 占位符格式: `?`
- WHERE/ORDER BY/LIMIT 拼接

**特性**:

- 支持文件路径连接
- 内存数据库支持

---

#### MongoDB 驱动 (`mongodb.rs`)

**实现**: 基于 `mongodb` crate

**核心功能**:

1. 连接管理: 支持 connection string 或 host 列表
2. 文档型 CRUD: find/insert/update/delete
3. 响应转换: BSON → JSON

**特性**:

- 同步客户端封装
- 支持默认数据库配置

---

#### Redis 驱动 (`redis.rs`)

**实现**: 基于 `redis` crate

**核心功能**:

1. 连接管理: 支持 URL 连接字符串
2. 命令执行: 支持任意 Redis 命令
3. 响应转换: Redis 类型 → JSON
4. 写操作: 根据返回值估算影响行数

**特性**:

- 命令式驱动
- 支持密码认证

---

#### SQL Server 驱动 (`sqlserver.rs`)

**状态**: 占位实现

**返回**: 所有操作返回"未实现"错误

---

#### Oracle 驱动 (`oracle.rs`)

**状态**: 暂无实现内容

---

### 5. 数据源配置 (`src/option/`)

**职责**: 定义数据源类型和连接参数

#### 核心定义 (`mod.rs`)

**数据源类型**:

- `DataSourceKind`: 枚举（MySQL/PostgreSQL/SQLite/...）
- `DataSource`: 数据源实体（ID、名称、描述、类型、选项）
- `DataSourceOptions`: 连接参数联合类型

**工具方法**:

- `DataSource::tables()`: 获取表列表
- `DataSource::display()`: 获取显示名称
- `ConnectionOptions` trait: 连接信息脱敏输出

---

#### 子模块

**各数据库配置结构体**:

- `mysql.rs`: `MySQLOptions`
- `postgres.rs`: `PostgreSQLOptions`
- `sqlite.rs`: `SQLiteOptions`
- `sqlserver.rs`: `SQLServerOptions`
- `oracle.rs`: `OracleOptions`
- `redis.rs`: `RedisOptions`
- `mongodb.rs`: `MongoDBOptions`

**配置字段**:

- 基本连接参数（host, port, user, password）
- 数据库特定选项（TLS, schema, read_only 等）
- `display_endpoint()`: 脱敏输出（隐藏密码）

---

### 6. 静态资源 (`assets/`)

**内容**: 数据库图标等静态文件

**用途**: UI 引用

**加载**: 通过 `FsAssets` 注册到 gpui

---

### 7. 项目配置 (`Cargo.toml`)

**核心依赖**:

- **UI 框架**: gpui, gpui-component
- **加密**: aes-gcm
- **序列化**: serde, serde_json
- **数据库驱动**: mysql, postgres, rusqlite, mongodb, redis
- **工具**: dirs, uuid, thiserror

### 8. 测试数据脚本 (`scripts/test/`)

**职责**: 为常见数据库批量生成演示数据，统一 10 张电商业务表模型，每表≥1000 行

- `mysql_init.sql`: MySQL 版本，使用递归 CTE 批量插入 10 表（含触发器、自引用分类）
- `oracle_init.sql`: Oracle 版本，PL/SQL 循环生成 1000 行，覆盖枚举校验与更新时间触发器
- `sqlite_init.sql`: SQLite 版本，递归 CTE 驱动数据插入，保持外键与约束一致
- `sqlserver_init.sql`: SQL Server 版本，CTE + 系统表构造序列，批量填充 10 张表
- `postgres_init.sql`: PostgreSQL 版本，使用多枚举类型 + `generate_series` 插入数据
- `redis_init.redis`: Redis 脚本，Lua 批量写入 10 类 key（哈希结构模拟关系型行）
- `mongodb_init.js`: MongoDB 脚本，批量插入 10 个集合并建立关键索引（邮件、SKU、运单号）

---

## 功能现状

### 已实现功能

#### 主窗口

1. 顶部标签栏（支持多标签切换）
2. 主题切换按钮
3. 新建数据源浮动窗口

#### 首页

1. 网格卡片展示数据源
2. 双击打开工作区标签
3. 当前仅内置演示 MySQL 数据源

#### MySQL 工作区

1. ✅ 左侧表列表导航
2. ✅ 动态标签页管理
3. ✅ 分页查询（上一页/下一页）
4. ✅ 数据表格展示（支持动态列）
5. ✅ 筛选/排序 UI（添加/删除规则）
6. ✅ 连接复用机制

#### 数据库驱动

1. ✅ MySQL/PostgreSQL/SQLite 驱动实现
2. ✅ SQL 构建器（MySQL/PostgreSQL/SQLite）
3. ✅ SELECT/COUNT 语句生成
4. ✅ 参数化查询

#### 新建数据源窗口

1. ✅ 数据库类型选择
2. ✅ 表单字段填充
3. ❌ 保存到缓存（逻辑未实现）
4. ❌ 测试连接（占位事件）

#### 缓存系统

1. ✅ AES-256-GCM 加密
2. ✅ 本地文件读写
3. ❌ 未被 UI 调用（待接入）

---

### 待完成功能

#### 高优先级

1. **筛选/排序功能**
    - 从 Dropdown 读取选中值
    - 构建实际的 `QueryConditions`
    - 将条件注入 SQL 查询

2. **缓存系统接入**
    - `SqlerApp` 从缓存加载数据源
    - 新建数据源保存到缓存
    - 测试连接功能实现

3. **其他数据源工作区**
    - PostgreSQL 工作区实现
    - SQLite 工作区实现
    - Redis/MongoDB 工作区实现

#### 中优先级

1. **查询编辑器**
    - SQL 编辑器标签页
    - 语法高亮
    - 执行查询并展示结果

2. **数据编辑**
    - 单元格编辑
    - 行增删
    - 保存变更到数据库

3. **导入/导出**
    - CSV 导入
    - JSON 导出
    - SQL 导出

#### 低优先级

1. **高级功能**
    - 查询历史
    - 收藏查询
    - 数据库结构可视化
    - 性能监控

---

## 项目状态

**当前阶段**: 核心功能开发中

**可用功能**:

- ✅ MySQL 数据源浏览和查询
- ✅ 多标签管理
- ✅ 分页导航

**待完善**:

- ⚠️ 筛选/排序逻辑
- ⚠️ 缓存系统接入
- ⚠️ 其他数据源支持
