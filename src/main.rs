use gpui::{
    px, size, App, AppContext as _, Application, AssetSource, Bounds, Result, SharedString,
    WindowBounds, WindowOptions,
};
use gpui_component::Root;
use std::{borrow::Cow, fs::read, path::PathBuf};

mod app;

use app::SqlerApp;

struct FsAssets;

impl AssetSource for FsAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
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

    fn list(&self, _path: &str) -> Result<Vec<SharedString>> {
        Ok(Vec::new())
    }
}

fn init_runtime(_cx: &mut App) {
    // TODO: 初始化数据库驱动、缓存等运行时组件
}

fn main() {
    let app = Application::new().with_assets(FsAssets);

    app.run(|cx: &mut App| {
        gpui_component::init(cx);
        init_runtime(cx);
        cx.activate(true);

        let window_bounds = Bounds::centered(None, size(px(1200.), px(800.)), cx);

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(window_bounds)),
                ..Default::default()
            },
            |window, cx| {
                let view = cx.new(|cx| SqlerApp::new(window, cx));
                cx.new(|cx| Root::new(view.into(), window, cx))
            },
        )
        .expect("failed to open window")
        .update(cx, |_, window, _| {
            window.set_window_title("Sqler");
            window.activate_window();
        })
        .unwrap();
    });
}
