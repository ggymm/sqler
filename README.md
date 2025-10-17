### 项目目标

跨平台数据库管理软件

#### 前提

支持以下数据库
MySQL，MongoDB，Oracle，PostgreSQL，Redis，SQLite，SQLServer
的数据查询
表或者视图结构管理
数据的导入和导出

#### 主体框架

1. 使用gpui绘图库，代码目录在/Volumes/Data/Code/temp/gpui/zed/crates/gpui
2. 使用gpui-component组件库，代码目录在/Volumes/Data/Code/temp/gpui/gpui-component
3. 资源统一放在了assets目录下

布局

1. 项目整体采用分tab布局（类似于浏览器）
2. 项目首页展示所有保存的数据源，并在顶部工具栏提供“新建数据源”入口
3. 双击数据源，打开新tab展示数据源详细信息
4. 每个tab内部分为顶部菜单，左侧导航栏，右侧内容区。每个tab的内容根据数据源类型的不同而定义

Tab页面布局

1.

新建数据源

1. 打开新tab页
2. 
