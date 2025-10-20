use std::borrow::Cow;
use std::fs::read;
use std::path::PathBuf;

use gpui::*;
use gpui_component::Root;
use gpui_component::{init, Theme, ThemeMode};

use app::SqlerApp;

mod app;
mod driver;

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

        Theme::change(ThemeMode::Light, None, cx);

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
