use iced::Size;
use iced::Task;

mod app;
mod cache;
mod comps;
mod driver;

pub fn main() -> iced::Result {
    iced::application("SQLER", app::update, app::view)
        .window_size(Size::new(1280.0, 800.0))
        .centered()
        .theme(app::theme)
        .default_font(app::default_font())
        .subscription(app::subscription)
        .run_with(|| (app::App::default(), Task::none()))
}
