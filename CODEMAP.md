# Sqler 代码地图 v5.0 (Workspace 架构)

## 项目概览

### 基本信息

- **名称**: `sqler`
- **目标**: 桌面化多标签数据库管理器，支持多类型数据库的连接、浏览、查询和管理
- **版本**: v1.0.0
- **代码总行数**: 9,393 行 (21 个 .rs 文件)
- **架构模式**: Cargo Workspace (2 crates)

### 技术栈

- **核心框架**: Rust + GPUI 0.2.2 + gpui-component 0.5.0
- **数据库驱动**: mysql, postgres, rusqlite, mongodb, redis 等
- **加密**: AES-256-GCM (数据源配置加密存储)
- **序列化**: serde, serde_json
- **日志**: tracing (终端+文件双重输出)

### Workspace 架构

**Crate 组织**:

```
sqler/
  Cargo.toml              # Workspace 根配置
  crates/
    sqler-core/           # 核心库 crate (3145 行)
      Cargo.toml
      src/
        lib.rs            # 数据模型定义 (734 行)
        driver/           # 数据库驱动 (2240 行)
        cache/            # 缓存系统 (171 行)
    sqler-app/            # 应用程序 crate (6248 行)
      Cargo.toml
      src/
        main.rs           # 应用入口 (124 行)
        app/              # 应用层 (391 行)
        comps/            # 公共组件 (308 行)
        create/           # 数据源创建 (1148 行)
        workspace/        # 工作区 (3273 行)
        transfer/         # 数据传输 (1003 行)
        subtask/          # 预留模块 (1 行)
      assets/             # 静态资源
        icons/
```

**依赖关系**:

- `sqler-app` **依赖** `sqler-core`
- `sqler-core` **零依赖** `sqler-app`（单向依赖，消除循环）
- 编译产物: 单个二进制 `sqler`（位于 sqler-app）

**Workspace 优势**:

1. **职责分离**: 核心逻辑（driver、cache、model）与 UI 层完全分离
2. **独立编译**: sqler-core 可单独编译和测试
3. **依赖管理**: workspace.dependencies 统一版本管理
4. **代码复用**: sqler-core 可被其他 crate 复用（如 CLI 工具、Web 服务）
5. **构建优化**: 核心库变更时 UI 层无需重新编译

---

## sqler-core crate（核心库，3145 行）

**路径**: `crates/sqler-core/`

**职责**: 数据模型定义、数据库驱动抽象、缓存系统

**依赖**:
- 数据库驱动: mysql, postgres, rusqlite, mongodb, redis
- 加密: aes-gcm
- 序列化: serde, serde_json
- 日志: tracing

**导出接口**: 通过 `pub use` 统一导出核心类型

```
pub use driver::{
    DatabaseDriver, DatabaseSession, DriverError,
    ExecReq, ExecResp, FilterCond, Operator, OrderCond,
    Paging, QueryReq, QueryResp, ValueCond,
    check_connection, create_connection, supp_kinds,
};

pub use cache::{AppCache, ArcCache, CacheError};
```

---

### 1. lib.rs - 核心类型定义（734 行）

**路径**: `crates/sqler-core/src/lib.rs`

**职责**: 定义所有核心数据结构和配置类型

#### 1.1 核心类型

##### TableInfo

```
pub struct TableInfo {
    pub name: String,
    pub row_count: Option<u64>,
    pub size_bytes: Option<u64>,
    pub last_accessed: Option<u64>,
}
```

##### ColumnInfo

```
pub struct ColumnInfo {
    pub name: String,
    pub kind: String,
    pub comment: String,
    pub nullable: bool,
    pub primary_key: bool,
    pub default_value: String,
    pub max_length: u64,
    pub auto_increment: bool,
}
```

##### SavedQuery

```
pub struct SavedQuery {
    pub name: String,
    pub content: String,
}
```

#### 1.2 数据源类型

##### DataSource

```
pub struct DataSource {
    pub id: String,                     // UUID
    pub name: String,                   // 显示名称
    pub kind: DataSourceKind,           // 数据库类型
    pub options: DataSourceOptions,     // 连接配置
}
```

**核心方法**:
- `new(name, kind, options)`: 创建数据源（自动生成 UUID）
- `display_endpoint()`: 生成连接字符串（隐藏密码）
- `display_overview()`: 生成概览信息列表

##### DataSourceKind

```
#[repr(u8)]
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

**标准顺序**: MySQL → SQLite → Postgres → Oracle → SQLServer → Redis → MongoDB

**方法**:
- `all()`: 返回所有类型
- `image()`: 返回图标路径
- `label()`: 返回显示名称
- `description()`: 返回描述信息

#### 1.3 列类型枚举

```
pub enum ColumnKind {
    TinyInt, SmallInt, Int, BigInt,
    Float, Double, Decimal,
    Char, VarChar, Text,
    Binary, VarBinary, Blob,
    Date, Time, DateTime, Timestamp,
    Boolean, Json, Uuid,
    Enum, Set,
    Document, Array,  // MongoDB
    String, List, Hash, ZSet,  // Redis
    Unknown,
}
```

#### 1.4 数据源配置类型

##### DataSourceOptions

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

##### MySQLOptions

```
pub struct MySQLOptions {
    pub host: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub database: String,
    pub use_tls: bool,
}
```

**方法**:
- `endpoint()`: 生成连接字符串（如 `mysql://host:3306/db`）
- `overview()`: 返回 `Vec<(&'static str, String)>` 格式的概览信息

**默认值**:
- host: `127.0.0.1`
- port: `3306`
- username: `root`

##### PostgresOptions

```
pub struct PostgresOptions {
    pub host: String,
    pub port: String,
    pub database: String,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
}
```

**方法**: `endpoint()`, `overview()`

**默认值**:
- host: `127.0.0.1`
- port: `5432`
- username: `postgres`

##### SQLiteOptions

```
pub struct SQLiteOptions {
    pub readonly: bool,
    pub filepath: String,
    pub password: Option<String>,
}
```

**方法**: `endpoint()`, `overview()`

**特点**:
- 支持只读模式
- 支持文件加密
- endpoint() 返回文件名而非完整路径

##### OracleOptions

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

**默认值**:
- host: `127.0.0.1`
- port: `1521`
- address: `ServiceName("xe")`
- username: `system`

##### SQLServerOptions

```
pub enum SQLServerAuth {
    SqlPassword,
    Integrated,
}

pub struct SQLServerOptions {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub auth: SQLServerAuth,
    pub instance: Option<String>,
}
```

**默认值**:
- host: `127.0.0.1`
- port: `1433`
- auth: `SqlPassword`

##### RedisOptions

```
pub enum RedisKind {
    Cluster,
    Standalone,
}

pub struct RedisOptions {
    pub kind: RedisKind,
    pub host: String,
    pub port: String,
    pub nodes: String,        // 集群模式节点列表
    pub username: Option<String>,
    pub password: Option<String>,
    pub use_tls: bool,
}
```

**默认值**:
- kind: `Standalone`
- host: `127.0.0.1`
- port: `6379`

**特点**:
- 支持单机和集群模式
- 集群模式使用 nodes 字段（逗号分隔）

##### MongoDBOptions

```
pub struct MongoDBHost {
    pub host: String,
    pub port: u16,
}

pub struct MongoDBOptions {
    pub connection_string: Option<String>,
    pub hosts: Vec<MongoDBHost>,
    pub replica_set: Option<String>,
    pub auth_source: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub use_tls: bool,
}
```

**默认值**:
- hosts: `[MongoDBHost { host: "127.0.0.1", port: 27017 }]`

