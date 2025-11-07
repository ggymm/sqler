# Sqler 代码地图

## 项目概览

### 基本信息

- **名称**: `sqler`
- **目标**: 桌面化多标签数据库管理器，支持多类型数据库的连接、浏览、查询和管理
- **版本**: v0.1.0

### 技术栈

- **核心框架**: Rust + GPUI (图形界面框架) + gpui-component
- **数据库驱动**: mysql, postgres, rusqlite, mongodb, redis 等
- **加密**: AES-256-GCM (数据源配置加密存储)
- **序列化**: serde, serde_json

---

## 代码结构

### 1. 入口模块 (`src/main.rs`, 84 行)

**职责**: 程序入口，应用初始化和窗口创建

**核心功能**:

1. 注册本地资源加载器 `FsAssets` (从 `assets/` 目录加载图标等资源)
2. 初始化 GPUI 框架和组件库
3. 配置全局主题 (字号 14pt，滚动条悬停显示)
4. 创建主窗口 (1280x800，居中显示)
5. 挂载 `SqlerApp` 作为根视图
6. 配置窗口关闭行为 (所有窗口关闭时退出应用)
7. 预留 `init_runtime()` 运行时初始化挂钩（当前为空实现）

---

### 2. 应用层 (`src/app/`)

**职责**: 核心 UI 逻辑、状态管理和用户交互

#### 2.1 应用状态 (`mod.rs`, ~390 行)

**核心结构**: `SqlerApp`

**状态字段**:

```rust
pub struct SqlerApp {
    pub tabs: Vec<TabState>,                           // 所有打开的标签页
    pub active_tab: String,                            // 当前活动标签 ID
    pub cache: CacheApp,                               // 缓存管理器(唯一数据源)
    pub create_window: Option<WindowHandle<Root>>,    // 新建数据源窗口句柄
    pub transfer_window: Option<WindowHandle<Root>>,  // 数据传输窗口句柄
}
```

**TabState 设计**:

```rust
pub struct TabState {
    pub id: String,           // 标签 ID：首页="home"，工作区=数据源UUID
    pub view: TabView,        // 视图内容
    pub title: SharedString,  // 标签标题
    pub closable: bool,       // 是否可关闭
}

pub enum TabView {
    Home,                     // 首页视图
    Workspace(WorkspaceState), // 工作区视图
}
```

**标签 ID 设计优势**:

- 首页使用固定字符串 `"home"`
- 工作区直接使用数据源的 UUID
- 消除了 TabId 包装类型和计数器，简化查找逻辑

**核心方法**:

1. `new()`: 初始化应用，加载主题和缓存
2. `close_tab()`: 关闭标签，自动切换到前一个标签
3. `active_tab()`: 切换活动标签
4. `create_tab()`: 创建工作区标签（避免重复，使用 `cache.sources()` 查找数据源）
5. `toggle_theme()`: 切换亮色/暗色主题
6. `display_create_window()`: 打开新建数据源窗口
7. `display_transfer_window()`: 打开数据传输窗口
8. `close_create_window()`: 关闭新建数据源窗口
9. `close_transfer_window()`: 关闭数据传输窗口

**数据源管理**:

- ✅ 通过 `cache.sources()` 获取数据源列表（零成本借用）
- ✅ 首页渲染使用 `app.cache.sources()` (src/app/workspace/mod.rs:93)
- ✅ 创建标签使用 `app.cache.sources()` (src/app/mod.rs:147)
- ✅ 单一数据源原则，无数据重复

**UI 渲染**:

- 顶部标签栏 (支持切换和关闭)
- 主题切换按钮
- 新建数据源按钮
- 内容区域 (动态渲染首页或工作区)

---

#### 2.2 公共组件 (`comps/`)

##### 组件工具 (`mod.rs`, ~100 行)

**提供功能**:

1. **元素 ID 拼接工具**:
   ```
   pub fn comp_id<I>(parts: I) -> ElementId
   ```
    - 示例: `comp_id(["tab", "mysql"])` → `"tab-mysql"`

2. **图标加载函数**:
    - `icon_close()`: 关闭图标
    - `icon_export()`: 导出图标
    - `icon_import()`: 导入图标
    - `icon_relead()`: 刷新图标
    - `icon_search()`: 搜索图标
    - `icon_sheet()`: 表格图标
    - `icon_transfer()`: 传输图标
    - `icon_trash()`: 删除图标

3. **布局扩展 Trait**:
    - `DivExt`: 为 `Div` 添加 `col_full()` 和 `row_full()` 快捷方法

---

##### 数据表格组件 (`table.rs`, ~150 行)

**核心结构**: `DataTable`

```
pub struct DataTable {
    col_defs: Vec<Column>,        // 列定义对象
    cols: Vec<SharedString>,      // 列名
    rows: Vec<Vec<SharedString>>, // 行数据
    loading: bool,                // 加载状态
}
```

**实现接口**: `gpui_component::TableDelegate`

**核心方法**:

1. `new(cols, rows)`: 创建表格，自动生成列定义
2. `update_data(cols, rows)`: 更新数据，支持动态列变更
3. `build(window, cx)`: 构建 Table Entity，配置表格属性

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

