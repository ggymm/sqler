mod app;
mod comps;

pub fn main() -> iced::Result {
    iced::application("Sqler", app::update, app::view)
        .default_font(app::default_font())
        .run()
}