**特点**:
- 支持连接字符串或主机列表两种配置方式
- 支持副本集配置

---

### 2. driver/ - 数据库驱动（2240 行）

**路径**: `crates/sqler-core/src/driver/`

**职责**: 统一数据库操作接口、SQL 查询构建和连接管理

#### 2.1 核心接口（mod.rs，288 行）

##### DatabaseDriver Trait

```
pub trait DatabaseDriver {
    type Config;
    fn supp_kinds(&self) -> Vec<ColumnKind>;
    fn check_connection(&self, config: &Self::Config) -> Result<(), DriverError>;
    fn create_connection(&self, config: &Self::Config) -> Result<Box<dyn DatabaseSession>, DriverError>;
}
```

##### DatabaseSession Trait

```
pub trait DatabaseSession: Send {
    fn exec(&mut self, req: ExecReq) -> Result<ExecResp, DriverError>;
    fn query(&mut self, req: QueryReq) -> Result<QueryResp, DriverError>;
    fn tables(&mut self) -> Result<Vec<TableInfo>, DriverError>;
    fn columns(&mut self, table: &str) -> Result<Vec<ColumnInfo>, DriverError>;
}
```

**特点**:
- `Send` trait: 支持跨线程传递
- 统一写操作: `exec()` 合并了原来的 insert/update/delete 方法
- 多模式支持: SQL/Builder/Command/Document

##### 请求/响应类型

| 类型           | 变体/字段                                                                                                                              | 用途          |
|--------------|------------------------------------------------------------------------------------------------------------------------------------|-------------|
| `QueryReq`   | `Sql {sql, args}` / `Builder {table, columns, paging, orders, filters}` / `Command {name, args}` / `Document {collection, filter}` | 查询请求        |
| `QueryResp`  | `Rows {cols, rows}` / `Value(Value)` / `Documents(Vec<Value>)`                                                                     | 查询响应        |
| `ExecReq`    | `Sql {sql}` / `Command {name, args}` / `Document {collection, operation}`                                                          | 执行请求（写操作）   |
| `ExecResp`   | `{affected: u64}`                                                                                                                  | 执行响应（受影响行数） |
| `DocumentOp` | `Insert {document}` / `Update {filter, update}` / `Delete {filter}`                                                                | 文档操作类型      |

##### 查询条件类型

```
pub struct Paging {
    page: usize,
    size: usize,
}

pub enum Operator {
    Equal, NotEqual, GreaterThan, LessThan,
    GreaterOrEqual, LessOrEqual,
    Like, NotLike, In, NotIn, Between,
    IsNull, IsNotNull,
}

pub struct FilterCond {
    pub field: String,
    pub operator: Operator,
    pub value: ValueCond,
}

pub enum ValueCond {
    Null,
    Bool(bool),
    String(String),
    Number(f64),
    List(Vec<String>),
    Range(String, String),
}

pub struct OrderCond {
    pub field: String,
    pub ascending: bool,
}
```

##### 全局函数

```
pub fn supp_kinds(kind: DataSourceKind) -> Vec<ColumnKind>
pub fn check_connection(opts: &DataSourceOptions) -> Result<(), DriverError>
pub fn create_connection(opts: &DataSourceOptions) -> Result<Box<dyn DatabaseSession>, DriverError>
pub fn validate_sql(sql: &str) -> Result<(), DriverError>
pub fn escape_quote(s: &str) -> String
pub fn escape_backtick(s: &str) -> String
```

#### 2.2 驱动实现状态

| 驱动             | 文件             | 行数  | 查询              | 写操作       | tables()           | columns()            | 状态      |
|----------------|----------------|-----|-----------------|-----------|--------------------|----------------------|---------|
| **MySQL**      | `mysql.rs`     | 405 | ✅ SQL + Builder | ✅ exec 方法 | ✅ SHOW TABLES      | ✅ SHOW COLUMNS FROM  | 全功能     |
| **PostgreSQL** | `postgres.rs`  | 473 | ✅ SQL + Builder | ✅ exec 方法 | ✅ pg_tables        | ✅ information_schema | 全功能     |
| **SQLite**     | `sqlite.rs`    | 385 | ✅ SQL + Builder | ✅ exec 方法 | ✅ sqlite_master    | ✅ PRAGMA table_info  | 全功能     |
| **MongoDB**    | `mongodb.rs`   | 291 | ✅ Document查询    | ✅ exec 方法 | ✅ list_collections | ❌ 返回错误               | 文档型     |
| **Redis**      | `redis.rs`     | 320 | ✅ Command执行     | ✅ exec 方法 | ❌ 返回错误             | ❌ 返回错误               | 键值型     |
| **SQL Server** | `sqlserver.rs` | 77  | ❌ 占位实现          | ❌ 占位实现    | ❌ 占位实现             | ❌ 占位实现               | **未实现** |
| **Oracle**     | `oracle.rs`    | 1   | -               | -         | -                  | -                    | **仅注释** |

**注**: 所有写操作（INSERT/UPDATE/DELETE）统一通过 `exec()` 方法处理

#### 2.3 MySQL 驱动（mysql.rs，405 行）

**实现**: 基于 `mysql` crate

**核心功能**:

1. **连接管理**: 支持字符集设置、连接池配置
2. **查询执行**: 支持参数化查询（占位符: `?`）
3. **标识符转义**: 反引号 `` ` `` (例: `` `table_name` ``)
4. **SQL 构建器**: WHERE/ORDER BY/LIMIT 拼接
5. **类型转换**: `mysql::Value` ↔ `serde_json::Value`

**columns() 实现**:

```
fn columns(&mut self, table: &str) -> Result<Vec<ColumnInfo>, DriverError> {
    let sql = format!("SHOW FULL COLUMNS FROM `{}`", escape_backtick(table));
    let rows: Vec<mysql::Row> = self.conn.query(&sql)?;

    rows.iter().map(|row| {
        Ok(ColumnInfo {
            name: row.get(0).ok_or(...)?,
            kind: row.get(1).ok_or(...)?,
            nullable: row.get(2).map(|s: String| s == "YES").unwrap_or(false),
            primary_key: row.get(3).map(|s: String| s == "PRI").unwrap_or(false),
            default_value: row.get(4).unwrap_or_default(),
            comment: row.get(8).unwrap_or_default(),
            // ...
        })
    }).collect()
}
```

**特点**:
- 使用 `SHOW FULL COLUMNS FROM` 语法获取完整列信息
- 反引号转义防止 SQL 注入
- 返回 `ColumnInfo` 结构，包含列的所有元数据

#### 2.4 PostgreSQL 驱动（postgres.rs，473 行）

**实现**: 基于 `postgres` crate

**核心功能**:

1. **连接管理**: 禁用 SSL
2. **查询执行**: 支持位置参数绑定 (`$1, $2, $3...`)
3. **标识符转义**: 双引号 `"` (例: `"table_name"`)
4. **类型转换**: PostgreSQL 原生类型 → JSON

**columns() 实现**:

```
fn columns(&mut self, table: &str) -> Result<Vec<ColumnInfo>, DriverError> {
    let sql = "
        SELECT column_name, data_type, is_nullable, column_default,
               character_maximum_length
        FROM information_schema.columns
        WHERE table_schema = 'public' AND table_name = $1
        ORDER BY ordinal_position";

    let rows = self.client.query(sql, &[&table])?;

    rows.iter().map(|row| {
        Ok(ColumnInfo {
            name: row.get(0),
            kind: row.get(1),
            nullable: row.get::<_, String>(2) == "YES",
            default_value: row.get(3).unwrap_or_default(),
            // ...
        })
    }).collect()
}
```