##### 创建窗口 (`mod.rs`, ~375 行)

**核心结构**: `CreateWindow`

```rust
pub struct CreateWindow {
    kind: Option<DataSourceKind>,  // 当前选中的数据库类型
    parent: WeakEntity<SqlerApp>,
    status: Option<CreateStatus>,  // 连接测试状态

    // 各类型的创建表单实体
    mysql: Entity<MySQLCreate>,
    postgres: Entity<PostgresCreate>,
    sqlite: Entity<SQLiteCreate>,
    oracle: Entity<OracleCreate>,
    sqlserver: Entity<SQLServerCreate>,
    redis: Entity<RedisCreate>,
    mongodb: Entity<MongoDBCreate>,
}

pub enum CreateStatus {
    Testing,
    Success(String),
    Error(String),
}
```

**核心方法**:

1. `new()`: 初始化窗口，注册 `on_release` 回调关闭父窗口引用
2. `cancel()`: 取消创建，关闭窗口
3. `check_conn()`: 异步测试连接，调用 `check_connection(&options)`
4. `create_conn()`: 保存数据源到缓存

**功能流程**:

1. **类型选择页**: 展示所有支持的数据库类型（带图标和描述）
2. **表单页**: 根据选中类型动态切换对应的创建表单
3. **底部操作**:
    - 测试连接按钮：异步调用 `check_connection()`
    - 上一步按钮：返回类型选择页
    - 取消按钮：关闭窗口
    - 保存按钮：保存到 `cache.sources_mut()` 并加密写入

**保存流程** (src/app/create/mod.rs:178-202):

1. 构建 `DataSource::new(name, kind, options)`
2. `app.cache.sources_mut().push(source)`
3. `app.cache.sources_update()` 加密写入 `sources.db`
4. 成功后关闭窗口，失败显示错误

**当前状态**:

- ✅ UI 完整实现
- ✅ 表单字段收集
- ✅ 测试连接逻辑（后台线程调用 `check_connection()`）
- ✅ 保存到缓存逻辑（已实现并接入）
- ❌ Oracle / SQL Server 驱动未实现（保存时返回错误提示）

---

##### 表单实现

**支持的数据库类型** (每个独立模块):

| 模块             | 数据库        | 行数   | 状态   |
|----------------|------------|------|------|
| `mysql.rs`     | MySQL      | ~100 | ✅ 完整 |
| `postgres.rs`  | PostgreSQL | ~100 | ✅ 完整 |
| `sqlite.rs`    | SQLite     | ~80  | ✅ 完整 |
| `oracle.rs`    | Oracle     | ~120 | ✅ 完整 |
| `sqlserver.rs` | SQL Server | ~120 | ✅ 完整 |
| `redis.rs`     | Redis      | ~80  | ✅ 完整 |
| `mongodb.rs`   | MongoDB    | ~150 | ✅ 完整 |

**表单特点**:

- 基于 `InputState` 组件构建
- 提供默认值和占位符
- 支持连接参数输入（主机、端口、用户名、密码等）
- 提供 `options(cx)` 方法构建对应的 Options 结构

---

#### 2.4 工作区 (`workspace/`)

##### 工作区路由 (`mod.rs`, ~136 行)

**职责**: 根据数据源类型构造对应工作区视图

**WorkspaceState 枚举**:

```rust
pub enum WorkspaceState {
    Common { view: Entity<CommonWorkspace> },     // 关系型数据库
    Redis { view: Entity<RedisWorkspace> },       // Redis
    MongoDB { view: Entity<MongoDBWorkspace> },   // MongoDB
}
```

**路由策略** (标准顺序):

```
match meta.kind {
    DataSourceKind::MySQL
    | DataSourceKind::SQLite
    | DataSourceKind::Postgres
    | DataSourceKind::Oracle
    | DataSourceKind::SQLServer => {
        WorkspaceState::Common { view }
    }
    DataSourceKind::Redis => {
        WorkspaceState::Redis { view }
    }
    DataSourceKind::MongoDB => {
        WorkspaceState::MongoDB { view }
    }
}
```

**数据源排序标准**:

```
MySQL → SQLite → Postgres → Oracle → SQLServer → Redis → MongoDB
```

**首页渲染** (`render_home`):

- 4 列网格布局
- 卡片展示数据源（名称、图标、连接地址）
- 双击卡片打开对应工作区标签

**工具函数**:

- `parse_count(value: &str) -> usize`: 解析数字字符串

---

##### CommonWorkspace - 关系型数据库工作区 (`common.rs`, ~1000 行)

**适用数据库**: MySQL, PostgreSQL, SQLite, Oracle, SQL Server

**核心结构**:

