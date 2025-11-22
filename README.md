### 项目目标

跨平台数据库管理软件

#### 数据库支持

1. MySQL
2. Oracle
3. SQLite
4. SQLServer
5. PostgreSQL
6. Redis
7. MongoDB

#### 主体框架

1. 资源统一放在 assets 目录
2. 使用 gpui 绘图库，代码目录在/Volumes/Data/Code/temp/gpui/zed/crates/gpui
3. 使用 gpui-component 组件库，代码目录在/Volumes/Data/Code/temp/gpui/gpui-component

#### 功能列表

1. 数据源管理

#### TODO

- [ ] 数据源测试和保存（MySQL，SQLite，Redis（集群），MongoDB，PostgreSQL）
- [ ] 数据导出和导入（transfer）（快速导入导出）
- [ ] Redis数据源工作区
- [x] 工作区中新增 SQL 查询页面
- [ ] 工作区 SQL 查询结果需要支持按照 tab 展示（支持一次执行多条语句）
- [ ] 工作区结构重构（优先级最低）
- [x] 将build的内容移动到driver中，修改queryReq的结构
- [ ] 工作区展示，保存的命令或者查询语句
- [ ] 编辑和删除数据源

#### 常用命令

```shell

# cargo install cargo-bloat
cargo bloat --release -n 50
cargo bloat --release --crates

# cargo install cargo-outdated
cargo outdated -R

```

~/.sqler
sources.db
cache
{uuid}
tables.json
queries.json