**特点**:
- 使用标准 `information_schema.columns` 视图
- 参数化查询防止 SQL 注入
- 按列顺序排序

#### 2.5 SQLite 驱动（sqlite.rs，385 行）

**实现**: 基于 `rusqlite` crate

**核心功能**:

1. **连接管理**: 支持只读/创建模式
2. **查询执行**: 占位符 `?`
3. **标识符转义**: 双引号 `"`
4. **特性**: 支持内存数据库

**columns() 实现**:

```
fn columns(&mut self, table: &str) -> Result<Vec<ColumnInfo>, DriverError> {
    let sql = format!("PRAGMA table_info(\"{}\")", escape_quote(table));
    let mut stmt = self.conn.prepare(&sql)?;

    let mut columns = vec![];
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        columns.push(ColumnInfo {
            name: row.get(1)?,           // 列名在第 2 列 (索引 1)
            kind: row.get(2)?,
            nullable: row.get(3).map(|v: i64| v == 0).unwrap_or(false),
            primary_key: row.get(5).map(|v: i64| v == 1).unwrap_or(false),
            default_value: row.get(4).unwrap_or_default(),
            // ...
        });
    }
    Ok(columns)
}
```

**特点**:
- 使用 SQLite 特有的 `PRAGMA table_info()` 命令
- 双引号转义防止注入

#### 2.6 MongoDB 驱动（mongodb.rs，291 行）

**实现**: 基于 `mongodb` crate

**核心功能**:

1. **连接管理**: 支持 connection string 或 host 列表
2. **文档型 CRUD**: find/insert_one/update_many/delete_many
3. **响应转换**: BSON → JSON
4. **集合名支持**: 数据库前缀 (`db.collection`)

**columns() 实现**:

```
fn columns(&mut self, _table: &str) -> Result<Vec<ColumnInfo>, DriverError> {
    Err(DriverError::Other("MongoDB 作为文档数据库不支持固定列结构查询".into()))
}
```

**特点**: 文档型数据库无固定列结构，返回明确的错误信息

#### 2.7 Redis 驱动（redis.rs，320 行）

**实现**: 基于 `redis` crate

**核心功能**:

1. **连接管理**: 支持 URL 连接字符串
2. **命令执行**: 支持任意 Redis 命令
3. **响应转换**: Redis 类型 → JSON
4. **影响行数估算**: 基于返回值类型

**columns() 实现**:

```
fn columns(&mut self, _table: &str) -> Result<Vec<ColumnInfo>, DriverError> {
    Err(DriverError::Other("Redis 作为键值数据库不支持列结构查询".into()))
}
```

**特点**: 键值型数据库无表和列概念，返回明确的错误信息

#### 2.8 SQL Server 驱动（sqlserver.rs，77 行）

**状态**: 占位实现

所有方法注释标记 `/// SQL Server 驱动占位实现。`

所有操作返回"暂未实现"错误。

**TODO**: 需要完整实现连接和查询逻辑

#### 2.9 Oracle 驱动（oracle.rs，1 行）

**状态**: 仅包含注释

```
// Oracle 驱动相关类型定义已移至 crates/sqler-core/src/lib.rs
```

配置结构（OracleOptions、OracleAddress）已在 lib.rs 中定义。

**TODO**: 需要完整实现驱动

---

### 3. cache/ - 缓存系统（171 行）

**路径**: `crates/sqler-core/src/cache/mod.rs`

**职责**: 本地存储数据源配置和缓存数据

#### 核心结构

```
pub struct AppCache {
    sources: Vec<DataSource>,     // 数据源列表
    sources_path: PathBuf,        // ~/.sqler/sources.db
    sources_cache: PathBuf,       // ~/.sqler/cache/
}

pub type ArcCache = Arc<RwLock<AppCache>>;
```

#### 存储机制

**目录结构**:

```
~/.sqler/
  sources.db          # 加密的数据源列表
  logs/               # 日志文件
  cache/
    {uuid}/
      tables.json   # 表信息缓存
      queries.json  # 保存的查询
```

**加密算法**: AES-256-GCM (仅加密 sources.db)

- 密钥: 256位（硬编码常量 `ENCRYPTION_KEY`）
- Nonce: 12字节（硬编码常量 `NONCE`）
- ⚠️ 生产环境应从环境变量或配置读取

**初始化流程**:

1. 创建 `~/.sqler/cache/` 目录（自动创建父目录）
2. 尝试解密加载 `sources.db`
3. 解密失败或文件不存在则使用空列表
4. 返回 `ArcCache = Arc::new(RwLock::new(cache))`

#### 核心 API

**数据源管理**:

```
pub fn sources(&self) -> &[DataSource]
pub fn sources_mut(&mut self) -> &mut Vec<DataSource>
pub fn sources_update(&mut self) -> Result<(), CacheError>
```

**使用示例**:
```
// 读取数据源
let cache = app.cache.read().unwrap();
let sources = cache.sources();

// 修改数据源
let mut cache = app.cache.write().unwrap();
cache.sources_mut().push(source);
cache.sources_update()?;
```

**表信息缓存**:

```
pub fn tables(&self, uuid: &str) -> Result<Vec<TableInfo>, CacheError>
pub fn tables_update(&self, uuid: &str, tables: &[TableInfo]) -> Result<(), CacheError>
```

**使用示例**:
```
// 初始化工作区时从缓存读取
let tables = cache.read().unwrap()
    .tables(&source.id)
    .unwrap_or_default();

// 刷新表列表时更新缓存
let cache = self.cache.write().unwrap();
cache.tables_update(&self.source.id, &tables)?;
```

**查询缓存**:

```
pub fn queries(&self, uuid: &str) -> Result<Vec<SavedQuery>, CacheError>
pub fn queries_update(&self, uuid: &str, queries: &[SavedQuery]) -> Result<(), CacheError>
```

**错误处理**:

```
pub enum CacheError {
    Io(io::Error),
    Serialization(serde_json::Error),
    Encryption(String),
    Decryption(String),
    DirectoryNotFound,
}
```

#### 设计亮点

1. **Arc<RwLock<>> 模式**: 支持跨线程共享和并发读写
2. **读写分离**: 多个读者或单个写者，保证数据一致性
3. **单一数据源**: `SqlerApp` 和所有工作区共享同一个 `ArcCache` 实例
4. **懒加载**: 按需创建 `cache/{uuid}/` 目录
5. **零成本抽象**: `sources()` 返回引用避免克隆开销
6. **分离存储**: 加密数据源配置 + 明文 JSON 缓存

---

## sqler-app crate（应用程序，6248 行）

**路径**: `crates/sqler-app/`

**职责**: UI 层、用户交互、窗口管理

**依赖**:
- sqler-core (workspace)
- gpui, gpui-component
- indexmap, lsp-types
- tracing, tracing-appender, tracing-subscriber

---

### 1. main.rs - 应用入口（124 行）

**路径**: `crates/sqler-app/src/main.rs`

**职责**: 程序入口，应用初始化和窗口创建

**核心功能**:

1. 注册本地资源加载器 `FsAssets` (从 `crates/sqler-app/assets/` 加载资源)
2. 初始化 GPUI 框架和组件库
3. **日志系统初始化** (`init_runtime()`):
    - 日志目录: `~/.sqler/logs/`
    - 文件滚动: 每天轮转
    - 文件命名: `sqler.log`
    - 日志级别: debug (开发模式) / info (发布模式)
    - 双重输出: 终端 (带颜色) + 文件 (无颜色)
4. 配置全局主题 (滚动条悬停显示)
5. 创建主窗口 (1280x800，居中显示)
6. 挂载 `SqlerApp` 作为根视图
7. 配置窗口关闭行为 (所有窗口关闭时退出应用)