```rust
pub struct CommonWorkspace {
    meta: DataSource,                         // 数据源元信息
    parent: WeakEntity<SqlerApp>,
    session: Option<Box<dyn DatabaseSession>>, // 连接实例（复用）

    tabs: Vec<TabItem>,                        // 标签页列表
    active_tab: SharedString,
    tables: Vec<SharedString>,                 // 表列表
    active_table: Option<SharedString>,
    sidebar_resize: Entity<ResizableState>,    // 侧边栏调整器
}

struct TabItem {
    id: SharedString,              // "relational-overview-tab" 或生成的 ID
    title: SharedString,           // "概览" 或 table_name
    content: TabContent,           // Overview 或 Table(TableContent)
    closable: bool,                // Overview 不可关闭
}

enum TabContent {
    Table(TableContent),           // 表数据标签
    Overview,                      // 概览标签
}

struct TableContent {
    id: SharedString,
    table: SharedString,
    columns: Vec<SharedString>,
    content: Entity<Table<DataTable>>,
    page_no: usize,
    page_size: usize,              // 固定 100
    total_rows: usize,
    order_rules: Vec<OrderRule>,   // 排序规则
    query_rules: Vec<QueryRule>,   // 筛选规则
    filter_enable: bool,
}

struct QueryRule {
    id: SharedString,
    value: Entity<InputState>,
    field: Entity<DropdownState<Vec<SharedString>>>,
    operator: Entity<DropdownState<Vec<SharedString>>>,
}

struct OrderRule {
    id: SharedString,
    field: Entity<DropdownState<Vec<SharedString>>>,
    order: Entity<DropdownState<Vec<SharedString>>>,  // "升序"/"降序"
}
```

**布局**: 左侧边栏（表列表）+ 右侧内容区（标签页）

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

- 避免重复创建连接开销
- 支持跨线程使用（DatabaseSession: Send）
- 失败自动重试

---

###### 标签页管理

**创建流程** (`create_table_tab`):

1. 生成唯一标签 ID: `relational-tab-table-data-{source_id}-{table_name}`
2. 检查标签是否已存在（避免重复）
3. 创建空 `TableContent`（Table 用空数据初始化）
4. 添加到标签列表并设置为活动标签
5. 调用 `reload_table_tab` 加载实际数据

**设计优势**:

- 避免代码重复（创建和刷新共用加载逻辑）
- 用户立即看到标签页（无需等待数据加载）
- 支持刷新功能

---

###### 数据加载 (`reload_table_tab`)

**执行流程**:

**① 准备阶段**（主线程）:

1. 从 `table_content` 获取当前页码、页大小、筛选/排序规则
2. 使用 `columns()` 方法获取列名（**新实现**）
3. 通过 `active_session()` 获取连接
4. 使用 `session.take()` 移动连接到闭包

**② 后台查询**（后台线程）:

1. 查询列名: `session.columns(&table)` - **统一接口**
2. 查询数据: 使用 `QueryReq::Builder` 构建查询
3. 查询总数: 使用 COUNT(*) 查询
4. 转换数据为 `Vec<Vec<SharedString>>`

**③ UI 更新**（主线程）:

1. 归还连接: `this.session = Some(session)`
2. 解构 `TablePage` 为独立变量（避免所有权冲突）
3. 更新 `data_tab` 的页码、总数、列名
4. 调用 `content.update()` 更新表格:
    - `delegate_mut().update_data(columns, rows)`: 更新数据
    - `refresh(cx)`: 重新准备列/行布局（**关键**！支持动态列）
    - `cx.notify()`: 触发重新渲染

**关键点**:

- `refresh(cx)` 必须调用，否则列结构不会更新
- 使用统一的 `columns()` trait 方法，消除数据库方言差异

---

###### 表格功能

**已实现功能**:

1. ✅ 分页导航（上一页/下一页）
2. ✅ 显示当前页范围和总数
3. ✅ 筛选/排序规则 UI（添加/删除规则）
4. ✅ 列筛选按钮
5. ✅ 数据筛选开关
6. ✅ 刷新表数据
7. ✅ 数据导出（打开传输窗口）

**TODO**:

- ❌ 筛选条件已收集但尚未写入 `QueryConditions`
- ❌ 排序规则已收集但尚未写入 `QueryConditions`
- ❌ 需要从 Dropdown 读取选中值并构建实际筛选/排序条件

---

##### RedisWorkspace - Redis 工作区 (`redis.rs`, ~360 行)

**核心结构**:

```rust
pub struct RedisWorkspace {
    meta: DataSource,
    parent: WeakEntity<SqlerApp>,
    session: Option<Box<dyn DatabaseSession>>,

    tabs: Vec<TabItem>,        // TabContent::Overview 或 Command
    active_tab: SharedString,
    sidebar_resize: Entity<ResizableState>,
}

enum TabContent {
    Command(CommandContent),   // 命令执行标签
    Overview,                  // 概览标签
}

struct CommandContent {
    id: SharedString,
    command_input: Entity<InputState>,
    result_table: Entity<Table<DataTable>>,
}
```

**布局**: 左侧简化侧边栏 + 右侧标签区

**特点**:

- 命令输入框 + 结果表格展示
- 支持多个命令标签页
- 工具栏: 刷新连接、新建命令

**TODO**:

- ❌ 实现命令执行逻辑（解析输入，调用 `session.query(QueryReq::Command {...})`）

---

##### MongoDBWorkspace - MongoDB 工作区 (`mongodb.rs`, ~480 行)

**核心结构**:

