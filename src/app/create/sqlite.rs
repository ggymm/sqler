use gpui::{prelude::*, *};
use gpui_component::{
    Sizable, Size,
    button::Button,
    form::{Form, field},
    input::{Input, InputState},
};

use crate::{app::comps::DivExt, model::SQLiteOptions};

pub struct SQLiteCreate {
    pub filepath: Entity<InputState>,
    pub password: Entity<InputState>,
}

impl SQLiteCreate {
    pub fn new(
        opts: Option<&SQLiteOptions>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let opts = opts.cloned().unwrap_or_default();

        Self {
            filepath: cx.new(|cx| InputState::new(window, cx).default_value(&opts.filepath)),
            password: cx.new(|cx| {
                InputState::new(window, cx)
                    .default_value(&opts.password.unwrap_or_default())
                    .masked(true)
            }),
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
        let password = self.password.read(cx).value().to_string();

        SQLiteOptions {
            readonly: false,
            filepath: self.filepath.read(cx).value().to_string(),
            password: if password.is_empty() { None } else { Some(password) },
        }
    }
}

impl Render for SQLiteCreate {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        Form::vertical()
            .layout(Axis::Horizontal)
            .with_size(Size::Large)
            .label_width(px(80.))
            .child(
                field().label("文件").child(
                    div()
                        .gap_2()
                        .row_full()
                        .items_center()
                        .child(Input::new(&self.filepath).cleanable(true))
                        .child(
                            Button::new("sqlite-choose-file")
                                .label("选择文件")
                                .outline()
                                .on_click(cx.listener({
                                    // rustfmt::skip
                                    |this: &mut SQLiteCreate, _, window, cx| {
                                        this.choose_file(window, cx);
                                    }
                                })),
                        ),
                ),
            )
            .child(
                field()
                    .label("密码")
                    .child(Input::new(&self.password).mask_toggle().cleanable(true)),
            )
    }
}
