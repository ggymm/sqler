use iced::Task;

mod app;
mod cache;
mod comps;
mod driver;

pub fn main() -> iced::Result {
    iced::application("SQler", app::update, app::view)
        .centered()
        .theme(app::theme)
        .default_font(app::default_font())
        .subscription(app::subscription)
        .run_with(|| (app::App::default(), Task::none()))
}