```rust
pub struct MongoDBWorkspace {
    meta: DataSource,
    parent: WeakEntity<SqlerApp>,
    session: Option<Box<dyn DatabaseSession>>,

    tabs: Vec<TabItem>,        // TabContent::Overview 或 Collection
    active_tab: SharedString,
    collections: Vec<SharedString>,
    active_collection: Option<SharedString>,
    sidebar_resize: Entity<ResizableState>,
}

enum TabContent {
    Collection(CollectionContent),  // 集合查看标签
    Overview,                       // 概览标签
}

struct CollectionContent {
    id: SharedString,
    collection: SharedString,
    filter_input: Entity<InputState>,
    content: Entity<Table<DataTable>>,
    page_no: usize,
    page_size: usize,           // 固定 100
    total_docs: usize,
}
```

**布局**: 左侧集合列表 + 右侧标签区

**特点**:

- JSON 筛选条件输入
- 分页导航（上一页/下一页）
- 显示文档范围和总数
- 集合列表双击打开
- 工具栏: 刷新集合、新建查询

**TODO**:

- ❌ 实现 JSON 筛选解析
- ❌ 实现文档查询和分页
- ❌ 调用 `session.query(QueryReq::Document {...})`

---

#### 2.5 数据传输 (`transfer/`)

##### 传输窗口 (`mod.rs`)

**核心结构**: `TransferWindow`

**功能**: 数据导入/导出

**当前状态**: 基础框架，具体逻辑待实现

---

### 3. 缓存系统 (`src/cache/mod.rs`, 165 行)

**职责**: 本地存储数据源配置和缓存数据

**核心结构**:

```rust
pub struct CacheApp {
    root: PathBuf,              // ~/.sqler
    sources: Vec<DataSource>,   // 数据源列表
}
```

#### 存储机制

**目录结构**:

```
~/.sqler/
  sources.db          # 加密的数据源列表
  cache/
    {uuid}/
      tables.json   # 表信息缓存
      queries.json  # 保存的查询
```

**加密算法**: AES-256-GCM (仅加密 sources.db)

- 密钥: 256位（当前硬编码）
- Nonce: 12字节（当前硬编码）

**初始化流程**:

1. 创建 `~/.sqler/cache/` 目录（自动创建父目录）
2. 尝试解密加载 `sources.db`
3. 解密失败或文件不存在则使用空列表

#### 核心 API

**数据源管理**:

- `sources()`: 获取数据源列表引用 `&[DataSource]` (零成本借用)
- `sources_mut()`: 获取可变引用 `&mut Vec<DataSource>`
- `sources_update()`: 加密并写入 `sources.db`

**表信息缓存**:

- `tables(uuid)`: 读取 `cache/{uuid}/tables.json`
- `tables_update(uuid, &[TableInfo])`: 写入表信息

**查询缓存**:

- `queries(uuid)`: 读取 `cache/{uuid}/queries.json`
- `queries_update(uuid, &[SavedQuery])`: 写入查询列表

**错误处理**:

- `CacheError` 枚举: Io, Serialization, Encryption, Decryption, DirectoryNotFound

#### 设计亮点

1. **单一数据源**: `SqlerApp` 直接使用 `cache.sources()`,无数据重复
2. **懒加载**: 按需创建 `cache/{uuid}/` 目录
3. **零成本抽象**: 返回引用避免克隆开销
4. **分离存储**: 加密数据源配置 + 明文 JSON 缓存

#### 当前状态

**已接入**:

- ✅ `SqlerApp.cache` 初始化并作为唯一数据源
- ✅ 新建数据源窗口保存逻辑已实现 (src/app/create/mod.rs:182-192)
- ✅ 首页展示真实数据源 (src/app/workspace/mod.rs:93)
- ✅ 创建工作区标签使用缓存数据 (src/app/mod.rs:147)

**待使用**:

- ❌ `tables()` / `tables_update()` 暂未被调用
- ❌ `queries()` / `queries_update()` 暂未被调用

---

### 4. 数据库驱动 (`src/driver/`, ~2800 行)

**职责**: 统一数据库操作接口、SQL 查询构建和连接管理

#### 4.1 核心接口 (`mod.rs`, 483 行)

**Trait 定义**:

```rust
pub trait DatabaseDriver {
    type Config;
    fn data_types(&self) -> Vec<Datatype>;
    fn check_connection(&self, config: &Self::Config) -> Result<(), DriverError>;
    fn create_connection(&self, config: &Self::Config) -> Result<Box<dyn DatabaseSession>, DriverError>;
}

pub trait DatabaseSession: Send {
    fn query(&mut self, request: QueryReq) -> Result<QueryResp, DriverError>;
    fn insert(&mut self, request: InsertReq) -> Result<UpdateResp, DriverError>;
    fn update(&mut self, request: UpdateReq) -> Result<UpdateResp, DriverError>;
    fn delete(&mut self, request: DeleteReq) -> Result<UpdateResp, DriverError>;
    fn tables(&mut self) -> Result<Vec<String>, DriverError>;
    fn columns(&mut self, table: &str) -> Result<Vec<String>, DriverError>;  // 新增
}
```

**请求/响应类型**:

