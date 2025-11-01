use gpui::{prelude::*, *};
use gpui_component::{
    button::Button,
    form::{form_field, Form},
    input::{InputState, TextInput},
    Sizable, Size,
};

use crate::{app::comps::DivExt, option::SQLiteOptions};

pub struct SQLiteCreate {
    pub name: Entity<InputState>,
    pub filepath: Entity<InputState>,
    pub password: Entity<InputState>,
}

impl SQLiteCreate {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            name: cx.new(|cx| InputState::new(window, cx).default_value("SQLite数据源")),
            filepath: cx.new(|cx| InputState::new(window, cx)),
            password: cx.new(|cx| InputState::new(window, cx).masked(true)),
        }
    }

    pub fn choose_file(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let path = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            multiple: false,
            directories: false,
            prompt: Some("选择数据库文件".into()),
        });

        let filepath = self.filepath.clone();
        cx.spawn_in(window, async move |_, cx| {
            if let Ok(Ok(Some(mut paths))) = path.await {
                if let Some(path) = paths.pop() {
                    let p = path.display().to_string();
                    let _ = cx.update(|window, cx| {
                        filepath.update(cx, |this, cx| {
                            this.set_value(&p, window, cx);
                        });
                    });
                }
            }
        })
        .detach();
    }

    pub fn options(
        &self,
        cx: &App,
    ) -> SQLiteOptions {
        let filepath = self.filepath.read(cx).value().to_string();
        let password = self.password.read(cx).value().to_string();

        SQLiteOptions {
            filepath,
            password: if password.is_empty() { None } else { Some(password) },
            read_only: false,
        }
    }
}

impl Render for SQLiteCreate {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div().flex().flex_col().gap_4().child(
            Form::vertical()
                .layout(Axis::Horizontal)
                .with_size(Size::Large)
                .label_width(px(80.))
                .child(form_field().label("名称").child(TextInput::new(&self.name).cleanable()))
                .child(
                    form_field().label("文件").child(
                        div()
                            .gap_2()
                            .row_full()
                            .items_center()
                            .child(TextInput::new(&self.filepath).cleanable())
                            .child(Button::new("sqlite-choose-file").outline().label("选择文件").on_click(
                                cx.listener(|this: &mut SQLiteCreate, _ev, window, cx| {
                                    this.choose_file(window, cx);
                                }),
                            )),
                    ),
                )
                .child(
                    form_field()
                        .label("密码")
                        .child(TextInput::new(&self.password).mask_toggle().cleanable()),
                ),
        )
    }
}
