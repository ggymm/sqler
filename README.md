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
2. 数据表查看
3. 执行 SQL 并查看结果

#### 常用命令

```shell

# cargo install cargo-bloat
cargo bloat --release -n 50
cargo bloat --release --crates

# cargo install cargo-outdated
cargo outdated -R

# 打包
cargo clean
cargo build --release

# 格式化
cargo fmt

# 更新依赖
cargo clean
rm -rf Cargo.lock
cargo update

```

```txt

仔细阅读全部代码，更新 CODEMAP.md 文件
0. 在当前 CODEMAP.md 文件基础之上进行更新
1. 必须实际阅读所有代码文件，对比差异
2. 不要读取任何 git 历史
3. 认真探索文件内容，更新 CODEMAP.md 文件

```