| 类型           | 变体                                                                                             | 用途    |
|--------------|------------------------------------------------------------------------------------------------|-------|
| `QueryReq`   | `Sql {sql, args}` / `Builder {...}` / `Command {name, args}` / `Document {collection, filter}` | 查询请求  |
| `QueryResp`  | `Rows(Vec<HashMap>)` / `Value(Value)` / `Documents(Vec<Value>)`                                | 查询响应  |
| `InsertReq`  | `Sql` / `Command` / `Document`                                                                 | 插入请求  |
| `UpdateReq`  | `Sql` / `Command` / `Document`                                                                 | 更新请求  |
| `DeleteReq`  | `Sql` / `Command` / `Document`                                                                 | 删除请求  |
| `UpdateResp` | `{affected: u64}`                                                                              | 写操作响应 |

**查询条件类型**:

| 类型           | 字段                                              | 说明    |
|--------------|-------------------------------------------------|-------|
| `FilterCond` | `{field, operator, value}`                      | 筛选条件  |
| `OrderCond`  | `{field, ascending}`                            | 排序规则  |
| `Operator`   | Equal, GreaterThan, Like, In, Between, IsNull 等 | 比较操作符 |
| `ValueCond`  | Null, Bool, String, Number, List, Range         | 条件值   |

**数据源类型** (`DataSourceKind`, 按标准顺序):

```rust
pub enum DataSourceKind {
    MySQL,
    SQLite,
    Postgres,
    Oracle,
    SQLServer,
    Redis,
    MongoDB,
}
```

**DataSource 结构**:

```rust
pub struct DataSource {
    pub id: String,                          // UUID
    pub name: String,                        // 显示名称
    pub desc: String,                        // 描述
    pub kind: DataSourceKind,                // 数据库类型
    pub options: DataSourceOptions,          // 连接配置
    pub extras: Option<HashMap<String, Value>>,  // 额外信息（表列表缓存）
}
```

**全局函数**:

| 函数                        | 参数                   | 返回                                              | 说明           |
|---------------------------|----------------------|-------------------------------------------------|--------------|
| `get_datatypes(kind)`     | `DataSourceKind`     | `Vec<Datatype>`                                 | 获取数据库支持的数据类型 |
| `check_connection(opts)`  | `&DataSourceOptions` | `Result<(), DriverError>`                       | 测试连接         |
| `create_connection(opts)` | `&DataSourceOptions` | `Result<Box<dyn DatabaseSession>, DriverError>` | 创建会话         |
| `validate_sql(sql)`       | `&str`               | `Result<(), DriverError>`                       | 验证 SQL 非空    |

---

#### 4.2 驱动实现状态

| 驱动             | 行数  | 查询              | 写操作                    | tables()           | columns()            | 状态        |
|----------------|-----|-----------------|------------------------|--------------------|----------------------|-----------|
| **MySQL**      | 466 | ✅ SQL + Builder | ✅ INSERT/UPDATE/DELETE | ✅ SHOW TABLES      | ✅ SHOW COLUMNS FROM  | 全功能       |
| **PostgreSQL** | 464 | ✅ SQL + Builder | ✅ SQL方式                | ✅ pg_tables        | ✅ information_schema | 全功能       |
| **SQLite**     | 409 | ✅ SQL + Builder | ✅ SQL方式                | ✅ sqlite_master    | ✅ PRAGMA table_info  | 全功能       |
| **MongoDB**    | 428 | ✅ Document查询    | ✅ INSERT/UPDATE/DELETE | ✅ list_collections | ❌ 返回错误               | 文档型       |
| **Redis**      | 310 | ✅ Command执行     | ✅ Command方式            | ❌ 返回错误             | ❌ 返回错误               | 键值型       |
| **SQL Server** | 164 | ❌ 占位实现          | ❌ 占位实现                 | ❌ 占位实现             | ❌ 占位实现               | **未实现**   |
| **Oracle**     | 72  | -               | -                      | -                  | -                    | **仅配置结构** |

---

#### 4.3 MySQL 驱动 (`mysql.rs`, 466 行)

**实现**: 基于 `mysql` crate

**核心功能**:

1. **连接管理**: 支持字符集设置、连接池配置
2. **查询执行**: 支持参数化查询（占位符: `?`）
3. **标识符转义**: 反引号 `` ` `` (例: `` `table_name` ``)
4. **SQL 构建器**: WHERE/ORDER BY/LIMIT 拼接
5. **类型转换**: `mysql::Value` ↔ `serde_json::Value`

**columns() 实现**:

```
fn columns(&mut self, table: &str) -> Result<Vec<String>, DriverError> {
    let sql = format!("SHOW COLUMNS FROM `{}`", table.replace('`', "``"));
    let rows: Vec<mysql::Row> = self.conn.query(&sql)?;

    let mut columns = Vec::new();
    for row in rows {
        if let Some(value) = row.get(0) {
            columns.push(mysql_value_to_string(value));
        }
    }
    Ok(columns)
}
```

**特点**:

- 使用 `SHOW COLUMNS FROM` 语法
- 反引号转义防止 SQL 注入
- 提取第一列（字段名）

---

#### 4.4 PostgreSQL 驱动 (`postgres.rs`, 464 行)

**实现**: 基于 `postgres` crate

**核心功能**:

1. **连接管理**: 禁用 SSL
2. **查询执行**: 支持位置参数绑定 (`$1, $2, $3...`)
3. **标识符转义**: 双引号 `"` (例: `"table_name"`)
4. **类型转换**: PostgreSQL 原生类型 → JSON

