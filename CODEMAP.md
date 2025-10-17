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
    - `TabKind` 仅保留首页 / 数据源两种标签；`SqlerApp::render` 调用 `content::render_root`
- `src/comps/mod.rs`：提供基础布局构造 `page()`，生成满屏纵向容器
- `src/views/`
    - `content.rs`：组合整体布局（含顶栏、主体、弹窗），并渲染各标签内容
    - `topbar.rs`：渲染顶栏（标签页 + “新建数据源”按钮 + 主题切换）
    - `create/mod.rs`：自定义“新建数据源”弹窗（类型选择 + 表单）
    - `create/{postgres,mysql,sqlite,sqlserver}.rs`：按数据库类型划分的创建表单
- `assets/`：图标等静态资源
- `Cargo.toml`：依赖 `gpui` 与 `gpui-component`

## 当前状态
- 顶部浏览器式标签栏可在“首页 / 数据源”之间切换，支持关闭二级页签
- 顶部标签容器固定宽度，若空间不足将等比例压缩；右侧提供“新建数据源”弹窗入口与主题切换
- 可关闭标签的按钮使用 `assets/icons/close.svg` 图标（通过 `FsAssets` 提供文件系统资源加载）
- 首页展示预置数据源，双击可打开详情标签
- “新建数据源”改为弹窗流程：先选择数据库类型，再填写对应表单；后续保存逻辑待接入
- 左侧边栏根据上下文显示表列表或占位说明；所有主要布局使用 gpui-component 样式（按钮、表单、滚动容器等）
- 标签页主内容启用 `min-height: 0` 处理，滚动区域恢复可用且顶部对齐；顶部标签栏新增主题切换按钮
