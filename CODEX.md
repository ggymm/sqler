## 根目录
- `README.md`：描述跨平台数据库管理器的目标、布局和两轮优化计划。
- `AGENTS.md` / `CLAUDE.md`：AI 代理说明文档，保留作为参考。
- `CODEX.md`：仓库拓扑与设计记录（当前文件）。

## 资源
- `assets/icons/`：UI 所需的 SVG 资源。
  - `assets/icons/db/` 收纳各数据库类型的图标（MySQL、PostgreSQL、Redis 等）。
  - 其他文件（`add.svg`, `new-conn.svg`, `theme-dark.svg`, ...）用于按钮、主题切换等控件。
- `assets/styles/app.css`：全局样式与亮/暗主题配色。

## 源码结构
```
src/
├── main.rs             # 程序入口，仅负责启动 App
├── app/                # 应用逻辑与顶层状态
│   ├── mod.rs          # App 组件、信号管理、关窗逻辑
│   ├── logic.rs        # 数据源加载/保存、表单校验、测试占位
│   └── theme.rs        # 主题枚举与工具函数
├── models/             # 数据模型
│   ├── mod.rs
│   ├── datasource.rs   # DataSource/DbKind/ConnectionForm 定义 + 演示数据
│   └── tabs.rs         # Tab/TabKind/DataSourceTabState + 创建弹窗状态机
└── views/              # 视图组件
    ├── mod.rs
    ├── tab_bar.rs
    ├── home.rs
    ├── data_source.rs
    ├── create_modal.rs
    ├── confirm.rs
    ├── forms.rs
    └── icons.rs
```

### 关键数据模型
- `DataSource`：保存连接信息，不持有实际连接；配套 `ConnectionForm` 负责表单态。
- `DbKind`：数据库类型枚举，提供标签与默认端口。
- `Tab` / `TabKind`：首页与数据源详情两类标签页，记录 `dirty` 状态用于关闭确认。
- `DataSourceTabState`：详情页内部状态（选中侧边项、表单快照、原始数据源）。
- `CreateModalState`：三步弹窗（类型选择 → 表单填写 → 测试&保存）的状态机，包含测试结果、错误与保存进度。

### 主 UI（`src/main.rs`）要点
- 顶层 `App` 使用 `Signal` 管理数据源列表、标签页数组、当前激活标签和待关闭确认。
- `TabBar` 自绘浏览器式标签条，支持激活/关闭、主题切换与“新建数据源”触发。
- `HomeTab` 展示演示数据源卡片（含图标），双击打开详情，无额外的新建按钮。
- `DataSourceView` 按“顶部菜单 + 左侧导航 + 右侧内容”布局渲染，概览页可编辑连接表单，其他页占位描述不同数据库任务。
- `CreateSourceModal` 通过三步弹窗完成类型选择、表单填写、连接测试与保存；成功后写入本地文件并自动打开详情标签。
- `ConfirmCloseModal` 在有未保存修改时阻止直接关闭标签。

### 当前进度
- ✅ 建立多标签页基础布局、关闭确认、亮/暗主题切换。
- ✅ 使用 `Signal` 驱动数据源展示与详情编辑流程。
- ✅ 新建数据源走弹窗流程，带假连接测试，成功后写入 `~/.sqler/datasources.json` 并打开新标签。
- ⏳ 连接测试仍为同步校验 + 延迟模拟，后续需接入真实驱动。

### 下一步建议
1. 在 `ui/` 下进一步拆分公共控件（如占位、表格渲染），保持模块清晰。
2. 将连接测试改为真正的异步驱动调用，并按数据库类型给出明确错误反馈。
3. 在详情页合入新增数据源（无需重新加载），补齐保存/加载的异常提示。
4. 根据 README 的第二阶段要求，丰富不同数据库类型的右侧内容面板。
