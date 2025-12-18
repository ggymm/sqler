use gpui::{prelude::*, *};
use gpui_component::{
    IconName, IndexPath, Sizable, Size,
    button::{Button, ButtonVariants},
    form::{Form, field},
    input::{Input, InputState},
    select::{Select, SelectState},
};

use crate::model::{RedisKind, RedisOptions};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RedisAuthMode {
    None,
    Password,
    UsernamePassword,
}

impl RedisAuthMode {
    fn all() -> &'static [RedisAuthMode] {
        &[
            RedisAuthMode::None,
            RedisAuthMode::Password,
            RedisAuthMode::UsernamePassword,
        ]
    }

    fn label(&self) -> &'static str {
        match self {
            RedisAuthMode::None => "无认证",
            RedisAuthMode::Password => "密码认证",
            RedisAuthMode::UsernamePassword => "账号密码认证",
        }
    }
}

pub struct RedisCreate {
    pub kind: Entity<SelectState<Vec<SharedString>>>,
    pub host: Entity<InputState>,
    pub port: Entity<InputState>,
    pub nodes: Entity<InputState>,
    pub auth: Entity<SelectState<Vec<SharedString>>>,
    pub username: Entity<InputState>,
    pub password: Entity<InputState>,
}

impl RedisCreate {
    pub fn new(
        opts: Option<&RedisOptions>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let opts = opts.cloned().unwrap_or_default();

        let kinds: Vec<SharedString> = RedisKind::all().iter().map(|k| k.label().into()).collect();
        let auths: Vec<SharedString> = RedisAuthMode::all().iter().map(|m| m.label().into()).collect();

        let kind_index = Some(IndexPath::new(
            RedisKind::all().iter().position(|kind| kind == &opts.kind).unwrap_or(0),
        ));

        let auth_mode = match (&opts.username, &opts.password) {
            (Some(_), _) => RedisAuthMode::UsernamePassword,
            (None, Some(_)) => RedisAuthMode::Password,
            (None, None) => RedisAuthMode::None,
        };
        let auth_index = Some(IndexPath::new(
            RedisAuthMode::all()
                .iter()
                .position(|mode| mode == &auth_mode)
                .unwrap_or(0),
        ));

        Self {
            kind: cx.new(|cx| SelectState::new(kinds, kind_index, window, cx)),
            host: cx.new(|cx| InputState::new(window, cx).default_value(&opts.host)),
            port: cx.new(|cx| InputState::new(window, cx).default_value(&opts.port)),
            nodes: cx.new(|cx| InputState::new(window, cx).default_value(&opts.nodes)),
            auth: cx.new(|cx| SelectState::new(auths, auth_index, window, cx)),
            username: cx.new(|cx| InputState::new(window, cx).default_value(&opts.username.unwrap_or_default())),
            password: cx.new(|cx| {
                InputState::new(window, cx)
                    .default_value(&opts.password.unwrap_or_default())
                    .masked(true)
            }),
        }
    }

    pub fn options(
        &self,
        cx: &App,
    ) -> RedisOptions {
        let kind_label = self.kind.read(cx).selected_value();
        let kind = if kind_label.as_ref().map(|s| s.as_ref()) == Some(RedisKind::Cluster.label()) {
            RedisKind::Cluster
        } else {
            RedisKind::Standalone
        };

        let username = self.username.read(cx).value().to_string();
        let password = self.password.read(cx).value().to_string();
        RedisOptions {
            kind,
            host: self.host.read(cx).value().to_string(),
            port: self.port.read(cx).value().to_string(),
            nodes: self.nodes.read(cx).value().to_string(),
            username: if username.is_empty() { None } else { Some(username) },
            password: if password.is_empty() { None } else { Some(password) },
            use_tls: false,
        }
    }
}

impl Render for RedisCreate {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let kind = self.kind.read(cx).selected_value();
        let kind_cluster = kind
            .as_ref()
            .map(|s| s.as_ref() == RedisKind::Cluster.label())
            .unwrap_or(false);

        let auth = self.auth.read(cx).selected_value();
        let show_password = auth
            .as_ref()
            .map(|s| s.as_ref() == RedisAuthMode::Password.label())
            .unwrap_or(false);

        let show_username = auth
            .as_ref()
            .map(|s| s.as_ref() == RedisAuthMode::UsernamePassword.label())
            .unwrap_or(false);

        Form::vertical()
            .layout(Axis::Horizontal)
            .with_size(Size::Large)
            .label_width(px(80.))
            .child(field().label("连接类型").child(Select::new(&self.kind)))
            .when_else(
                kind_cluster,
                |form| {
                    form.child(
                        field().label("集群节点").child(
                            Input::new(&self.nodes).cleanable(true).suffix(
                                Button::new("redis-nodes-tooltip")
                                    .icon(IconName::Info)
                                    .ghost()
                                    .xsmall()
                                    .tooltip("格式如下:127.0.0.1:7000,127.0.0.1:7001"),
                            ),
                        ),
                    )
                },
                |form| {
                    form.child(field().label("主机").child(Input::new(&self.host).cleanable(true)))
                        .child(field().label("端口").child(Input::new(&self.port).cleanable(true)))
                },
            )
            .child(field().label("认证模式").child(Select::new(&self.auth)))
            .when(show_username, |form| {
                form.child(field().label("账号").child(Input::new(&self.username).cleanable(true)))
            })
            .when(show_username || show_password, |form| {
                form.child(
                    field()
                        .label("密码")
                        .child(Input::new(&self.password).mask_toggle().cleanable(true)),
                )
            })
    }
}