**columns() 实现**:

```
fn columns(&mut self, table: &str) -> Result<Vec<String>, DriverError> {
    let sql = "SELECT column_name FROM information_schema.columns
               WHERE table_schema = 'public' AND table_name = $1
               ORDER BY ordinal_position";
    let rows = self.client.query(sql, &[&table])?;

    let mut columns = Vec::new();
    for row in rows {
        let column_name: String = row.get(0);
        columns.push(column_name);
    }
    Ok(columns)
}
```

**特点**:

- 使用标准 `information_schema.columns` 视图
- 参数化查询防止 SQL 注入
- 按列顺序排序

---

#### 4.5 SQLite 驱动 (`sqlite.rs`, 409 行)

**实现**: 基于 `rusqlite` crate

**核心功能**:

1. **连接管理**: 支持只读/创建模式
2. **查询执行**: 占位符 `?`
3. **标识符转义**: 双引号 `"`
4. **特性**: 支持内存数据库

**columns() 实现**:

```
fn columns(&mut self, table: &str) -> Result<Vec<String>, DriverError> {
    let sql = format!("PRAGMA table_info(\"{}\")", table.replace('"', "\"\""));
    let mut stmt = self.conn.prepare(&sql)?;

    let mut columns = Vec::new();
    let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;

    for row in rows {
        columns.push(row?);
    }
    Ok(columns)
}
```

**特点**:

- 使用 SQLite 特有的 `PRAGMA table_info()` 命令
- 提取第 2 列（索引 1）作为列名
- 双引号转义防止注入

---

#### 4.6 MongoDB 驱动 (`mongodb.rs`, 428 行)

**实现**: 基于 `mongodb` crate

**核心功能**:

1. **连接管理**: 支持 connection string 或 host 列表
2. **文档型 CRUD**: find/insert_one/update_many/delete_many
3. **响应转换**: BSON → JSON
4. **集合名支持**: 数据库前缀 (`db.collection`)

**columns() 实现**:

```
fn columns(&mut self, _table: &str) -> Result<Vec<String>, DriverError> {
    Err(DriverError::Other("MongoDB 作为文档数据库不支持固定列结构查询".into()))
}
```

**特点**:

- 文档型数据库无固定列结构
- 返回明确的错误信息

---

#### 4.7 Redis 驱动 (`redis.rs`, 310 行)

**实现**: 基于 `redis` crate

**核心功能**:

1. **连接管理**: 支持 URL 连接字符串
2. **命令执行**: 支持任意 Redis 命令
3. **响应转换**: Redis 类型 → JSON
4. **影响行数估算**: 基于返回值类型

**columns() 实现**:

```
fn columns(&mut self, _table: &str) -> Result<Vec<String>, DriverError> {
    Err(DriverError::Other("Redis 作为键值数据库不支持列结构查询".into()))
}
```

**特点**:

- 键值型数据库无表和列概念
- 返回明确的错误信息

---

#### 4.8 SQL Server 驱动 (`sqlserver.rs`, 164 行)

**状态**: 占位实现

**所有操作**: 返回"暂未实现"错误

```
fn columns(&mut self, _table: &str) -> Result<Vec<String>, DriverError> {
    Err(DriverError::Other("SQL Server 查询列信息暂未实现".into()))
}
```

**TODO**: 需要完整实现连接和查询逻辑

---

#### 4.9 Oracle 驱动 (`oracle.rs`, 72 行)

**状态**: 仅包含配置结构

```
pub enum OracleAddress {
    ServiceName(String),
    Sid(String),
}

pub struct OracleOptions {
    pub host: String,
    pub port: u16,
    pub address: OracleAddress,
    pub username: String,
    pub password: Option<String>,
    pub wallet_path: Option<String>,
}
```

**TODO**: 需要完整实现驱动

---

### 5. 数据源配置类型

#### DataSourceOptions 枚举

```
pub enum DataSourceOptions {
    MySQL(MySQLOptions),
    SQLite(SQLiteOptions),
    Postgres(PostgresOptions),
    Oracle(OracleOptions),
    SQLServer(SQLServerOptions),
    Redis(RedisOptions),
    MongoDB(MongoDBOptions),
}
```

#### 各数据库配置

| 数据库            | 关键字段                                                                           | endpoint() 示例                  |
|----------------|--------------------------------------------------------------------------------|--------------------------------|
| **MySQL**      | host, port, username, password, database, charset, use_tls                     | `mysql://user@host:3306/db`    |
| **PostgreSQL** | host, port, username, password, database, use_tls                              | `postgres://user@host:5432/db` |
| **SQLite**     | filepath, password, read_only                                                  | `sqlite:///path/to/db`         |
| **Oracle**     | host, port, address (ServiceName/Sid), username, password, wallet_path         | `oracle://host:1521?sid=xe`    |
| **SQLServer**  | host, port, database, username, password, auth, instance                       | `sqlserver://host:1433/db`     |
| **Redis**      | host, port, username, password, use_tls                                        | `redis://host:6379`            |
| **MongoDB**    | connection_string/hosts, replica_set, auth_source, username, password, use_tls | `mongodb://host:27017`         |

