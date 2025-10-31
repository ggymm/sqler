use gpui::{prelude::*, *};
use gpui_component::{
    form::{form_field, Form},
    input::{InputState, TextInput},
    Sizable, Size,
};

pub struct MongoDBCreate {
    pub name: Entity<InputState>,
    pub host: Entity<InputState>,
    pub port: Entity<InputState>,
    pub username: Entity<InputState>,
    pub password: Entity<InputState>,
    pub database: Entity<InputState>,
}

impl MongoDBCreate {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            name: cx.new(|cx| InputState::new(window, cx).default_value("MongoDB数据源")),
            host: cx.new(|cx| InputState::new(window, cx)),
            port: cx.new(|cx| InputState::new(window, cx)),
            username: cx.new(|cx| InputState::new(window, cx)),
            password: cx.new(|cx| InputState::new(window, cx).masked(true)),
            database: cx.new(|cx| InputState::new(window, cx)),
        }
    }
}

impl Render for MongoDBCreate {
    fn render(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div().flex().flex_col().gap_4().child(
            Form::vertical()
                .layout(Axis::Horizontal)
                .with_size(Size::Large)
                .label_width(px(80.))
                .child(form_field().label("名称").child(TextInput::new(&self.name).cleanable()))
                .child(form_field().label("主机").child(TextInput::new(&self.host).cleanable()))
                .child(form_field().label("端口").child(TextInput::new(&self.port).cleanable()))
                .child(
                    form_field()
                        .label("账号")
                        .child(TextInput::new(&self.username).cleanable()),
                )
                .child(
                    form_field()
                        .label("密码")
                        .child(TextInput::new(&self.password).mask_toggle().cleanable()),
                )
                .child(
                    form_field()
                        .label("数据库")
                        .child(TextInput::new(&self.database).cleanable()),
                ),
        )
    }
}
