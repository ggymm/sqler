use std::borrow::Cow;
use std::fs::read;
use std::path::PathBuf;

use gpui::*;
use gpui_component::init;
use gpui_component::Root;

use app::SqlerApp;

mod app;
mod driver;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DataSourceType {
    MySQL,
    Oracle,
    SQLite,
    SQLServer,
    PostgreSQL,
    Redis,
    MongoDB,
}

impl DataSourceType {

    pub fn all() -> &'static [DataSourceType] {
        &[
            DataSourceType::MySQL,
            DataSourceType::Oracle,
            DataSourceType::SQLite,
            DataSourceType::SQLServer,
            DataSourceType::PostgreSQL,
            DataSourceType::Redis,
            DataSourceType::MongoDB,
        ]
    }
    
    pub fn image(&self) -> &'static str {
        match self {
            DataSourceType::MySQL => "icons/mysql.svg",
            DataSourceType::Oracle => "icons/oracle.svg",
            DataSourceType::SQLite => "icons/sqlite.svg",
            DataSourceType::SQLServer => "icons/sqlserver.svg",
            DataSourceType::PostgreSQL => "icons/postgresql.svg",
            DataSourceType::Redis => "icons/redis.svg",
            DataSourceType::MongoDB => "icons/mongodb.svg",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            DataSourceType::MySQL => "MySQL",
            DataSourceType::Oracle => "Oracle",
            DataSourceType::SQLite => "SQLite",
            DataSourceType::SQLServer => "SQLServer",
            DataSourceType::PostgreSQL => "PostgreSQL",
            DataSourceType::Redis => "Redis",
            DataSourceType::MongoDB => "MongoDB",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            DataSourceType::MySQL => "常用于业务库与分析库，默认 utf8mb4",
            DataSourceType::Oracle => "Oracle",
            DataSourceType::SQLite => "本地文件数据库，适合轻量级项目",
            DataSourceType::SQLServer => "企业级数据库，支持实例/域账号",
            DataSourceType::PostgreSQL => "支持 Schema、SSL 等高级特性",
            DataSourceType::Redis => "Redis",
            DataSourceType::MongoDB => "MongoDB",
        }
    }
}

#[derive(Clone)]
pub struct ConnectionPreset {
    pub host: SharedString,
    pub port: SharedString,
    pub database: SharedString,
    pub username: SharedString,
}

#[derive(Clone)]
pub struct DataSourceMeta {
    pub id: u64,
    pub name: SharedString,
    pub kind: DataSourceType,
    pub description: SharedString,
    pub connection: ConnectionPreset,
    pub tables: Vec<SharedString>,
}

struct FsAssets;

impl AssetSource for FsAssets {
    fn load(
        &self,
        path: &str,
    ) -> Result<Option<Cow<'static, [u8]>>> {
        if path.is_empty() {
            return Ok(None);
        }

        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let full = manifest_dir.join("assets").join(path);

        match read(full) {
            Ok(data) => Ok(Some(Cow::Owned(data))),
            Err(_) => Ok(None),
        }
    }

    fn list(
        &self,
        _path: &str,
    ) -> Result<Vec<SharedString>> {
        Ok(Vec::new())
    }
}

fn init_runtime(_cx: &mut App) {
    // TODO: 初始化数据库驱动、缓存等运行时组件
}

fn main() {
    let app = Application::new().with_assets(FsAssets);
    app.run(|cx: &mut App| {
        init(cx);
        init_runtime(cx);
        cx.activate(true);
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        let window_size = size(px(1280.), px(800.));
        let window_bounds = Bounds::centered(None, window_size, cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(window_bounds)),
                titlebar: Some(TitlebarOptions {
                    title: None,
                    appears_transparent: false,
                    traffic_light_position: None,
                }),
                ..Default::default()
            },
            |window, cx| {
                let view = cx.new(|cx| SqlerApp::new(window, cx));
                cx.new(|cx| Root::new(view.into(), window, cx))
            },
        )
        .expect("failed to open window")
        .update(cx, |_, window, _| {
            window.activate_window();
        })
        .expect("failed to update window");
    });
}
