mod app;
mod comps;

pub fn main() -> iced::Result {
    iced::application("Sqler", app::update, app::view).run()
}