**FsAssets 实现**:

```
struct FsAssets;

impl AssetSource for FsAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let full = manifest_dir.join("assets").join(path);
        match fs::read(full) {
            Ok(data) => Ok(Some(Cow::Owned(data))),
            Err(_) => Ok(None),
        }
    }
}
```

**特点**: 从 crate 目录加载资源，支持开发和发布环境

---

### 2. app/ - 应用层（391 行）

**路径**: `crates/sqler-app/src/app/mod.rs`

**职责**: 核心 UI 逻辑、状态管理和用户交互

#### 核心结构

```
pub struct SqlerApp {
    pub tabs: IndexMap<String, TabContext>,             // 标签页集合
    pub active_tab: String,                              // 当前活动标签 ID
    pub cache: ArcCache,                                 // 缓存管理器
    pub windows: HashMap<String, WindowHandle<Root>>,   // 浮动窗口集合
}

pub struct TabContext {
    pub icon: SharedString,
    pub title: SharedString,
    pub content: TabContent,
    pub closable: bool,
}

pub enum TabContent {
    Home,
    Workspace(Workspace),
}

pub enum WindowKind {
    Create(Option<DataSource>),
    Import(DataSource),
    Export(DataSource),
}
```

#### 标签 ID 设计

- 使用 `IndexMap<String, TabContext>` 保持标签顺序
- Key 作为标签 ID：首页="home"，工作区=数据源UUID
- 消除了独立的 ID 字段，简化查找逻辑
- O(1) 时间复杂度的标签查找和删除

#### 核心方法

1. `new()`: 初始化应用，加载主题和缓存
2. `close_tab()`: 关闭标签，自动切换到前一个标签
3. `active_tab()`: 切换活动标签
4. `create_tab()`: 创建工作区标签（避免重复，使用 `cache.sources()` 查找数据源）
5. `toggle_theme()`: 切换亮色/暗色主题
6. `create_window()`: 创建浮动窗口（Create/Import/Export，HashMap 自动去重）
7. `close_window()`: 关闭指定窗口并从 HashMap 中移除

#### 数据源管理

- ✅ 通过 `cache.read().unwrap().sources()` 获取数据源列表（零成本借用）
- ✅ 首页渲染使用 `app.cache.read().unwrap().sources()`
- ✅ 创建标签使用 `app.cache.read().unwrap().sources()` 查找数据源
- ✅ 单一数据源原则（Arc<RwLock<AppCache>>），无数据重复
- ✅ 读写分离：读操作用 `read()`，写操作用 `write()`

#### UI 渲染

- 顶部标签栏 (支持切换和关闭)
- 主题切换按钮
- 新建数据源按钮
- 内容区域 (动态渲染首页或工作区)

---

### 3. comps/ - 公共组件（308 行）

**路径**: `crates/sqler-app/src/comps/`

#### 3.1 组件工具（mod.rs，92 行）

**提供功能**:

1. **元素 ID 拼接工具**:
   ```
   pub fn comp_id<I>(parts: I) -> ElementId
   ```
   示例: `comp_id(["tab", "mysql"])` → `"tab-mysql"`

2. **布局扩展 Trait** (`DivExt`):
    - `full()`: 设置为全尺寸 (size_full)
    - `col_full()`: 垂直方向充满 (flex-col + h_full)
    - `row_full()`: 水平方向充满 (flex-row + w_full)
    - `scrollbar_x()`: 水平滚动条
    - `scrollbar_y()`: 垂直滚动条
    - `scrollbar_all()`: 双向滚动条

**实现目标**: 为 `Div` 和 `Stateful<Div>` 提供统一的布局快捷方法

#### 3.2 图标组件（icon.rs，63 行）

**核心枚举**: `AppIcon`

**支持的图标**:
- MySQL, PostgreSQL, SQLite, Oracle, SQLServer, Redis, MongoDB
- Close, Export, Import, Reload, Search, Sheet, Transfer, Trash

**特点**:
- 统一的图标加载接口
- 自动从 `assets/icons/` 加载 SVG 文件

#### 3.3 数据表格组件（table.rs，153 行）

**核心结构**:

```
pub struct DataTable {
    cols: Vec<Column>,            // 列定义对象
    rows: Vec<Vec<SharedString>>, // 行数据
    loading: bool,                // 加载状态
}
```

**实现接口**: `gpui_component::table::TableDelegate`

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

**智能列宽计算**:
- 大写字母: 12px
- 小写/数字: 8px
- ASCII 符号: 6px
- 中文字符: 16px
- 最终宽度: `(max_width + 24).max(80).min(400)`

**动态列支持**:
- 通过 `update_data()` 更新数据
- 调用 `table.refresh(cx)` 重新准备列/行布局
- 支持从 0 列动态变更到任意列数

---

### 4. create/ - 数据源创建（1148 行）

**路径**: `crates/sqler-app/src/create/`

#### 4.1 创建窗口（mod.rs，483 行）

**核心结构**:

```
pub struct CreateWindow {
    cache: ArcCache,
    parent: WeakEntity<SqlerApp>,

    name: Entity<InputState>,
    kind: Option<DataSourceKind>,
    status: Option<DataSourceStatus>,
    source_id: Option<String>,

    // 各类型的创建表单实体
    mysql: Entity<MySQLCreate>,
    oracle: Entity<OracleCreate>,
    sqlite: Entity<SQLiteCreate>,
    sqlserver: Entity<SQLServerCreate>,
    postgres: Entity<PostgresCreate>,
    redis: Entity<RedisCreate>,
    mongodb: Entity<MongoDBCreate>,
}

pub struct CreateWindowBuilder {
    cache: Option<ArcCache>,
    source: Option<DataSource>,
    parent: Option<WeakEntity<SqlerApp>>,
}

pub enum DataSourceStatus {
    Testing,
    Error(String),
    Success(String),
}
```

**核心方法**:

1. `CreateWindowBuilder::new()`: 创建构建器
2. `builder.cache(cache)`: 设置缓存管理器
3. `builder.source(source)`: 设置编辑的数据源（可选）
4. `builder.parent(parent)`: 设置父应用引用
5. `builder.build(window, cx)`: 构建窗口
6. `check_conn()`: 异步测试连接，调用 `sqler_core::check_connection(&options)`
7. `create_conn()`: 保存数据源到缓存

**窗口配置**:

- 尺寸: 1280x720
- 位置: (0, 20) 固定左上角
- 类型: 浮动窗口
- 不可最小化

**功能流程**:

1. **类型选择页**: 展示所有支持的数据库类型（带图标和描述）
2. **表单页**: 根据选中类型动态切换对应的创建表单
3. **底部操作**:
    - 测试连接按钮：异步调用 `check_connection()`
    - 上一步按钮：返回类型选择页
    - 取消按钮：关闭窗口
    - 保存按钮：保存到 `cache.sources_mut()` 并加密写入

**保存流程**:

1. 构建 `DataSource::new(name, kind, options)` 或更新现有数据源
2. 获取写锁: `let mut cache = self.cache.write().unwrap()`
3. 保存数据源:
   - 新建: `cache.sources_mut().push(source)`
   - 编辑: 查找并替换现有数据源
4. 加密写入: `cache.sources_update()`
5. 成功后关闭窗口，失败显示错误

**当前状态**:

- ✅ UI 完整实现
- ✅ 表单字段收集
- ✅ 测试连接逻辑（后台线程调用 `check_connection()`）
- ✅ 保存到缓存逻辑（已实现并接入）
- ✅ 支持新建和编辑模式（通过 Builder 模式）
- ❌ Oracle / SQL Server 驱动未实现（保存时返回错误提示）

