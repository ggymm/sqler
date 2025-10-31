### 项目目标

跨平台数据库管理软件

#### 项目要求

需要支持以下数据库

1. MySQL
2. Oracle
3. SQLite
4. SQLServer
5. PostgreSQL
6. Redis
7. MongoDB

核心功能

1. 表结构管理
2. 数据查询
3. 数据导入导出

#### 主体框架

1. 使用gpui绘图库，代码目录在/Volumes/Data/Code/temp/gpui/zed/crates/gpui
2. 使用gpui-component组件库，代码目录在/Volumes/Data/Code/temp/gpui/gpui-component
3. 资源统一放在了assets目录下

#### 设计方案

##### 整体布局

1. 整体采用 tab 布局
2. 首页展示全部数据源卡片视图
3. 顶部有切换主题和新建数据源按钮

##### 新建数据源

1. 新建数据源弹出新窗口
2. 需要有测试连接按钮
3. 有上一步，取消和确认按钮
4. 内容区域需要可以滚动

```rust

// 1. 标准库导入
use std::collections::HashMap;
use std::sync::Arc;

// 2. 外部 crate 导入（按字母顺序）
use gpui::{prelude::*, *};
use gpui_component::{
    button::{Button, ButtonVariants},
    dropdown::{Dropdown, DropdownState},
    input::{InputState, TextInput},
    resizable::{h_resizable, resizable_panel, ResizableState},
    switch::Switch,
    tab::{Tab, TabBar},
    table::Table,
    ActiveTheme, Disableable, InteractiveElementExt,
    Selectable, Sizable, Size, StyledExt,
};
use uuid::Uuid;

// 4. 当前 crate 导入（按模块分组）
use crate::{
    app::comps::{
        comp_id, icon_close, icon_export, icon_import,
        icon_relead, icon_search, icon_sheet, icon_trash,
        DataTable, DivExt,
    },
    build::{
        create_builder, ConditionValue, DatabaseType,
        FilterCondition, Operator, QueryConditions, SortOrder,
    },
    driver::{
        DatabaseDriver, DatabaseSession, DriverError,
        MySQLDriver, QueryReq, QueryResp,
    },
    option::{DataSource, DataSourceOptions},
};

```