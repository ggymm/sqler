## 项目快照
- 名称：`sqler`
- 技术栈：Rust + gpui + gpui-component
- 目标：提供多标签页数据库管理 UI（浏览数据源、新建连接、后续扩展查询）

## 核心结构
- `src/main.rs`
    - `SqlerApp`：窗口根视图，维护顶层标签数据（`TabState`）及保存的数据源列表
        - `toggle_theme` 通过 `Theme::change` 切换深浅主题
        - `impl Render` 仅委托给 `views::main::render`
    - `FsAssets`：实现 `AssetSource`，从项目 `assets/` 目录加载 SVG 图标等静态资源
    - `TabState`：描述单个页签（首页、已有数据源、新建数据源），持有内部状态
    - `NewDataSourceState` / `DataSourceTabState`：分别负责“新建数据源”表单状态与现有数据源视图（包含右侧子标签、侧边栏表列表）
    - `ConnectionForm`：封装 gpui 输入组件实体，提供连接信息字段（名称、主机、端口、账户、Schema 等）
- `src/comps/mod.rs`：提供基础布局构造 `page()`，生成满屏纵向容器
- `src/views/`
    - `main.rs`：组合页面骨架，串联 header 与 content
    - `header.rs`：渲染顶栏（标签页滚动区域 + 主题切换按钮）
    - `content.rs`：根据 `TabKind` 渲染首页、新建数据源、数据源详情
- `assets/`：图标等静态资源
- `Cargo.toml`：依赖 `gpui` 与 `gpui-component`

## 当前状态
- 顶部浏览器式标签栏可在“首页 / 数据源 / 新建”之间切换，支持关闭二级页签
- 顶部标签容器固定宽度并启用横向滚动，避免压缩右侧主题切换按钮
- 可关闭标签的按钮使用 `assets/icons/close.svg` 图标（通过 `FsAssets` 提供文件系统资源加载）
- 首页展示预置数据源，双击可打开详情标签
- “新建数据源”标签提供类型选择 + 连接信息表单，右侧布局包含内层固定“配置”页签
- 左侧边栏根据上下文显示表列表或占位说明；所有主要布局使用 gpui-component 样式（按钮、表单、滚动容器等）
- 标签页主内容启用 `min-height: 0` 处理，滚动区域恢复可用且顶部对齐；顶部标签栏新增主题切换按钮
