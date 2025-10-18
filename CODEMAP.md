## 项目快照
- 名称：`sqler`
- 技术栈：Rust + gpui + gpui-component
- 目标：提供多标签页数据库管理 UI（浏览数据源、新建连接、后续扩展查询）
## 核心结构
- `src/main.rs`
    - 应用入口：注册资源加载器、初始化通用运行时组件（数据库驱动、缓存等占位）
- `src/views/mod.rs`
    - 定义 UI 状态：`SqlerApp`、`TabState`、`DataSourceTabState`、`NewDataSourceState` 等
    - `NewDataSourceState` 记录当前选中的数据库类型及对应表单状态
    - `TabKind` 仅保留首页 / 数据源两种标签；`SqlerApp::render` 调用 `workspace::render_root`
- `src/comps/mod.rs`：提供基础布局构造 `page()`，生成满屏纵向容器
- `src/driver/`
    - `mod.rs`：统一驱动接口、错误类型与 `test_connection` 入口；集成 SQLx / Tiberius
    - `{postgres,mysql,sqlite,sqlserver}.rs`：针对不同数据源实现具体驱动配置与连接测试
- `src/views/`
    - `workspace/mod.rs`：组合整体布局（顶栏、主体），并调度各类型数据源工作区
    - `workspace/{postgres,mysql,sqlite,sqlserver}.rs`：针对不同数据库类型渲染专属工作区视图
    - `topbar.rs`：渲染顶栏（标签页 + “新建数据源”按钮 + 主题切换）
    - `create/mod.rs`：独立窗口 `CreateDataSourceWindow`，负责新建数据源流程（类型选择 + 表单）
    - `create/{postgres,mysql,sqlite,sqlserver}.rs`：按数据库类型划分的创建表单
- `assets/`：图标等静态资源
- `Cargo.toml`：依赖 `gpui` 与 `gpui-component`

## 当前状态
- 顶部浏览器式标签栏可在“首页 / 数据源”之间切换，支持关闭二级页签
- 顶部标签容器固定宽度，若空间不足会等比例压缩；右侧提供“新建数据源”窗口入口与主题切换
- 可关闭标签的按钮使用 `assets/icons/close.svg` 图标（通过 `FsAssets` 提供文件系统资源加载）
- 首页展示预置数据源，双击可打开详情标签
- “新建数据源”改为独立窗口：顶部标题 + 中间内容区域（类型选择 / 表单滚动区）+ 底部操作栏
- 底部操作栏集成“测试连接 / 上一步 / 取消 / 保存”，并在未选择类型时自动禁用相关按钮
- 驱动层提供 Postgres / MySQL / SQLite / SQL Server 实际连接测试实现，可配合 UI 按钮调用
- 左侧边栏根据上下文显示表列表或占位说明；所有主要布局使用 gpui-component 样式（按钮、表单、滚动容器等）
- 标签页主内容启用 `min-height: 0` 处理，滚动区域恢复可用且顶部对齐；顶部标签栏新增主题切换按钮
