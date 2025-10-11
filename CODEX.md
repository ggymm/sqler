# CODE MAP

## App Shell
- `src/app/mod.rs`: 根应用状态机，负责连接管理、任务调度、消息路由；`Palette`/`ThemeMode` 定义主题色板。
- `src/app/content.rs`: 顶层内容容器，将 `App` 状态映射到各内容标签页。
- `src/app/sidebar.rs`, `src/app/topbar.rs`, `src/app/dialog.rs`: 侧边栏、顶部栏和连接对话框的 UI 与状态。

## 内容模块
- `src/app/content/common.rs`: 内容层公用工具箱；定义 `LoadState<T>`、`LoadStateMessages`、`generic_toolbar_button`、`load_state_list_view`、`centered_message`、`surface_panel` 等，统一加载状态、卡片样式和占位面板。
- `src/app/content/overview/mod.rs`: 保留 MySQL 概览数据结构与 SQL 常量，并从 `common` 导出 UI 工具；`MysqlContentState` 集中维护所有异步数据。
- `src/app/content/overview/tables.rs`: MySQL 表页，依赖 `MysqlContentState` 与 `TABLE_ICON_PATH`；使用 `loading_view`/`error_view` 等公共分支，并处理本地筛选、排序。
- `src/app/content/overview/functions.rs`: 函数/存储过程列表；通过 `load_state_list_view` + `LoadStateMessages` 消除状态分支重复。
- `src/app/content/overview/users.rs`: 用户列表页；同样复用 `load_state_list_view`，只关注 `user_row` 的渲染细节。
- `src/app/content/overview/queries.rs`: 查询占位页，借助 `surface_panel` 输出占位内容。
- `src/app/content/redis.rs`: Redis 概览页；`RedisContentState` 缓存 `INFO keyspace` 解析出的数据库列表，并通过卡片展示键数量/过期情况。
- `src/app/content/table_data.rs`: 表数据详情页；`sanitize_cell` 现会剔除所有控制字符和换行符，再截断超长内容，确保数据表格单元不会换行。
- `src/app/content/saved_queries.rs`, `saved_functions.rs`, `query_editor.rs`: 各自通过 `surface_panel` 渲染占位提示，避免重复容器样式。

## 存储层
- `src/driver/mod.rs`: 驱动注册、请求/响应定义。当前仅保留 SQL 查询路径；执行接口与 MongoDB 文档查询均已移除，`DriverRegistry` 只暴露 `test_connection` 与 `query`。
- `src/driver/mysql.rs`, `sqlite.rs`: 具体 SQL 驱动实现，直接返回 `QueryPayload::Tabular`。
- `src/driver/redis.rs`: Redis 驱动；提供同步 PING 测试与 `INFO KEYSPACE` 查询，统一封装成表格响应供概览页解析。
- `src/driver/mongodb.rs`: 仅提供连接探活，查询接口返回 `Unsupported`，避免死代码。

## 通用组件
- `src/comps/popup`, `src/comps/table.rs`: 复用 UI 组件与表格工具函数。`table.rs` 的单元格文本绘制使用无限宽度 + 图层裁剪，确保内容始终单行显示，即便实际文本超过当前列宽。
