use std::collections::HashMap;
use std::time::Instant;

use gpui::{prelude::*, *};
use gpui_component::{
    ActiveTheme, Selectable, Sizable, Size, StyledExt,
    button::{Button, ButtonGroup},
    input::{Input, InputState},
    list::ListItem,
    resizable::{resizable_panel, v_resizable},
    select::{Select, SelectState},
    tree::{tree, TreeItem, TreeState},
};

use crate::{
    app::{
        SqlerApp,
        comps::{DivExt, comp_id, icon_relead},
    },
    driver::DatabaseSession,
    model::DataSource,
};

#[derive(Clone)]
pub struct KeyInfo {
    pub full_key: String,
    pub key_type: String,
    pub value: String,
    pub size: String,
    pub ttl: String,
}

fn build_tree_items(keys: &HashMap<String, KeyInfo>) -> Vec<TreeItem> {
    let mut tree_map: HashMap<String, Vec<String>> = HashMap::new();

    for key in keys.keys() {
        let parts: Vec<String> = key.split(':').map(|s| s.to_string()).collect();
        if parts.is_empty() {
            continue;
        }

        for i in 0..parts.len() {
            let parent_path = if i == 0 {
                String::new()
            } else {
                parts[0..i].join(":")
            };
            let current_path = parts[0..=i].join(":");

            tree_map
                .entry(parent_path)
                .or_insert_with(Vec::new)
                .push(current_path);
        }
    }

    fn build_node(
        path: &str,
        tree_map: &HashMap<String, Vec<String>>,
        keys: &HashMap<String, KeyInfo>,
    ) -> TreeItem {
        let parts: Vec<&str> = if path.is_empty() {
            vec![]
        } else {
            path.split(':').collect()
        };
        let name = parts.last().unwrap_or(&"").to_string();

        let children = tree_map
            .get(path)
            .map(|child_paths| {
                let mut unique_paths: Vec<String> = child_paths.iter().cloned().collect();
                unique_paths.sort();
                unique_paths.dedup();

                unique_paths
                    .into_iter()
                    .map(|child_path| build_node(&child_path, tree_map, keys))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        if children.is_empty() {
            TreeItem::new(path.to_string(), name)
        } else {
            TreeItem::new(path.to_string(), format!("{} ({})", name, children.len())).children(children)
        }
    }

    let root_children = tree_map.get("").cloned().unwrap_or_default();
    let mut unique_roots: Vec<String> = root_children;
    unique_roots.sort();
    unique_roots.dedup();

    unique_roots
        .into_iter()
        .map(|path| build_node(&path, &tree_map, keys))
        .collect()
}

pub enum ViewType {
    Overview,
    Browse,
    Command,
}

pub struct OverviewContent {}

pub struct BrowseContent {
    pub keys: HashMap<String, KeyInfo>,
    pub tree_state: Entity<TreeState>,
    pub selected_key: Option<String>,
    pub last_refresh_time: Instant,
    pub search_input: Entity<InputState>,
    pub key_type_filter: Entity<SelectState<Vec<SharedString>>>,
    pub detail_key_type: Entity<SelectState<Vec<SharedString>>>,
    pub detail_ttl: Entity<SelectState<Vec<SharedString>>>,
}

pub struct CommandContent {
    pub command_input: Entity<InputState>,
    pub command_result: Option<String>,
}

pub struct RedisWorkspace {
    pub parent: WeakEntity<SqlerApp>,

    pub source: DataSource,
    pub session: Option<Box<dyn DatabaseSession>>,

    pub active: ViewType,
    pub browse: Option<BrowseContent>,
    pub command: Option<CommandContent>,
    pub overview: Option<OverviewContent>,
}

impl RedisWorkspace {
    fn browse_content(&mut self) -> Option<&mut BrowseContent> {
        self.browse.as_mut()
    }

    fn command_content(&mut self) -> Option<&mut CommandContent> {
        self.command.as_mut()
    }

    fn switch_to_browse(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if matches!(self.active, ViewType::Browse) {
            return;
        }

        if self.browse.is_none() {
            let key_types = vec![
                "所有类型".into(),
                "string".into(),
                "hash".into(),
                "list".into(),
                "set".into(),
                "zset".into(),
            ];

            let detail_types = vec![
                "string".into(),
                "hash".into(),
                "list".into(),
                "set".into(),
                "zset".into(),
            ];

            let ttl_options = vec![
                "无TTL".into(),
                "10秒".into(),
                "1分钟".into(),
                "10分钟".into(),
                "1小时".into(),
            ];

            let keys = HashMap::new();
            let tree_items = build_tree_items(&keys);
            let tree_state = cx.new(|cx| TreeState::new(cx).items(tree_items));

            self.browse = Some(BrowseContent {
                keys,
                tree_state,
                selected_key: None,
                last_refresh_time: Instant::now(),
                search_input: cx.new(|cx| InputState::new(window, cx).placeholder("搜索")),
                key_type_filter: cx.new(|cx| SelectState::new(key_types, None, window, cx)),
                detail_key_type: cx.new(|cx| SelectState::new(detail_types, None, window, cx)),
                detail_ttl: cx.new(|cx| SelectState::new(ttl_options, None, window, cx)),
            });
        }

        self.active = ViewType::Browse;
        cx.notify();
    }

    fn switch_to_command(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if matches!(self.active, ViewType::Command) {
            return;
        }

        if self.command.is_none() {
            self.command = Some(CommandContent {
                command_input: cx.new(|cx| {
                    InputState::new(window, cx).placeholder("输入 Redis 命令，例如: GET key 或 SET key value")
                }),
                command_result: None,
            });
        }

        self.active = ViewType::Command;
        cx.notify();
    }

    fn switch_to_overview(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if matches!(self.active, ViewType::Overview) {
            return;
        }

        if self.overview.is_none() {
            self.overview = Some(OverviewContent {});
        }

        self.active = ViewType::Overview;
        cx.notify();
    }

    fn select_key(
        &mut self,
        key: String,
        cx: &mut Context<Self>,
    ) {
        if let Some(browse) = self.browse_content() {
            browse.selected_key = Some(key);
            cx.notify();
        }
    }

    fn find_key_info(
        &self,
        key: &str,
    ) -> Option<KeyInfo> {
        self.browse
            .as_ref()
            .and_then(|browse| browse.keys.get(key).cloned())
    }

    fn render_browse_view(
        &self,
        browse: &BrowseContent,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme().clone();
        let id = &self.source.id;
        let keys = browse.keys.clone();

        // 渲染键列表（左侧面板）
        let key_list = div()
            .flex()
            .flex_col()
            .gap_2()
            .col_full()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_2()
                    .p_2()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(div().flex_1().child(Select::new(&browse.key_type_filter).with_size(Size::Small)))
                    .child(div().w(px(200.)).child(Input::new(&browse.search_input).with_size(Size::Small))),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .px_2()
                    .py_1()
                    .gap_4()
                    .text_xs()
                    .font_semibold()
                    .text_color(theme.muted_foreground)
                    .border_b_1()
                    .border_color(theme.border)
                    .child(div().flex_1().child("键"))
                    .child(div().w(px(60.)).child("类型"))
                    .child(div().w(px(300.)).child("值"))
                    .child(div().w(px(60.)).child("大小"))
                    .child(div().w(px(80.)).child("TTL")),
            )
            .child(
                tree(&browse.tree_state, move |_ix, entry, _selected, _window, _cx| {
                    let item = entry.item();
                    let key_id = item.id.to_string();
                    let key_info = keys.get(key_id.as_str());

                    let mut list_item = ListItem::new(item.id.clone())
                        .pl(px(16.) * entry.depth() + px(12.))
                        .rounded_md();

                    if entry.is_folder() {
                        list_item = list_item.child(item.label.clone());
                    } else if let Some(info) = key_info {
                        list_item = list_item
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap_1()
                                    .child(div().flex_1().text_sm().child(item.label.clone()))
                                    .child(
                                        div()
                                            .flex()
                                            .flex_row()
                                            .gap_4()
                                            .text_xs()
                                            .child(div().w(px(60.)).child(info.key_type.clone()))
                                            .child(
                                                div()
                                                    .w(px(300.))
                                                    .overflow_hidden()
                                                    .whitespace_nowrap()
                                                    .child(info.value.clone()),
                                            )
                                            .child(div().w(px(60.)).child(info.size.clone()))
                                            .child(div().w(px(80.)).child(info.ttl.clone())),
                                    ),
                            );
                    }

                    list_item
                })
                .flex_1(),
            );

        // 渲染键详情（右侧面板）
        let key_detail = if let Some(key) = &browse.selected_key {
            if let Some(info) = self.find_key_info(key) {
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_4()
                    .col_full()
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .w(px(80.))
                                    .text_sm()
                                    .text_color(theme.muted_foreground)
                                    .child("键名称:"),
                            )
                            .child(div().flex_1().text_sm().child(info.full_key)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .w(px(80.))
                                    .text_sm()
                                    .text_color(theme.muted_foreground)
                                    .child("键类型:"),
                            )
                            .child(div().w(px(200.)).child(Select::new(&browse.detail_key_type).with_size(Size::Small))),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(div().text_sm().text_color(theme.muted_foreground).child("值:"))
                            .child(
                                div()
                                    .flex_1()
                                    .min_h(px(200.))
                                    .p_2()
                                    .text_sm()
                                    .border_1()
                                    .border_color(theme.border)
                                    .rounded_md()
                                    .bg(theme.secondary)
                                    .child(info.value),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .w(px(80.))
                                    .text_sm()
                                    .text_color(theme.muted_foreground)
                                    .child("TTL:"),
                            )
                            .child(div().w(px(200.)).child(Select::new(&browse.detail_ttl).with_size(Size::Small))),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_2()
                            .justify_end()
                            .child(Button::new(comp_id(["redis-detail-apply"])).label("应用").outline())
                            .child(Button::new(comp_id(["redis-detail-cancel"])).label("放弃").outline()),
                    )
                    .into_any_element()
            } else {
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .col_full()
                    .text_sm()
                    .text_color(theme.muted_foreground)
                    .child("请选择一个键查看详情")
                    .into_any_element()
            }
        } else {
            div()
                .flex()
                .items_center()
                .justify_center()
                .col_full()
                .text_sm()
                .text_color(theme.muted_foreground)
                .child("请选择一个键查看详情")
                .into_any_element()
        };

        v_resizable(comp_id(["redis-content", id]))
            .child(
                resizable_panel()
                    .size(px(300.0))
                    .size_range(px(200.0)..Pixels::MAX)
                    .child(key_list),
            )
            .child(key_detail)
            .into_any_element()
    }

    fn render_command_view(
        &self,
        command: &CommandContent,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme();
        let id = &self.source.id;

        div()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .col_full()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .font_semibold()
                            .text_color(theme.foreground)
                            .child("Redis 命令执行"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_2()
                            .child(div().flex_1().child(Input::new(&command.command_input).with_size(Size::Small)))
                            .child(
                                Button::new(comp_id(["redis-command-execute", id]))
                                    .label("执行")
                                    .outline(),
                            )
                            .child(
                                Button::new(comp_id(["redis-command-clear", id]))
                                    .label("清空")
                                    .outline(),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .flex_1()
                    .child(
                        div()
                            .text_sm()
                            .font_semibold()
                            .text_color(theme.foreground)
                            .child("执行结果"),
                    )
                    .child(
                        div()
                            .id(comp_id(["redis-command-result", id]))
                            .flex_1()
                            .p_3()
                            .text_sm()
                            .border_1()
                            .border_color(theme.border)
                            .rounded_md()
                            .bg(theme.secondary)
                            .overflow_y_scroll()
                            .when_some(command.command_result.as_ref(), |this, result| {
                                this.child(result.clone())
                            })
                            .when(command.command_result.is_none(), |this| {
                                this.child(
                                    div()
                                        .text_color(theme.muted_foreground)
                                        .child("命令执行结果将显示在此处"),
                                )
                            }),
                    ),
            )
            .into_any_element()
    }

    fn render_overview_view(
        &self,
        _overview: &OverviewContent,
        _cx: &mut Context<Self>,
    ) -> AnyElement {
        div().col_full().into_any_element()
    }
}