#### 4.2 表单实现

**支持的数据库类型** (每个独立模块):

| 模块             | 文件              | 数据库        | 行数  | 状态   |
|----------------|-----------------|------------|-----|------|
| `mysql.rs`     | mysql.rs        | MySQL      | 74  | ✅ 完整 |
| `postgres.rs`  | postgres.rs     | PostgreSQL | 80  | ✅ 完整 |
| `sqlite.rs`    | sqlite.rs       | SQLite     | 114 | ✅ 完整 |
| `oracle.rs`    | oracle.rs       | Oracle     | 64  | ✅ 完整 |
| `sqlserver.rs` | sqlserver.rs    | SQL Server | 71  | ✅ 完整 |
| `redis.rs`     | redis.rs        | Redis      | 174 | ✅ 完整 |
| `mongodb.rs`   | mongodb.rs      | MongoDB    | 88  | ✅ 完整 |

**表单特点**:

- 基于 `InputState` 组件构建
- 提供默认值和占位符
- 支持连接参数输入（主机、端口、用户名、密码等）
- 提供 `options(cx)` 方法构建对应的 Options 结构

---

### 5. workspace/ - 工作区（3273 行）

**路径**: `crates/sqler-app/src/workspace/`

#### 5.1 工作区路由（mod.rs，327 行）

**职责**: 根据数据源类型构造对应工作区视图

**Workspace 枚举**:

```
pub enum Workspace {
    Common { view: Entity<CommonWorkspace> },
    Redis { view: Entity<RedisWorkspace> },
    MongoDB { view: Entity<MongoDBWorkspace> },
}
```

**路由策略** (标准顺序):

```
match source.kind {
    DataSourceKind::MySQL
    | DataSourceKind::SQLite
    | DataSourceKind::Postgres
    | DataSourceKind::Oracle
    | DataSourceKind::SQLServer => {
        Workspace::Common { view }
    }
    DataSourceKind::Redis => {
        Workspace::Redis { view }
    }
    DataSourceKind::MongoDB => {
        Workspace::MongoDB { view }
    }
}
```

**数据源排序标准**:

```
MySQL → SQLite → Postgres → Oracle → SQLServer → Redis → MongoDB
```

**辅助功能**:

- `parse_elapsed(elapsed: f64)`: 格式化执行时间（< 1ms / Xms / X.XXs / Xm Xs）
- `parse_position(text, offset)`: 计算文本偏移对应的 LSP Position
- `EditorComps`: SQL 编辑器自动补全提供器（基于 keywords.json）

**首页渲染** (`render_home`):

- 4 列网格布局
- 卡片展示数据源（名称、图标、连接地址）
- 双击卡片打开对应工作区标签

#### 5.2 CommonWorkspace - 关系型数据库工作区（common.rs，1760 行）

**适用数据库**: MySQL, PostgreSQL, SQLite, Oracle, SQL Server

**核心结构**:

```
pub struct CommonWorkspace {
    pub cache: ArcCache,
    pub parent: WeakEntity<SqlerApp>,

    pub source: DataSource,
    pub session: Option<Box<dyn DatabaseSession>>,

    pub tabs: IndexMap<SharedString, TabContext>,
    pub active_tab: SharedString,
    pub tables: IndexMap<String, TableInfo>,
    pub active_table: Option<String>,
}

struct TabContext {
    title: SharedString,
    content: TabContent,
    closable: bool,
}

enum TabContent {
    Query(QueryContent),
    Table(TableContent),
    Schema(SchemaContent),
    Overview,
}

struct TableContent {
    id: SharedString,
    page: usize,
    count: usize,
    table: SharedString,
    columns: Vec<SharedString>,
    form_items: Vec<Entity<InputState>>,
    order_rules: Vec<OrderRule>,
    query_rules: Vec<QueryRule>,
    detail_panel: bool,
    detail_panel_idx: usize,
    detail_panel_state: Entity<ResizableState>,
    datatable: Entity<TableState<DataTable>>,
    _subscription: Subscription,
}

struct QueryContent {
    id: SharedString,
    active: usize,
    summary: bool,
    editor: Entity<InputState>,
    results: Vec<QueryResult>,
}

struct SchemaContent {
    id: SharedString,
    table: SharedString,
    columns: Vec<ColumnInfo>,
    datatable: Entity<TableState<DataTable>>,
}

struct QueryRule {
    id: String,
    value: Entity<InputState>,
    field: Entity<SelectState<Vec<SharedString>>>,
    operator: Entity<SelectState<Vec<SharedString>>>,
}

struct OrderRule {
    id: String,
    field: Entity<SelectState<Vec<SharedString>>>,
    order: Entity<SelectState<Vec<SharedString>>>,
}
```

**布局**: 左侧边栏（表列表）+ 右侧内容区（标签页）

**设计亮点**:

1. **IndexMap 优势**:
   - 保持标签和表的插入顺序
   - O(1) 时间复杂度的查找、插入、删除
   - Key 即 ID，无需额外存储

2. **标签 ID 生成规则**:
   - 概览标签: `"overview"`
   - 表数据标签: `"{table_name}_table"`
   - 表结构标签: `"{table_name}_schema"`
   - 查询标签: `"query_{uuid}"`

3. **表信息缓存**:
   - 使用 `IndexMap<String, TableInfo>` 存储完整表信息
   - 包含表名、行数、创建时间、注释等元数据
   - 自动同步到 `~/.sqler/cache/{uuid}/tables.json`

##### 连接管理

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

##### 标签页管理

**创建流程** (`create_table_tab`):

1. 生成标签 ID: `"{table_name}_table"`
2. 检查标签是否已存在（IndexMap 自动去重）
3. 创建空 `TableContent`（DataTable 用空数据初始化）
4. 插入到 `tabs: IndexMap<SharedString, TabContext>`
5. 设置为活动标签
6. 调用 `reload_table_tab` 加载实际数据

**设计优势**:

- IndexMap 的 Key 即标签 ID，无需独立存储
- 自动去重：重复打开同一表时，激活已有标签
- 避免代码重复（创建和刷新共用加载逻辑）
- 用户立即看到标签页（无需等待数据加载）

##### 数据加载（reload_table_tab）

**执行流程**:

**① 准备阶段**（主线程）:

1. 从 `table_content` 获取当前页码、表名、筛选/排序规则
2. 通过 `active_session()` 获取连接
3. 使用 `session.take()` 移动连接到闭包

**② 后台查询**（后台线程）:

1. 查询列信息: `session.columns(&table)` - 返回 `Vec<ColumnInfo>`
2. 构建查询请求: `QueryReq::Builder { table, columns, paging, orders, filters }`
3. 查询数据: `session.query(req)` - 返回 `QueryResp::Rows`
4. 转换数据为 `Vec<Vec<SharedString>>`

**③ UI 更新**（主线程）:

1. 归还连接: `this.session = Some(session)`
2. 解构查询结果为独立变量（避免所有权冲突）
3. 更新 `table_content` 的页码、总数、列名
4. 调用 `datatable.update()` 更新表格:
    - `delegate_mut().update_data(columns, rows)`: 更新数据
    - `refresh(cx)`: 重新准备列/行布局（**关键**！支持动态列）
    - `cx.notify()`: 触发重新渲染

**关键点**:

- `refresh(cx)` 必须调用，否则列结构不会更新
- 使用统一的 `columns()` trait 方法，消除数据库方言差异
- 页面大小固定为 **500 行/页** (常量 `PAGE_SIZE`)

##### 表格功能

**已实现功能**:

1. ✅ 分页导航（上一页/下一页，每页 500 行）
2. ✅ 显示当前页范围和总数
3. ✅ 筛选/排序规则 UI（添加/删除规则）
4. ✅ 筛选条件应用到查询（从 SelectState 读取选中值并构建 FilterCond）
5. ✅ 排序规则应用到查询（从 SelectState 读取选中值并构建 OrderCond）
6. ✅ 刷新表数据
7. ✅ 数据导出（打开传输窗口）
8. ✅ SQL 查询编辑器功能（多 SQL 支持、异步执行、错误处理、结果展示、执行时间统计）
9. ✅ 表结构查看功能（列信息表格、详情面板、列详细信息展示）

#### 5.3 RedisWorkspace - Redis 工作区（redis.rs，903 行）

**核心结构**:

```
pub struct RedisWorkspace {
    pub parent: WeakEntity<SqlerApp>,
    pub source: DataSource,
    pub session: Option<Box<dyn DatabaseSession>>,

    pub active: ViewType,
    pub browse: Option<BrowseContent>,
    pub command: Option<CommandContent>,
    pub overview: Option<OverviewContent>,
}

pub enum ViewType {
    Overview,
    Browse,
    Command,
}

struct BrowseContent {
    tree_state: Entity<TreeState>,
    keys: HashMap<String, KeyInfo>,
}

pub struct KeyInfo {
    pub key: SharedString,
    pub ttl: SharedString,
    pub kind: SharedString,   // string/list/hash/zset/set
    pub size: SharedString,
}
```

**布局**: 左侧视图切换 + 右侧内容区

**特点**:

- **键浏览器** (BrowseContent): 基于 `:` 分隔符构建层级树
- **命令执行** (CommandContent): 支持任意 Redis 命令
- **概览视图** (OverviewContent): 显示连接信息和统计数据
- 分页浏览: 每页 500 个键

**功能流程**:

1. **概览视图**: 显示 Redis 服务器信息、键数量、内存使用等
2. **键浏览器**:
   - 基于 `:` 分隔符构建层级树（如 `user:1:name` 显示为 `user > 1 > name`）
   - 显示键类型、TTL、大小
   - 支持键搜索和筛选
3. **命令执行**:
   - 输入任意 Redis 命令
   - 结果以表格形式展示
   - 支持命令历史

**TODO**:

- ❌ 实现命令执行逻辑（解析输入，调用 `session.query(QueryReq::Command {...})`）

#### 5.4 MongoDBWorkspace - MongoDB 工作区（mongodb.rs，283 行）

**核心结构**:

```
pub struct MongoDBWorkspace {
    meta: DataSource,
    parent: WeakEntity<SqlerApp>,
    session: Option<Box<dyn DatabaseSession>>,

    tabs: Vec<TabItem>,
    active_tab: SharedString,
    collections: Vec<SharedString>,
    active_collection: Option<SharedString>,
    sidebar_resize: Entity<ResizableState>,
}

enum TabContent {
    Collection(CollectionContent),
    Overview,
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

### 6. transfer/ - 数据传输（1003 行）

**路径**: `crates/sqler-app/src/transfer/`

**职责**: 数据导入/导出功能

**模块组成** (mod.rs, 43 行):

- `ImportWindow`: 数据导入窗口
- `ExportWindow`: 数据导出窗口
- `TransferKind` 枚举: CSV / JSON / SQL

#### 6.1 导入窗口（import.rs，711 行）

**核心结构**:

```
pub struct ImportWindow {
    meta: DataSource,
    parent: WeakEntity<SqlerApp>,

    step: ImportStep,
    files: Vec<ImportFile>,
    tables: Vec<SharedString>,

    // CSV 参数配置
    col_index: Entity<InputState>,
    data_index: Entity<InputState>,
    row_delimiter: Entity<InputState>,
    col_delimiter: Entity<InputState>,

