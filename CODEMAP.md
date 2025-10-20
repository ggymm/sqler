## 项目概览
- 名称：`sqler`
- 技术栈：Rust + gpui/gpui-component + SQLx/Tiberius
- 目标：桌面化多标签数据库管理器，可浏览数据源、维护连接并扩展查询能力

## 代码结构
- `src/main.rs`
  - 程序入口，注册资源加载器 `FsAssets`，初始化运行时后打开主窗口，挂载 `SqlerApp`
- `src/app/`
  - `mod.rs`：核心应用状态
    - 声明 `SqlerApp`、`TabState` 等 UI 状态及 `NewDataSourceState`
    - 管理窗口生命周期（创建/聚焦“新建数据源”窗口），驱动主题切换、标签增删
    - 主渲染函数将工作区容器设为 `flex_1`，保证底部布局获取满高
    - 数据源元信息 `DataSourceMeta` 使用 `option::StoredOptions` 保存连接配置，供工作区与驱动共享
  - `workspace/`
    - `mod.rs`：主窗口渲染逻辑，组合顶栏与工作区，首页/数据源均固定顶部，左右布局撑满高度并各自滚动
    - `{postgres,mysql,sqlite,sqlserver}.rs`：每种数据库的工作区视图（展示配置、占位说明）
  - `create/`
    - `mod.rs`：`CreateDataSourceWindow`，提供类型选择、表单填充和底部操作栏
    - `{postgres,mysql,sqlite,sqlserver}.rs`：各数据库表单状态与渲染
  - `topbar.rs`：标签栏 UI，控制标签切换/关闭、触发新建窗口与主题切换
  - `comps/mod.rs`：通用页面布局（全屏纵向容器等）
- `src/driver/`
  - `mod.rs`：统一驱动接口 `DatabaseDriver`、`DriverError`，以及 `test_connection` 入口
  - `postgres.rs`：基于 `sqlx::postgres` 的连接校验（支持 SSL 模式）
  - `mysql.rs`：使用 `sqlx::mysql` 建立连接，支持自定义字符集与 TLS
  - `sqlite.rs`：通过 `sqlx::sqlite` 检查本地文件并尝试连接，兼容只读模式
  - `sqlserver.rs`：借助 `tiberius` + `tokio-native-tls` 完成 SQL Server 握手，支持 SQL 密码登录
- `src/option/`
  - `mod.rs`：定义 `ConnectionOptions` 接口并聚合各数据库选项
  - `{mysql,postgres,sqlite,sqlserver,oracle,redis,mongodb}.rs`：描述对应数据源的连接参数与默认值
- `src/cache/`、`src/export/`：当前为空，为后续缓存/导出功能预留入口
- `assets/`：静态资源（图标等）
- `Cargo.toml`
  - UI 依赖（gpui/gpui-component）+ 数据库驱动依赖（SQLx/Tiberius/ Tokio 等）

## 功能现状
- 主窗口：顶部标签栏（首页 / 数据源标签，可关闭 data 标签）、内容区与顶栏操作
- 首页：展示预置数据源卡片，双击可打开详情标签
- “新建数据源”窗口：
  - 顶部标题固定；中部根据状态展示类型卡片或对应表单（支持滚动）
  - 底部操作栏提供“测试连接 / 上一步 / 取消 / 保存”，未选类型时自动禁用相关按钮
- 各数据库表单：按字段分类显示输入框，记忆当前配置状态
- 驱动层：接入 PostgreSQL / MySQL / SQLite / SQL Server 的真实连接测试逻辑，为后续 UI 调用奠定基础