impl Render for RedisWorkspace {
    fn render(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let id = &self.source.id;
        let theme = cx.theme().clone();

        let is_overview = matches!(self.active, ViewType::Overview);
        let is_browse = matches!(self.active, ViewType::Browse);
        let is_command = matches!(self.active, ViewType::Command);

        let elapsed = self
            .browse
            .as_ref()
            .map(|b| b.last_refresh_time.elapsed())
            .unwrap_or_default();
        let time_str = if elapsed.as_secs() < 60 {
            format!("{}秒前", elapsed.as_secs())
        } else if elapsed.as_secs() < 3600 {
            format!("{}分钟前", elapsed.as_secs() / 60)
        } else {
            format!("{}小时前", elapsed.as_secs() / 3600)
        };

        div()
            .id(comp_id(["redis", id]))
            .col_full()
            .child(
                div()
                    .id(comp_id(["redis-header", id]))
                    .flex()
                    .flex_row()
                    .items_center()
                    .p_4()
                    .gap_2()
                    .border_b_1()
                    .border_color(theme.border)
                    .child(
                        ButtonGroup::new(comp_id(["redis-view-switcher", id]))
                            .outline()
                            .compact()
                            .child(
                                Button::new(comp_id(["redis-view-overview", id]))
                                    .label("概览")
                                    .selected(is_overview),
                            )
                            .child(
                                Button::new(comp_id(["redis-view-browse", id]))
                                    .label("浏览数据")
                                    .selected(is_browse),
                            )
                            .child(
                                Button::new(comp_id(["redis-view-command", id]))
                                    .label("执行命令")
                                    .selected(is_command),
                            )
                            .on_click(cx.listener(move |view, selected: &Vec<usize>, window, cx| {
                                match selected[0] {
                                    0 => view.switch_to_overview(window, cx),
                                    1 => view.switch_to_browse(window, cx),
                                    2 => view.switch_to_command(window, cx),
                                    _ => {}
                                }
                            })),
                    )
                    .child(
                        Button::new(comp_id(["redis-header-reload", id]))
                            .icon(icon_relead().with_size(Size::Small))
                            .label("刷新")
                            .outline(),
                    )
                    .child(
                        Button::new(comp_id(["redis-header-load-more", id]))
                            .label("获取更多")
                            .outline(),
                    )
                    .child(
                        Button::new(comp_id(["redis-header-load-all", id]))
                            .label("获取全部")
                            .outline(),
                    )
                    .child(
                        Button::new(comp_id(["redis-header-add", id]))
                            .label("新建数据")
                            .outline(),
                    )
                    .child(div().flex_1())
                    .child(
                        div()
                            .text_sm()
                            .text_color(theme.muted_foreground)
                            .child(format!("上次刷新: {}", time_str)),
                    ),
            )
            .child(match &self.active {
                ViewType::Overview => {
                    if let Some(overview) = &self.overview {
                        self.render_overview_view(overview, cx)
                    } else {
                        self.switch_to_overview(window, cx);
                        div().col_full().into_any_element()
                    }
                }
                ViewType::Browse => {
                    if let Some(browse) = &self.browse {
                        self.render_browse_view(browse, cx)
                    } else {
                        self.switch_to_browse(window, cx);
                        div().col_full().into_any_element()
                    }
                }
                ViewType::Command => {
                    if let Some(command) = &self.command {
                        self.render_command_view(command, cx)
                    } else {
                        self.switch_to_command(window, cx);
                        div().col_full().into_any_element()
                    }
                }
            })
    }
}