    file_kinds: Entity<DropdownState<Vec<SharedString>>>,
    import_modes: Entity<DropdownState<Vec<SharedString>>>,
}
```

**导入步骤** (`ImportStep` 枚举):

1. **Kind**: 文件类型与参数配置
2. **Files**: 选择待导入文件
3. **Table**: 配置源文件与目标表映射
4. **Import**: 选择导入模式并执行

**导入模式** (`ImportMode` 枚举):

- Replace: 替换 - 清空表后导入新数据
- Append: 追加 - 在表末尾追加新数据
- Update: 更新 - 更新已存在的数据
- AppendOrUpdate: 追加或更新 - 存在则更新，不存在则追加
- AppendNoUpdate: 追加不更新 - 仅追加不存在的数据

**ImportFile 结构**:

```
struct ImportFile {
    path: PathBuf,
    option: TableOption,
    new_table: Entity<InputState>,
    exist_table: Entity<DropdownState<Vec<SharedString>>>,
}
```

**窗口配置**:

- 尺寸: 1280x720
- 位置: (0, 20) 固定左上角
- 类型: 浮动窗口
- 标题: "数据导入"

**核心功能**:

1. ✅ 步骤式导入流程 UI
2. ✅ 文件选择器集成 (`prompt_for_paths`)
3. ✅ CSV 参数配置（字段行、分隔符等）
4. ✅ 文件与目标表映射（支持新建表/选择已存在表）
5. ✅ 导入模式选择
6. ❌ 实际导入逻辑待实现

#### 6.2 导出窗口（export.rs，249 行）

**核心结构**:

```
pub struct ExportWindow {
    parent: WeakEntity<SqlerApp>,
    format: Option<TransferKind>,
    file_path: Entity<InputState>,
    table_name: Entity<InputState>,
}
```

**窗口配置**:

- 尺寸: 1280x720
- 位置: (0, 20) 固定左上角
- 类型: 浮动窗口
- 标题: "数据导出"

**核心功能**:

1. ✅ 格式选择 UI（CSV / JSON / SQL，卡片式选择）
2. ✅ 源表名称输入
3. ✅ 目标文件路径输入
4. ❌ 实际导出逻辑待实现

#### 6.3 TransferKind 枚举

**支持的格式**:

- CSV: 逗号分隔值文件，适用于表格数据
- JSON: JSON 格式文件，适用于结构化数据
- SQL: SQL 脚本文件，包含完整的建表和插入语句

**方法**:

- `all()`: 返回所有格式
- `label()`: 返回格式标签
- `description()`: 返回格式描述
- `from_label(label)`: 从标签解析格式

---

### 7. subtask/ - 预留模块（1 行）

**路径**: `crates/sqler-app/src/subtask/mod.rs`

**状态**: 空模块，仅包含两个空行

**用途**: 为未来的子任务系统预留模块位置

---

### 8. assets/ - 静态资源

**路径**: `crates/sqler-app/assets/`

**内容**: 数据库图标等静态文件

**图标列表**:

- `icons/mysql.svg`
- `icons/postgresql.svg`
- `icons/sqlite.svg`
- `icons/oracle.svg`
- `icons/sqlserver.svg`
- `icons/redis.svg`
- `icons/mongodb.svg`
- `icons/home.svg`
- `icons/close.svg`, `icons/export.svg`, `icons/import.svg`, ...

**加载**: 通过 `FsAssets` 从 `CARGO_MANIFEST_DIR/assets/` 注册到 GPUI

---

## 功能现状

### 已实现功能 ✅

#### 主窗口

1. ✅ 顶部标签栏（支持多标签切换）
2. ✅ 主题切换按钮（亮色/暗色）
3. ✅ 新建数据源浮动窗口
4. ✅ 日志系统（终端+文件双重输出，每日轮转）

#### 首页

1. ✅ 网格卡片展示数据源
2. ✅ 双击打开工作区标签
3. ✅ 显示数据源图标和连接地址

#### 关系型数据库工作区 (CommonWorkspace)

1. ✅ 左侧表列表导航
2. ✅ 动态标签页管理
3. ✅ 分页查询（上一页/下一页，每页 500 行）
4. ✅ 数据表格展示（支持动态列）
5. ✅ 筛选/排序规则 UI（添加/删除规则）
6. ✅ 筛选条件应用到查询（构建 FilterCond 并传递给 QueryReq::Builder）
7. ✅ 排序规则应用到查询（构建 OrderCond 并传递给 QueryReq::Builder）
8. ✅ 连接复用机制
9. ✅ 刷新表数据
10. ✅ 统一的 `columns()` 方法（消除 SQL 方言差异）
11. ✅ SQL 查询编辑器完整实现（多 SQL、异步执行、错误处理、结果展示、执行时间统计）
12. ✅ 表结构查看完整实现（列信息表格、详情面板、字段详细信息）

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
6. ✅ `supp_kinds()` 方法返回支持的列类型

#### 新建数据源窗口

1. ✅ 数据库类型选择
2. ✅ 7 种数据库的表单实现
3. ✅ 测试连接功能（异步调用 `check_connection()`）
4. ✅ 保存到缓存（已实现并接入）
5. ✅ 状态提示（测试中/成功/失败）

#### 数据导入窗口

1. ✅ 步骤式导入流程 UI（4 步骤）
2. ✅ 文件选择器集成
3. ✅ CSV 参数配置（字段行、分隔符等）
4. ✅ 文件与目标表映射 UI
5. ✅ 支持新建表/选择已存在表
6. ✅ 导入模式选择（5 种模式）
7. ❌ 实际导入逻辑待实现

#### 数据导出窗口

1. ✅ 格式选择 UI（CSV / JSON / SQL）
2. ✅ 源表名称输入
3. ✅ 目标文件路径输入
4. ❌ 文件保存对话框集成
5. ❌ 实际导出逻辑待实现

#### 缓存系统

1. ✅ AES-256-GCM 加密（sources.db）
2. ✅ JSON 存储（tables.json, queries.json）
3. ✅ 目录结构：`~/.sqler/cache/{uuid}/`
4. ✅ 单一数据源原则（消除数据重复）
5. ✅ 零成本抽象（返回引用避免克隆）
6. ✅ 懒加载（按需创建目录）
7. ✅ 数据源管理已接入 UI
8. ✅ 表信息缓存已接入（CommonWorkspace 初始化时读取、reload_tables 时更新）
9. ❌ 查询缓存暂未使用

---

### 待完成功能 ❌

#### 高优先级

1. **数据导入/导出执行逻辑**
    - 实现 CSV/JSON/SQL 解析器
    - 实现批量数据插入
    - 实现进度跟踪和错误处理
    - 集成文件保存对话框

2. **查询缓存使用**
    - 实现保存查询功能，使用 `queries.json`
    - 实现查询历史和收藏功能

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

1. **数据编辑**
    - 单元格编辑
    - 行增删
    - 保存变更到数据库

2. **错误处理优化**
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

### 1. Workspace 架构优势

- **职责分离**: 核心逻辑（sqler-core）与 UI 层（sqler-app）完全分离
- **单向依赖**: sqler-app 依赖 sqler-core，避免循环依赖
- **独立编译**: sqler-core 可单独编译和测试
- **代码复用**: sqler-core 可被其他项目复用（CLI、Web 服务等）
- **统一版本管理**: workspace.dependencies 集中管理依赖版本

### 2. Trait 驱动架构

- `DatabaseDriver` 和 `DatabaseSession` 统一多数据库接口
- 通过 `columns()` trait 方法消除 SQL 方言差异
- 支持 SQL、文档、命令三种查询模式

### 3. 连接复用策略

- 使用 `Option<Box<dyn DatabaseSession>>` 实现懒加载和复用
- 支持跨线程移动（DatabaseSession: Send）
- 失败自动重试

### 4. 参数化查询

- 各驱动正确处理占位符和参数绑定
- 防止 SQL 注入攻击
- 标识符转义（反引号、双引号）

### 5. 工作区架构

- `Workspace` 枚举支持多种工作区类型
- Common 工作区统一处理所有关系型数据库
- 专用工作区（Redis、MongoDB）针对性优化

### 6. 数据源 ID 设计

- 使用 UUID 作为标签 ID
- 避免 TabId 包装类型
- 简化查找和路由逻辑

### 7. 窗口管理设计

**HashMap 去重机制**:

- 使用 `HashMap<String, WindowHandle<Root>>` 管理浮动窗口
- 同一类型窗口只能打开一个
- 窗口关闭时自动从 HashMap 中移除

**窗口键命名规则**:

- 新建数据源窗口: `"create"`
- 数据导入窗口: `"import-{uuid}"`
- 数据导出窗口: `"export-{uuid}"`

### 8. 缓存系统设计

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
- ⚠️ 密钥和 Nonce 硬编码，生产环境应从环境变量读取

**懒加载**:

- 按需创建 `cache/{uuid}/` 目录
- 文件不存在返回空列表，不阻塞系统

### 9. 动态列支持

- DataTable 通过 `update_data()` 和 `refresh()` 支持动态列数
- 无需重建表格组件

### 10. 数据源排序标准

- 统一排序：MySQL → SQLite → Postgres → Oracle → SQLServer → Redis → MongoDB
- 所有 match 语句遵循相同顺序
- 提高代码一致性和可维护性

---

## 代码统计

### Workspace 总览

| 分类             | 文件数 | 代码行数      | 占比    |
|----------------|-----|-----------|-------|
| **sqler-core** | 10  | 3,145     | 33.5% |
| **sqler-app**  | 21  | 6,248     | 66.5% |
| **总计**         | 31  | **9,393** | 100%  |

### sqler-core crate 统计（3145 行）

| 模块          | 文件数 | 代码行数  | 占比    |
|-------------|-----|-------|-------|
| lib.rs      | 1   | 734   | 23.3% |
| driver/     | 8   | 2,240 | 71.2% |
| cache/      | 1   | 171   | 5.4%  |
| **总计**      | 10  | 3,145 | 100%  |

**driver/ 模块细分**:

| 文件           | 行数    | 状态   |
|--------------|-------|------|
| postgres.rs  | 473   | 全功能  |
| mysql.rs     | 405   | 全功能  |
| sqlite.rs    | 385   | 全功能  |
| redis.rs     | 320   | 全功能  |
| mongodb.rs   | 291   | 全功能  |
| mod.rs       | 288   | 接口定义 |
| sqlserver.rs | 77    | 占位实现 |
| oracle.rs    | 1     | 仅注释  |
| **总计**       | 2,240 |      |

### sqler-app crate 统计（6248 行）

| 模块          | 文件数 | 代码行数  | 占比    |
|-------------|-----|-------|-------|
| workspace/  | 4   | 3,273 | 52.4% |
| create/     | 8   | 1,148 | 18.4% |
| transfer/   | 3   | 1,003 | 16.1% |
| app/        | 1   | 391   | 6.3%  |
| comps/      | 3   | 308   | 4.9%  |
| main.rs     | 1   | 124   | 2.0%  |
| subtask/    | 1   | 1     | 0.0%  |
| **总计**      | 21  | 6,248 | 100%  |

**workspace/ 模块细分**:

| 文件         | 行数    | 说明          |
|------------|-------|-------------|
| common.rs  | 1,760 | 关系型数据库工作区   |
| redis.rs   | 903   | Redis 工作区   |
| mod.rs     | 327   | 工作区路由       |
| mongodb.rs | 283   | MongoDB 工作区 |
| **总计**     | 3,273 |             |

**create/ 模块细分**:

| 文件           | 行数    | 说明            |
|--------------|-------|---------------|
| mod.rs       | 483   | 创建窗口主逻辑       |
| redis.rs     | 174   | Redis 表单      |
| sqlite.rs    | 114   | SQLite 表单     |
| mongodb.rs   | 88    | MongoDB 表单    |
| postgres.rs  | 80    | PostgreSQL 表单 |
| mysql.rs     | 74    | MySQL 表单      |
| sqlserver.rs | 71    | SQL Server 表单 |
| oracle.rs    | 64    | Oracle 表单     |
| **总计**       | 1,148 |               |

**transfer/ 模块细分**:

| 文件        | 行数    | 说明     |
|-----------|-------|--------|
| import.rs | 711   | 数据导入窗口 |
| export.rs | 249   | 数据导出窗口 |
| mod.rs    | 43    | 传输类型定义 |
| **总计**    | 1,003 |        |

**comps/ 模块细分**:

| 文件          | 行数  | 说明       |
|-------------|-----|----------|
| table.rs    | 153 | 数据表格组件   |
| mod.rs      | 92  | 组件工具     |
| icon.rs     | 63  | 图标组件     |
| **总计**      | 308 |          |

---

## 项目配置

### Workspace 配置（根 Cargo.toml）

```
[workspace]
members = [
    "crates/sqler-core",
    "crates/sqler-app",
]
resolver = "2"