**通用方法**:

- `endpoint()`: 生成连接字符串（隐藏密码）
- `overview()`: 生成概览信息列表

---

### 6. 测试数据脚本 (`scripts/test/`)

**职责**: 为常见数据库批量生成演示数据，统一 10 张电商业务表模型，每表≥1000 行

**支持的数据库**:

| 脚本                   | 数据库        | 特性                     |
|----------------------|------------|------------------------|
| `mysql_init.sql`     | MySQL      | 递归 CTE 批量插入，触发器        |
| `postgres_init.sql`  | PostgreSQL | 枚举类型，`generate_series` |
| `sqlite_init.sql`    | SQLite     | 递归 CTE，外键约束            |
| `sqlserver_init.sql` | SQL Server | CTE + 系统表构造序列          |
| `oracle_init.sql`    | Oracle     | PL/SQL 循环，枚举校验         |
| `redis_init.redis`   | Redis      | Lua 批量写入哈希结构           |
| `mongodb_init.js`    | MongoDB    | 批量插入文档，关键索引            |

**辅助工具**:

- `generate_csv_data.py`: Python 数据生成器
- `csv/`: 预生成的 10 张电商 CSV（每表≥1000 行）

**表模型**: customers, orders, products, order_items, categories, reviews, addresses, payments, shipping, inventory

---

### 7. 静态资源 (`assets/`)

**内容**: 数据库图标等静态文件

**图标列表**:

- `icons/mysql.svg`
- `icons/postgresql.svg`
- `icons/sqlite.svg`
- `icons/oracle.svg`
- `icons/sqlserver.svg`
- `icons/redis.svg`
- `icons/mongodb.svg`

**加载**: 通过 `FsAssets` 注册到 GPUI

---

### 8. 项目配置 (`Cargo.toml`)

**核心依赖**:

| 分类        | 依赖                                        |
|-----------|-------------------------------------------|
| **UI 框架** | gpui, gpui-component                      |
| **加密**    | aes-gcm                                   |
| **序列化**   | serde, serde_json                         |
| **数据库驱动** | mysql, postgres, rusqlite, mongodb, redis |
| **工具**    | dirs, uuid, thiserror                     |

---

## 功能现状

### 已实现功能 ✅

#### 主窗口

1. ✅ 顶部标签栏（支持多标签切换）
2. ✅ 主题切换按钮（亮色/暗色）
3. ✅ 新建数据源浮动窗口

#### 首页

1. ✅ 网格卡片展示数据源
2. ✅ 双击打开工作区标签
3. ✅ 显示数据源图标和连接地址

#### 关系型数据库工作区 (CommonWorkspace)

1. ✅ 左侧表列表导航
2. ✅ 动态标签页管理
3. ✅ 分页查询（上一页/下一页）
4. ✅ 数据表格展示（支持动态列）
5. ✅ 筛选/排序 UI（添加/删除规则）
6. ✅ 连接复用机制
7. ✅ 刷新表数据
8. ✅ 统一的 `columns()` 方法（消除 SQL 方言差异）

#### Redis 工作区

1. ✅ 概览标签
2. ✅ 命令标签页 UI
3. ✅ 侧边栏布局

#### MongoDB 工作区

1. ✅ 概览标签
2. ✅ 集合列表侧边栏
3. ✅ 集合标签页 UI
4. ✅ JSON 筛选输入框
5. ✅ 分页导航 UI

#### 数据库驱动

1. ✅ MySQL/PostgreSQL/SQLite 驱动完整实现
2. ✅ MongoDB/Redis 驱动完整实现
3. ✅ 统一的 `DatabaseSession` trait
4. ✅ `columns()` 方法在所有关系型数据库中实现
5. ✅ 参数化查询防止 SQL 注入

#### 新建数据源窗口

1. ✅ 数据库类型选择
2. ✅ 7 种数据库的表单实现
3. ✅ 测试连接功能（异步调用 `check_connection()`）
4. ✅ 保存到缓存（已实现并接入）
5. ✅ 状态提示（测试中/成功/失败）

#### 缓存系统

1. ✅ AES-256-GCM 加密（sources.db）
2. ✅ JSON 存储（tables.json, queries.json）
3. ✅ 目录结构：`~/.sqler/cache/{uuid}/`
4. ✅ 单一数据源原则（消除数据重复）
5. ✅ 零成本抽象（返回引用避免克隆）
6. ✅ 懒加载（按需创建目录）
7. ✅ 数据源管理已接入 UI
8. ❌ 表信息缓存暂未使用
9. ❌ 查询缓存暂未使用

---

### 待完成功能 ❌

#### 高优先级

1. **筛选/排序功能**
    - 从 Dropdown 读取选中值
    - 构建实际的 `QueryConditions`
    - 将条件注入 SQL 查询

2. **表信息和查询缓存使用**
    - 工作区加载表列表时读取/更新 `tables.json`
    - 实现保存查询功能，使用 `queries.json`
    - 避免重复查询表元信息

3. **数据源编辑和删除功能**
    - 首页右键菜单（编辑/删除）
    - 编辑窗口（复用 CreateWindow）
    - 删除确认对话框

