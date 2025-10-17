## 项目快照
- 名称：`sqler`
- 技术栈：Rust + gpui + gpui-component
- 目标：提供多标签页数据库管理 UI（浏览数据源、新建连接、后续扩展查询）

## 核心结构
- `src/main.rs`
    - `SqlerApp`：窗口根视图，维护顶层标签数据（`TabState`）及保存的数据源列表
    - `TabState`：描述单个页签（首页、已有数据源、新建数据源），持有内部状态
    - `NewDataSourceState` / `DataSourceTabState`：分别负责“新建数据源”表单状态与现有数据源视图（包含右侧子标签、侧边栏表列表）
    - `ConnectionForm`：封装 gpui 输入组件实体，提供连接信息字段（名称、主机、端口、账户、Schema 等）
    - UI 渲染函数：`render_tab_bar`、`render_home`、`render_workspace_toolbar`、`render_inner_tab_bar`、`render_connection_form`、`render_data_source_detail`
- `assets/`：图标等静态资源
- `Cargo.toml`：依赖 `gpui` 与 `gpui-component`

## 当前状态
- 顶部浏览器式标签栏可在“首页 / 数据源 / 新建”之间切换，支持关闭二级页签
- 首页展示预置数据源，双击可打开详情标签
- “新建数据源”标签提供类型选择 + 连接信息表单，右侧布局包含内层固定“配置”页签
- 左侧边栏根据上下文显示表列表或占位说明；所有主要布局使用 gpui-component 样式（按钮、表单、滚动容器等）
