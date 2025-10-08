use iced::Task;

mod app;
mod comps;
mod driver;

pub fn main() -> iced::Result {
    iced::application("SQler", app::update, app::view)
        .subscription(app::subscription)
        .theme(app::theme)
        .centered()
        .default_font(app::default_font())
        .run_with(|| (app::App::default(), Task::none()))
}