4. **Redis/MongoDB 工作区功能实现**
    - Redis 命令执行逻辑
    - MongoDB 文档查询和筛选
    - 结果解析和展示

5. **SQL Server 驱动完整实现**
    - 连接管理
    - 查询执行
    - tables() 和 columns() 实现

6. **Oracle 驱动完整实现**
    - 基于 `oracle` crate 实现
    - 连接管理和查询

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

4. **错误处理优化**
    - 友好的错误提示
    - 连接失败重试
    - 超时处理

#### 低优先级

1. **高级功能**
    - 查询历史
    - 收藏查询
    - 数据库结构可视化
    - 性能监控
    - 查询执行计划

2. **UI 增强**
    - 键盘快捷键
    - 右键菜单
    - 拖拽导入文件

---

## 关键设计亮点

### 1. Trait 驱动架构

- `DatabaseDriver` 和 `DatabaseSession` 统一多数据库接口
- 通过 `columns()` trait 方法消除 SQL 方言差异
- 支持 SQL、文档、命令三种查询模式

### 2. 连接复用策略

- 使用 `Option<Box<dyn DatabaseSession>>` 实现懒加载和复用
- 支持跨线程移动（DatabaseSession: Send）
- 失败自动重试

### 3. 参数化查询

- 各驱动正确处理占位符和参数绑定
- 防止 SQL 注入攻击
- 标识符转义（反引号、双引号）

### 4. 工作区架构

- `WorkspaceState` 枚举支持多种工作区类型
- Common 工作区统一处理所有关系型数据库
- 专用工作区（Redis、MongoDB）针对性优化

### 5. 数据源 ID 设计

- 使用 UUID 作为标签 ID
- 避免 TabId 包装类型
- 简化查找和路由逻辑

### 6. 缓存系统设计

**单一数据源原则**:
- `SqlerApp` 直接使用 `cache.sources()` 获取数据源
- 消除数据重复，无需手动同步
- 编译器保证数据一致性

**零成本抽象**:
- `sources()` 返回 `&[DataSource]` 避免克隆
- 只读访问零开销
- 需要修改时使用 `sources_mut()`

**分离存储**:
- `sources.db`: AES-256-GCM 加密（保护敏感信息）
- `tables.json` / `queries.json`: 明文 JSON（缓存数据）

**懒加载**:
- 按需创建 `cache/{uuid}/` 目录
- 文件不存在返回空列表，不阻塞系统

### 7. 动态列支持

- DataTable 通过 `update_data()` 和 `refresh()` 支持动态列数
- 无需重建表格组件

### 8. 数据源排序标准

- 统一排序：MySQL → SQLite → Postgres → Oracle → SQLServer → Redis → MongoDB
- 所有 match 语句遵循相同顺序
- 提高代码一致性和可维护性

---

## 代码统计

| 模块      | 文件数    | 代码行数（估算）  |
|---------|--------|-----------|
| app/    | 28     | ~3500     |
| driver/ | 8      | ~2800     |
| cache/  | 1      | ~165      |
| model/  | 3      | ~150      |
| main.rs | 1      | ~90       |
| **总计**  | **41** | **~6700** |

---

## 项目状态

**当前阶段**: 核心功能开发中

**可用功能**:

- ✅ MySQL/PostgreSQL/SQLite 数据源浏览和查询
- ✅ 多标签管理
- ✅ 分页导航
- ✅ 连接复用
- ✅ 统一的列查询接口
- ✅ 新建数据源窗口（测试连接+保存）
- ✅ 缓存系统（单一数据源原则）

**开发中**:

- 🚧 筛选/排序逻辑
- 🚧 Redis/MongoDB 工作区功能
- 🚧 表信息和查询缓存使用

**待开发**:

- 📋 数据源编辑和删除
- 📋 SQL Server/Oracle 驱动
- 📋 查询编辑器
- 📋 数据编辑
- 📋 导入/导出

---

## 贡献指南

### 代码规范

1. **导入顺序**:
   ```rust
   // 1. 标准库导入
   use std::sync::Arc;

   // 2. 外部 crate 导入（按字母顺序）
   use gpui::{prelude::*, *};
   use serde::{Deserialize, Serialize};

   // 3. 当前 crate 导入（按模块分组）
   use crate::{
       app::comps::DataTable,
       driver::{DatabaseDriver, DriverError},
   };
   ```

2. **数据源排序**: 所有涉及 `DataSourceKind` 的 match 语句必须遵循标准顺序：
   MySQL → SQLite → Postgres → Oracle → SQLServer → Redis → MongoDB

3. **错误处理**: 优先使用 `Result<T, DriverError>`，避免 panic

4. **命名约定**:
    - 结构体：大驼峰 (PascalCase)
    - 函数/变量：蛇形 (snake_case)
    - 常量：全大写蛇形 (UPPER_SNAKE_CASE)

### 测试

- 在 `scripts/test/` 目录下提供测试数据脚本
- 每个数据库至少 10 张表，每表≥1000 行数据
- 覆盖常见数据类型和关系

---

**最后更新**: 2025-01-07 (完善缓存系统,消除数据重复)
