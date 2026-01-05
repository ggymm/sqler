use std::{borrow::Cow, io::stdout, mem::forget};

use gpui::*;
use gpui_component::{Root, Theme, init, scroll::ScrollbarShow};
use rust_embed::RustEmbed;
use tracing_appender::{
    non_blocking,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{EnvFilter, fmt::layer, layer::SubscriberExt, util::SubscriberInitExt};

use sqler_core::logs_dir;

use crate::app::SqlerApp;

mod app;
mod comps;
mod create;
mod transfer;
mod workspace;

#[derive(RustEmbed)]
#[folder = "assets"]
#[include = "icons/**/*.svg"]
struct FsAssets;

impl AssetSource for FsAssets {
    fn load(
        &self,
        path: &str,
    ) -> Result<Option<Cow<'static, [u8]>>> {
        if path.is_empty() {
            return Ok(None);
        }
        Ok(Self::get(path).map(|f| f.data))
    }

    fn list(
        &self,
        path: &str,
    ) -> Result<Vec<SharedString>> {
        Ok(Self::iter()
            .filter_map(|p| p.starts_with(path).then(|| p.into()))
            .collect())
    }
}

fn init_runtime(_cx: &mut App) {
    let log_dir = logs_dir();

    let log_level = if cfg!(debug_assertions) { "debug" } else { "info" };
    let log_rolling = RollingFileAppender::builder()
        .rotation(Rotation::DAILY) // rotate log files once every hour
        .filename_prefix("sqler") // log file names will be prefixed with `myapp.`
        .filename_suffix("log") // log file names will be suffixed with `.log`
        .build(&log_dir)
        .expect("Failed to create log file appender");
    let (non_blocking, _guard) = non_blocking(log_rolling);

    tracing_subscriber::registry()
        .with(EnvFilter::new(log_level))
        .with(layer().with_writer(stdout))
        .with(layer().with_writer(non_blocking).with_ansi(false))
        .init();
    forget(_guard);
}

fn main() {
    let app = Application::new().with_assets(FsAssets);
    app.run(|cx| {
        init(cx);
        init_runtime(cx);

        tracing::info!("Sqler 应用启动成功");
        tracing::info!("版本: {}", env!("CARGO_PKG_VERSION"));

        cx.activate(true);
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        let theme = Theme::global_mut(cx);
        theme.scrollbar_show = ScrollbarShow::Hover;

        let window_size = size(px(1280.), px(800.));
        let window_bounds = Bounds::centered(None, window_size, cx);
        cx.open_window(
            WindowOptions {
                kind: WindowKind::Normal,
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
                cx.new(|cx| Root::new(view, window, cx))
            },
        )
        .expect("failed to open window")
        .update(cx, |_, window, _| {
            window.activate_window();
        })
        .expect("failed to update window");
    });
}