[workspace.package]
version = "1.0.0"
edition = "2024"

[workspace.dependencies]
sqler-core = { path = "crates/sqler-core" }

gpui = { version = "0.2.2" }
gpui-component = { version = "0.5.0", features = ["tree-sitter-languages"] }

dirs = "6"
uuid = "1"
aes-gcm = "0.10"
indexmap = "2"
lsp-types = "0.97.0"
thiserror = "2"

serde = { version = "1", features = ["derive"] }
serde_json = "1"

tracing = "0.1"
tracing-appender = "0.2"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

mongodb = { version = "3", features = ["sync"] }
mysql = { version = "26" }
postgres = { version = "0.19" }
redis = { version = "1.0.1", features = ["cluster"] }
rusqlite = { version = "0.37", features = ["bundled"] }
```

### sqler-core 配置

```
[package]
name = "sqler-core"
version.workspace = true
edition.workspace = true

[lib]
name = "sqler_core"
path = "src/lib.rs"

[dependencies]
dirs.workspace = true
uuid.workspace = true
aes-gcm.workspace = true
thiserror.workspace = true

serde.workspace = true
serde_json.workspace = true

tracing.workspace = true

mongodb.workspace = true
mysql.workspace = true
postgres.workspace = true
redis.workspace = true
rusqlite.workspace = true
```

### sqler-app 配置

```
[package]
name = "sqler-app"
version.workspace = true
edition.workspace = true

[[bin]]
name = "sqler"
path = "src/main.rs"

[dependencies]
sqler-core.workspace = true

gpui.workspace = true
gpui-component.workspace = true

dirs.workspace = true
uuid.workspace = true
indexmap.workspace = true
lsp-types.workspace = true

serde.workspace = true
serde_json.workspace = true

tracing.workspace = true
tracing-appender.workspace = true
tracing-subscriber.workspace = true
```

### 发布配置

```
[profile.release]
lto = "thin"
strip = true
debug = false
panic = 'abort'
opt-level = 3
codegen-units = 1
```

---

## 贡献指南

### 代码规范

1. **导入顺序**:
   ```
   // 1. 标准库导入
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

2. **数据源排序**: 所有涉及 `DataSourceKind` 的 match 语句必须遵循标准顺序：
   MySQL → SQLite → Postgres → Oracle → SQLServer → Redis → MongoDB

3. **错误处理**: 优先使用 `Result<T, DriverError>`，避免 panic

4. **命名约定**:
    - 结构体：大驼峰 (PascalCase)
    - 函数/变量：蛇形 (snake_case)
    - 常量：全大写蛇形 (UPPER_SNAKE_CASE)

### 测试

- 在 `docs/testdata/` 目录下提供测试数据脚本
- 每个数据库至少 10 张表，每表≥1000 行数据
- 覆盖常见数据类型和关系

---

## 版本历史

**v5.0 (2025-12-20) - Workspace 架构重组**:

- 🎯 **架构升级**: 重构为 Cargo Workspace 模式
- 📦 **Crate 分离**:
  - 新增 `sqler-core` crate（3145行）：核心库，包含 driver、cache、model
  - 新增 `sqler-app` crate（6248行）：应用程序，包含 UI 层
- 🔄 **模块迁移**:
  - `src/driver/` → `crates/sqler-core/src/driver/`
  - `src/cache/` → `crates/sqler-core/src/cache/`
  - `src/model.rs` → `crates/sqler-core/src/lib.rs` (合并)
  - `src/app/` → `crates/sqler-app/src/app/`
  - `assets/` → `crates/sqler-app/assets/`
- 📊 **代码统计**: 9,403 → 9,393 行 (-10行，代码优化)
- 🏗️ **依赖管理**: workspace.dependencies 统一版本管理
- 📝 **文档更新**: CODEMAP.md 完全重写为 Workspace 架构

**模块拆分详情**:
- sqler-core 导出接口: DatabaseDriver, DatabaseSession, AppCache, ArcCache 等
- sqler-app 依赖 sqler-core，单向依赖无循环
- 编译产物: 单个二进制 `sqler`（位于 sqler-app）

**v4 (2025-12-16)**:

- 更新代码总行数: 8,699 → 9,403 (+704 行，+8.1%)
- 更新文件总数: 31 → 32 个文件 (+1)
- **workspace/redis.rs**: 371 → 905 行 (+534 行，+144%) - 最大增长
    - 新增 BrowseContent (键浏览器)
    - 新增 CommandContent (命令执行)
    - 新增 OverviewContent (概览)
    - 基于 `:` 分隔符的键树构建
- **workspace/common.rs**: 1,655 → 1,763 行 (+108 行，+6.5%)
    - 新增 QueryContent (SQL 查询标签)
    - 新增 SchemaContent (表结构标签)
    - 增强 TableContent (表数据标签)
- **comps/** 模块扩展: 201 → 308 行 (+107 行)
    - 新增 `icon.rs` (63 行) - AppIcon 枚举
    - `table.rs`: 122 → 153 行 (+31 行) - 智能列宽计算
    - `mod.rs`: 79 → 92 行 (+13 行)
- **create/** 模块增长: 1,092 → 1,149 行 (+57 行)
- **driver/** 模块优化: 2,392 → 2,240 行 (-152 行，-6.4%)
- **model.rs**: 668 → 724 行 (+56 行)

**v3 (2025-12-03)**:

- 更新代码总行数: 8,262 → 8,699 (+437 行)
- `common.rs`: 1,318 → 1,655 (+337 行) - 最大增长
- `mongodb.rs`: 501 → 285 (-216 行) - 代码优化重构
- 新增 codegen 和 update 模块占位符

**v2 (2025-01-24)**:

- 更新代码总行数: 7,752 → 8,262 (+510 行)
- `common.rs`: 1,058 → 1,318 (+260 行)
- TabContent 结构扩展 (新增 Query 和 Schema)
- 窗口管理改为 HashMap 方式
- 页面大小调整为 500 行/页
- 新增 SQL 关键字文档 (keywords.json, 762 行)

**v1 (2025-01-17)**:

- 初始版本，基于实际代码详细记录所有文件行数和实现细节

**最后更新**: 2025-12-20 (Workspace 架构重组)
