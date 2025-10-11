use iced::widget::{Stack, button, column, container, horizontal_space, row, text};
use iced::{Alignment, Background, Color, Element, Font, Length, Shadow, Size, Subscription, Task, Theme, window};
use std::collections::HashMap;
use std::time::{Duration, Instant};

mod content;
mod dialog;
mod sidebar;
mod topbar;

use content::{
    LoadState, MysqlContentState, MysqlLoadState, MysqlProcess, MysqlRoutine, MysqlTable, MysqlTableData, MysqlUser,
    PROCESSLIST_SQL, ROUTINES_SQL, TABLES_SQL, TableMenuAction, USERS_SQL, content, parse_processlist, parse_routines,
    parse_table_data, parse_tables, parse_users,
};
use dialog::{ConnectionFormState, FormField, NewConnectionDialog, connection_info_modal, modal_view};
#[allow(unused_imports)]
pub use sidebar::ConnectionConfig;
use sidebar::sidebar;
pub use sidebar::{Connection, ConnectionsState, DatabaseKind};
use topbar::topbar;

use crate::comps::popup::overlay_backdrop;
use crate::driver::{DriverRegistry, QueryRequest};

#[derive(Debug)]
pub struct App {
    theme: ThemeMode,
    active_tab: ContentTab,
    connections: ConnectionsState,
    dialog: Option<NewConnectionDialog>,
    dialog_minimized: bool,
    drivers: DriverRegistry,
    active_connection: Option<usize>,
    connection_status: Option<ConnectionStatusInfo>,
    window_size: Size,
    mysql_content: HashMap<usize, MysqlContentState>,
    workspace_tabs: Vec<WorkspaceTab>,
    active_workspace_tab: usize,
    next_workspace_tab_id: usize,
}

impl Default for App {
    fn default() -> Self {
        let overview_tab = WorkspaceTab {
            id: 0,
            title: ContentTab::Tables.title().into(),
            kind: WorkspaceTabKind::Overview,
            closable: false,
        };

        Self {
            theme: ThemeMode::Light,
            active_tab: ContentTab::Tables,
            connections: ConnectionsState::new(),
            dialog: None,
            dialog_minimized: false,
            drivers: DriverRegistry::new(),
            active_connection: None,
            connection_status: None,
            window_size: Size::new(1280.0, 800.0),
            mysql_content: HashMap::new(),
            workspace_tabs: vec![overview_tab],
            active_workspace_tab: 0,
            next_workspace_tab_id: 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionStatusInfo {
    pub connection_id: usize,
    pub status: ConnectionStatus,
}

impl ConnectionStatusInfo {
    pub fn connecting(connection_id: usize) -> Self {
        Self {
            connection_id,
            status: ConnectionStatus::Connecting,
        }
    }

    pub fn success(connection_id: usize) -> Self {
        Self {
            connection_id,
            status: ConnectionStatus::Success,
        }
    }

    pub fn failed(
        connection_id: usize,
        reason: String,
    ) -> Self {
        Self {
            connection_id,
            status: ConnectionStatus::Failed(reason),
        }
    }

    pub fn details(connection_id: usize) -> Self {
        Self {
            connection_id,
            status: ConnectionStatus::Details,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConnectionStatus {
    Connecting,
    Success,
    Failed(String),
    Details,
}

impl App {
    pub fn palette(&self) -> Palette {
        self.theme.palette()
    }

    pub fn theme(&self) -> ThemeMode {
        self.theme
    }

    pub fn active_tab(&self) -> ContentTab {
        self.active_tab
    }

    pub fn selected_connection(&self) -> Option<usize> {
        self.connections.selected()
    }

    pub fn connection(
        &self,
        id: usize,
    ) -> Option<&Connection> {
        self.connections.find(id)
    }

    pub fn window_size(&self) -> Size {
        self.window_size
    }

    pub fn workspace_tabs(&self) -> &[WorkspaceTab] {
        &self.workspace_tabs
    }

    pub fn active_workspace_tab(&self) -> Option<&WorkspaceTab> {
        self.workspace_tabs.get(self.active_workspace_tab)
    }

    #[allow(dead_code)]
    pub fn active_connection(&self) -> Option<&Connection> {
        self.active_connection.and_then(|id| self.connections.find(id))
    }

    pub fn mysql_state(
        &self,
        id: usize,
    ) -> Option<&MysqlContentState> {
        self.mysql_content.get(&id)
    }

    fn schedule_mysql_load(
        &mut self,
        connection_id: usize,
        tab: ContentTab,
    ) -> Task<Message> {
        let Some(connection) = self.connections.find(connection_id).cloned() else {
            return Task::none();
        };

        if connection.kind != DatabaseKind::Mysql {
            return Task::none();
        }

        match tab {
            ContentTab::Tables => self.schedule_mysql_tables(connection_id, &connection),
            ContentTab::Queries => self.schedule_mysql_processlist(connection_id, &connection),
            ContentTab::Functions => self.schedule_mysql_routines(connection_id, &connection),
            ContentTab::Users => self.schedule_mysql_users(connection_id, &connection),
        }
    }

    fn schedule_mysql_tables(
        &mut self,
        connection_id: usize,
        connection: &Connection,
    ) -> Task<Message> {
        let should_load = {
            let state = self.mysql_content.entry(connection_id).or_default();
            if !state.tables.should_load() {
                false
            } else {
                state.tables = MysqlLoadState::Loading;
                true
            }
        };

        if !should_load {
            return Task::none();
        }

        let params = connection.to_params();
        let sql = TABLES_SQL.to_string();

        self.drivers
            .query(params, QueryRequest::Sql { statement: sql })
            .map(move |result| {
                let parsed = result
                    .map_err(|e| e.to_string())
                    .and_then(|response| parse_tables(response));
                Message::MysqlTablesLoaded(connection_id, parsed)
            })
    }

    fn schedule_mysql_table_data(
        &mut self,
        connection_id: usize,
        connection: &Connection,
        table_name: String,
    ) -> Task<Message> {
        let params = connection.to_params();
        let safe_name = table_name.replace('`', "``");
        let sql = format!("SELECT * FROM `{}` LIMIT 100", safe_name);
        let table_key = table_name.clone();

        self.drivers
            .query(params, QueryRequest::Sql { statement: sql })
            .map(move |result| {
                let parsed = result
                    .map_err(|e| e.to_string())
                    .and_then(|response| parse_table_data(response));
                Message::MysqlTableDataLoaded(connection_id, table_key.clone(), parsed)
            })
    }

    fn schedule_mysql_processlist(
        &mut self,
        connection_id: usize,
        connection: &Connection,
    ) -> Task<Message> {
        let should_load = {
            let state = self.mysql_content.entry(connection_id).or_default();
            if !state.processlist.should_load() {
                false
            } else {
                state.processlist = MysqlLoadState::Loading;
                true
            }
        };

        if !should_load {
            return Task::none();
        }

        let params = connection.to_params();
        let sql = PROCESSLIST_SQL.to_string();

        self.drivers
            .query(params, QueryRequest::Sql { statement: sql })
            .map(move |result| {
                let parsed = result
                    .map_err(|e| e.to_string())
                    .and_then(|response| parse_processlist(response));
                Message::MysqlProcesslistLoaded(connection_id, parsed)
            })
    }

    fn schedule_mysql_routines(
        &mut self,
        connection_id: usize,
        connection: &Connection,
    ) -> Task<Message> {
        let should_load = {
            let state = self.mysql_content.entry(connection_id).or_default();
            if !state.routines.should_load() {
                false
            } else {
                state.routines = MysqlLoadState::Loading;
                true
            }
        };

        if !should_load {
            return Task::none();
        }

        let params = connection.to_params();
        let sql = ROUTINES_SQL.to_string();

        self.drivers
            .query(params, QueryRequest::Sql { statement: sql })
            .map(move |result| {
                let parsed = result
                    .map_err(|e| e.to_string())
                    .and_then(|response| parse_routines(response));
                Message::MysqlRoutinesLoaded(connection_id, parsed)
            })
    }

    fn schedule_mysql_users(
        &mut self,
        connection_id: usize,
        connection: &Connection,
    ) -> Task<Message> {
        let should_load = {
            let state = self.mysql_content.entry(connection_id).or_default();
            if !state.users.should_load() {
                false
            } else {
                state.users = MysqlLoadState::Loading;
                true
            }
        };

        if !should_load {
            return Task::none();
        }

        let params = connection.to_params();
        let sql = USERS_SQL.to_string();

        self.drivers
            .query(params, QueryRequest::Sql { statement: sql })
            .map(move |result| {
                let parsed = result
                    .map_err(|e| e.to_string())
                    .and_then(|response| parse_users(response));
                Message::MysqlUsersLoaded(connection_id, parsed)
            })
    }
}

pub fn update(
    app: &mut App,
    message: Message,
) -> Task<Message> {
    match message {
        Message::ToggleTheme => app.theme.toggle(),
        Message::SelectContentTab(tab) => {
            app.active_tab = tab;
            if let Some(index) = app
                .workspace_tabs
                .iter()
                .position(|t| matches!(t.kind, WorkspaceTabKind::Overview))
            {
                if let Some(entry) = app.workspace_tabs.get_mut(index) {
                    entry.title = tab.title().into();
                }
                app.active_workspace_tab = index;
            }
            if let Some(active) = app.active_connection {
                return app.schedule_mysql_load(active, tab);
            }
        }
        Message::SelectWorkspaceTab(tab_id) => {
            if let Some(index) = app.workspace_tabs.iter().position(|tab| tab.id == tab_id) {
                app.active_workspace_tab = index;
            }
        }
        Message::CloseWorkspaceTab(tab_id) => {
            if let Some(index) = app.workspace_tabs.iter().position(|tab| tab.id == tab_id) {
                if !app.workspace_tabs[index].closable {
                    return Task::none();
                }
                app.workspace_tabs.remove(index);
                if app.workspace_tabs.is_empty() {
                    app.workspace_tabs.push(WorkspaceTab {
                        id: 0,
                        title: app.active_tab.title().into(),
                        kind: WorkspaceTabKind::Overview,
                        closable: false,
                    });
                    app.active_workspace_tab = 0;
                    app.next_workspace_tab_id = 1;
                } else if app.active_workspace_tab >= app.workspace_tabs.len() {
                    app.active_workspace_tab = app.workspace_tabs.len() - 1;
                } else if index <= app.active_workspace_tab && app.active_workspace_tab > 0 {
                    app.active_workspace_tab -= 1;
                }
            }
        }
        Message::ShowNewConnectionDialog => {
            app.dialog = Some(NewConnectionDialog::SelectingType);
            app.dialog_minimized = false;
        }
        Message::ShowNewQueryWorkspace => {
            let tab_id = app.next_workspace_tab_id;
            app.next_workspace_tab_id += 1;

            let connection_id = app.active_connection.or(app.selected_connection());

            let title = format!("查询 {}", tab_id);
            app.workspace_tabs.push(WorkspaceTab {
                id: tab_id,
                title,
                kind: WorkspaceTabKind::QueryEditor {
                    connection_id,
                    initial_sql: None,
                },
                closable: true,
            });
            app.active_workspace_tab = app.workspace_tabs.len() - 1;
        }
        Message::SelectConnection(id) => {
            let double_clicked = app.connections.select(id);

            if double_clicked {
                return Task::done(Message::ActivateConnection(id));
            }
        }
        Message::ActivateConnection(id) => {
            if let Some(connection) = app.connections.find(id) {
                let params = connection.to_params();
                app.connection_status = Some(ConnectionStatusInfo::connecting(id));

                let task = app
                    .drivers
                    .test_connection(params)
                    .map(move |result| Message::ConnectionActivationFinished(id, result.map_err(|e| e.to_string())));

                return task;
            } else {
                app.connection_status = Some(ConnectionStatusInfo::failed(id, "连接不存在".into()));
            }
        }
        Message::CancelDialog => {
            app.dialog = None;
            app.dialog_minimized = false;
        }
        Message::NewConnectionTypeSelected(kind) => {
            app.dialog = Some(NewConnectionDialog::Editing(ConnectionFormState::new(kind)));
            app.dialog_minimized = false;
        }
        Message::BackToConnectionTypeSelection => {
            if let Some(NewConnectionDialog::Editing(_)) = app.dialog {
                app.dialog = Some(NewConnectionDialog::SelectingType);
                app.dialog_minimized = false;
            }
        }
        Message::UpdateFormField(field, value) => {
            if let Some(NewConnectionDialog::Editing(form_state)) = &mut app.dialog {
                form_state.clear_error();
                form_state.form.update(field, value);
            }
        }
        Message::SubmitNewConnection => {
            if let Some(NewConnectionDialog::Editing(form_state)) = app.dialog.take() {
                let next_id = app.connections.next_id();
                let is_edit = form_state.existing_id.is_some();

                match form_state.build_connection(next_id) {
                    Ok(connection) => {
                        let connection_id = connection.id;
                        if is_edit {
                            app.connections.update(connection.clone());
                        } else {
                            app.connections.add(connection.clone());
                        }
                        app.mysql_content.remove(&connection_id);
                        app.dialog = None;
                        app.dialog_minimized = false;
                    }
                    Err(error) => {
                        let mut state = form_state;
                        state.error = Some(error);
                        app.dialog = Some(NewConnectionDialog::Editing(state));
                    }
                }
            }
        }
        Message::MinimizeDialog => {
            if app.dialog.is_some() {
                app.dialog_minimized = true;
            }
        }
        Message::RestoreDialog => {
            if app.dialog.is_some() {
                app.dialog_minimized = false;
            }
        }
        Message::WindowResized(_id, size) => {
            app.window_size = size;
        }
        Message::TestConnection => {
            if let Some(NewConnectionDialog::Editing(form_state)) = &mut app.dialog {
                match form_state.form.to_params() {
                    Ok(params) => {
                        form_state.testing = true;
                        form_state.test_result = None;

                        let task = app
                            .drivers
                            .test_connection(params)
                            .map(|result| Message::TestConnectionFinished(result.map_err(|e| e.to_string())));

                        return task;
                    }
                    Err(error) => {
                        form_state.test_result = Some(Err(error));
                    }
                }
            }
        }
        Message::TestConnectionFinished(result) => {
            if let Some(NewConnectionDialog::Editing(form_state)) = &mut app.dialog {
                form_state.testing = false;
                form_state.test_result = Some(result);
                app.dialog_minimized = false;
            }
        }
        Message::ConnectionActivationFinished(id, result) => match result {
            Ok(()) => {
                app.connections.activate(id);
                app.active_connection = Some(id);
                app.connection_status = Some(ConnectionStatusInfo::success(id));
                app.mysql_content.remove(&id);
                let tab = app.active_tab;
                return app.schedule_mysql_load(id, tab);
            }
            Err(error) => {
                if app.active_connection == Some(id) {
                    app.active_connection = None;
                    app.connections.deactivate();
                }
                app.connection_status = Some(ConnectionStatusInfo::failed(id, error));
            }
        },
        Message::ViewConnection(id) => {
            if app.connections.find(id).is_some() {
                app.connection_status = Some(ConnectionStatusInfo::details(id));
            } else {
                app.connection_status = Some(ConnectionStatusInfo::failed(id, "连接不存在".into()));
            }
        }
        Message::EditConnection(id) => {
            if let Some(connection) = app.connections.find(id) {
                let form_state = ConnectionFormState::from_connection(connection);
                app.dialog = Some(NewConnectionDialog::Editing(form_state));
                app.dialog_minimized = false;
            } else {
                app.connection_status = Some(ConnectionStatusInfo::failed(id, "连接不存在".into()));
            }
        }
        Message::DeleteConnection(id) => {
            app.connections.remove(id);
            app.mysql_content.remove(&id);
            if app.active_connection == Some(id) {
                app.active_connection = None;
                app.connection_status = None;
            }
            if app.connection_status.as_ref().map(|info| info.connection_id) == Some(id) {
                app.connection_status = None;
            }

            let mut removed_tabs = false;
            app.workspace_tabs.retain(|tab| {
                let should_remove = match &tab.kind {
                    WorkspaceTabKind::Overview => false,
                    WorkspaceTabKind::TableData { connection_id: cid, .. } => *cid == id,
                    WorkspaceTabKind::QueryEditor {
                        connection_id: Some(cid),
                        ..
                    } => *cid == id,
                    WorkspaceTabKind::QueryEditor {
                        connection_id: None, ..
                    } => false,
                    WorkspaceTabKind::SavedQueryList { connection_id: cid } => *cid == id,
                    WorkspaceTabKind::SavedFunctionList { connection_id: cid } => *cid == id,
                };

                if should_remove && tab.closable {
                    removed_tabs = true;
                    false
                } else {
                    true
                }
            });

            if app.workspace_tabs.is_empty() {
                app.workspace_tabs.push(WorkspaceTab {
                    id: 0,
                    title: app.active_tab.title().into(),
                    kind: WorkspaceTabKind::Overview,
                    closable: false,
                });
                app.active_workspace_tab = 0;
                app.next_workspace_tab_id = 1;
            } else if removed_tabs {
                if app.active_workspace_tab >= app.workspace_tabs.len() {
                    app.active_workspace_tab = app.workspace_tabs.len() - 1;
                }
            }
        }
        Message::DismissConnectionStatus => {
            app.connection_status = None;
        }
        Message::MysqlTablesLoaded(id, result) => {
            if let Some(state) = app.mysql_content.get_mut(&id) {
                state.tables = match result {
                    Ok(data) => {
                        state.selected_table = state.selected_table.filter(|idx| *idx < data.len());
                        MysqlLoadState::Ready(data)
                    }
                    Err(err) => MysqlLoadState::Error(err),
                };
            }
        }
        Message::MysqlProcesslistLoaded(id, result) => {
            if let Some(state) = app.mysql_content.get_mut(&id) {
                state.processlist = match result {
                    Ok(data) => MysqlLoadState::Ready(data),
                    Err(err) => MysqlLoadState::Error(err),
                };
            }
        }
        Message::MysqlRoutinesLoaded(id, result) => {
            if let Some(state) = app.mysql_content.get_mut(&id) {
                state.routines = match result {
                    Ok(data) => MysqlLoadState::Ready(data),
                    Err(err) => MysqlLoadState::Error(err),
                };
            }
        }
        Message::MysqlUsersLoaded(id, result) => {
            if let Some(state) = app.mysql_content.get_mut(&id) {
                state.users = match result {
                    Ok(data) => MysqlLoadState::Ready(data),
                    Err(err) => MysqlLoadState::Error(err),
                };
            }
        }
        Message::MysqlFilterTables(id, filter) => {
            app.mysql_content
                .entry(id)
                .or_insert_with(MysqlContentState::default)
                .table_filter = filter;
        }
        Message::MysqlOpenTableData(connection_id, table_name) => {
            let Some(connection) = app.connection(connection_id).cloned() else {
                return Task::none();
            };

            let mut should_schedule = false;
            {
                let state = app
                    .mysql_content
                    .entry(connection_id)
                    .or_insert_with(MysqlContentState::default);

                let entry = state
                    .table_data
                    .entry(table_name.clone())
                    .or_insert_with(Default::default);
                if matches!(entry, LoadState::Idle | LoadState::Error(_)) {
                    *entry = LoadState::Loading;
                    should_schedule = true;
                }
            }

            if let Some(index) = app.workspace_tabs.iter().position(|tab| {
                matches!(
                    &tab.kind,
                    WorkspaceTabKind::TableData {
                        connection_id: existing,
                        table_name: existing_name,
                    } if *existing == connection_id && *existing_name == table_name
                )
            }) {
                app.active_workspace_tab = index;
            } else {
                let tab_id = app.next_workspace_tab_id;
                app.next_workspace_tab_id += 1;
                let title = format!("{} 数据", table_name);
                app.workspace_tabs.push(WorkspaceTab {
                    id: tab_id,
                    title,
                    kind: WorkspaceTabKind::TableData {
                        connection_id,
                        table_name: table_name.clone(),
                    },
                    closable: true,
                });
                app.active_workspace_tab = app.workspace_tabs.len() - 1;
            }

            if should_schedule {
                return app.schedule_mysql_table_data(connection_id, &connection, table_name);
            }
        }
        Message::MysqlTableDataLoaded(connection_id, table_name, result) => {
            if let Some(state) = app.mysql_content.get_mut(&connection_id) {
                match result {
                    Ok(data) => {
                        state.table_data.insert(table_name.clone(), LoadState::Ready(data));
                    }
                    Err(err) => {
                        state.table_data.insert(table_name.clone(), LoadState::Error(err));
                    }
                }
            }
        }
        Message::MysqlTableMenuAction(_id, _action) => {
            // Actions will be wired once table-specific operations are implemented.
        }
        Message::MysqlSelectTable(connection_id, index) => {
            let mut open_table: Option<String> = None;
            {
                let state = app
                    .mysql_content
                    .entry(connection_id)
                    .or_insert_with(MysqlContentState::default);

                state.selected_table = Some(index);

                let now = Instant::now();
                let double_clicked = state
                    .last_table_click
                    .map(|(last_index, instant)| {
                        last_index == index && now.duration_since(instant) <= Duration::from_millis(400)
                    })
                    .unwrap_or(false);

                state.last_table_click = Some((index, now));

                if double_clicked {
                    if let MysqlLoadState::Ready(tables) = &state.tables {
                        if let Some(table) = tables.get(index) {
                            open_table = Some(table.name.clone());
                        }
                    }
                }
            }

            if let Some(table_name) = open_table {
                return Task::done(Message::MysqlOpenTableData(connection_id, table_name));
            }
        }
    }

    Task::none()
}

pub fn view(app: &App) -> Element<'_, Message> {
    let palette = app.palette();

    let base: Element<'_, Message> = container(
        column![topbar(app, palette), body(app, palette)]
            .spacing(0)
            .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(move |_| container::Style {
        background: Some(Background::Color(palette.background)),
        text_color: Some(palette.text),
        border: iced::border::Border::default(),
        shadow: Shadow::default(),
    })
    .into();

    let mut stack = Stack::new().width(Length::Fill).height(Length::Fill).push(base);

    if let Some(info) = &app.connection_status {
        let connection = app.connections.find(info.connection_id);

        stack = stack.push(overlay_backdrop(palette)).push(connection_info_modal(
            info,
            connection,
            palette,
            app.window_size(),
        ));
    }

    if let Some(dialog) = &app.dialog {
        stack = stack.push(overlay_backdrop(palette)).push(modal_view(
            dialog,
            palette,
            app.dialog_minimized,
            app.window_size(),
        ));
    }

    stack.into()
}

fn body(
    app: &App,
    palette: Palette,
) -> Element<'_, Message> {
    let sidebar_panel = container(sidebar(&app.connections, palette))
        .width(Length::Fixed(260.0))
        .style(move |_| container::Style {
            background: Some(Background::Color(palette.surface)),
            border: iced::border::Border {
                color: palette.border,
                width: 1.0,
                radius: 0.0.into(),
            },
            text_color: Some(palette.text),
            shadow: Shadow::default(),
        });

    let workspace_tabs = workspace_tabs_strip(app, palette);

    let workspace_content = app
        .active_workspace_tab()
        .map(|tab| content(app, palette, tab))
        .unwrap_or_else(|| {
            container(text("暂无打开的标签"))
                .width(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
        });

    let workspace_column = column![
        workspace_tabs,
        container(workspace_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(24)
            .style(move |_| container::Style {
                background: Some(Background::Color(palette.surface)),
                border: iced::border::Border {
                    color: palette.border,
                    width: 1.0,
                    radius: 0.0.into(),
                },
                text_color: Some(palette.text),
                shadow: Shadow::default(),
            }),
    ]
    .spacing(0)
    .height(Length::Fill);

    let workspace_panel = container(workspace_column)
        .width(Length::Fill)
        .style(move |_| container::Style {
            background: Some(Background::Color(palette.surface)),
            border: iced::border::Border {
                color: palette.border,
                width: 1.0,
                radius: 0.0.into(),
            },
            text_color: Some(palette.text),
            shadow: Shadow::default(),
        });

    row![sidebar_panel, workspace_panel].height(Length::Fill).into()
}

fn workspace_tabs_strip(
    app: &App,
    palette: Palette,
) -> Element<'static, Message> {
    let active_index = app.active_workspace_tab;

    let mut tabs_row = row![horizontal_space().width(Length::Fixed(4.0))];
    for (index, tab) in app.workspace_tabs().iter().cloned().enumerate() {
        let is_active = index == active_index;
        tabs_row = tabs_row.push(workspace_tab_entry(tab, is_active, palette));
    }

    tabs_row = tabs_row
        .push(horizontal_space().width(Length::Fill))
        .spacing(0)
        .align_y(Alignment::Center);

    container(tabs_row)
        .width(Length::Fill)
        .padding([8, 12])
        .style(move |_| container::Style {
            background: Some(Background::Color(palette.background)),
            border: iced::border::Border {
                color: palette.border,
                width: 1.0,
                radius: 0.0.into(),
            },
            text_color: Some(palette.text),
            shadow: Shadow::default(),
        })
        .into()
}

fn workspace_tab_entry(
    tab: WorkspaceTab,
    is_active: bool,
    palette: Palette,
) -> Element<'static, Message> {
    let tab_id = tab.id;
    let title = tab.title.clone();

    let title_text = text(title)
        .size(14)
        .color(if is_active { palette.text } else { palette.text_muted });

    let mut inner_row = row![title_text].spacing(8).align_y(Alignment::Center);

    if tab.closable {
        let close_button = button(
            text("×")
                .size(14)
                .color(if is_active { palette.text } else { palette.text_muted }),
        )
        .padding([4, 6])
        .style(move |_, status| {
            use iced::widget::button::Status;

            let hovered = matches!(status, Status::Hovered);
            let pressed = matches!(status, Status::Pressed);
            iced::widget::button::Style {
                background: Some(Background::Color(if pressed {
                    palette.surface
                } else if hovered {
                    palette.surface_muted
                } else {
                    Color::TRANSPARENT
                })),
                border: iced::border::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 16.0.into(),
                },
                text_color: if pressed {
                    palette.accent
                } else if is_active {
                    palette.text
                } else if hovered {
                    palette.text
                } else {
                    palette.text_muted
                },
                shadow: Shadow::default(),
            }
        })
        .on_press(Message::CloseWorkspaceTab(tab_id));

        inner_row = inner_row.push(close_button);
    }

    let button_body = container(inner_row).padding([6, 16]).style(move |_| container::Style {
        background: Some(Background::Color(if is_active {
            palette.surface
        } else {
            palette.background
        })),
        border: iced::border::Border {
            color: if is_active { palette.surface } else { Color::TRANSPARENT },
            width: 1.0,
            radius: 10.0.into(),
        },
        text_color: Some(palette.text),
        shadow: Shadow::default(),
    });

    button(button_body)
        .padding(0)
        .style(move |_, status| {
            use iced::widget::button::Status;

            let hovered = matches!(status, Status::Hovered);
            iced::widget::button::Style {
                background: Some(Background::Color(if is_active {
                    palette.surface
                } else if hovered {
                    palette.surface_muted
                } else {
                    palette.background
                })),
                border: iced::border::Border {
                    color: if is_active {
                        palette.surface
                    } else if hovered {
                        palette.border
                    } else {
                        Color::TRANSPARENT
                    },
                    width: if is_active { 1.0 } else { 0.0 },
                    radius: 10.0.into(),
                },
                text_color: palette.text,
                shadow: Shadow::default(),
            }
        })
        .on_press(Message::SelectWorkspaceTab(tab_id))
        .into()
}

pub fn subscription(_app: &App) -> Subscription<Message> {
    window::resize_events().map(|(id, size)| Message::WindowResized(id, size))
}

pub fn theme(app: &App) -> Theme {
    match app.theme {
        ThemeMode::Dark => Theme::Dark,
        ThemeMode::Light => Theme::Light,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentTab {
    Tables,
    Queries,
    Functions,
    Users,
}

impl ContentTab {
    pub fn title(&self) -> &'static str {
        match self {
            ContentTab::Tables => "表",
            ContentTab::Queries => "查询",
            ContentTab::Functions => "函数",
            ContentTab::Users => "用户",
        }
    }

    pub fn icon_path(&self) -> &'static str {
        match self {
            ContentTab::Tables => "assets/icons/table.svg",
            ContentTab::Queries => "assets/icons/query.svg",
            ContentTab::Functions => "assets/icons/function.svg",
            ContentTab::Users => "assets/icons/user.svg",
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorkspaceTab {
    pub id: usize,
    pub title: String,
    pub kind: WorkspaceTabKind,
    pub closable: bool,
}

#[derive(Debug, Clone)]
pub enum WorkspaceTabKind {
    Overview,
    TableData {
        connection_id: usize,
        table_name: String,
    },
    QueryEditor {
        connection_id: Option<usize>,
        initial_sql: Option<String>,
    },
    #[allow(dead_code)]
    SavedQueryList {
        connection_id: usize,
    },
    #[allow(dead_code)]
    SavedFunctionList {
        connection_id: usize,
    },
}

#[derive(Debug, Clone)]
pub enum Message {
    WindowResized(window::Id, Size),
    ToggleTheme,
    SelectContentTab(ContentTab),
    SelectWorkspaceTab(usize),
    CloseWorkspaceTab(usize),
    ShowNewConnectionDialog,
    ShowNewQueryWorkspace,
    SelectConnection(usize),
    ActivateConnection(usize),
    ViewConnection(usize),
    EditConnection(usize),
    DeleteConnection(usize),
    CancelDialog,
    NewConnectionTypeSelected(DatabaseKind),
    BackToConnectionTypeSelection,
    UpdateFormField(FormField, String),
    SubmitNewConnection,
    MinimizeDialog,
    RestoreDialog,
    TestConnection,
    TestConnectionFinished(Result<(), String>),
    ConnectionActivationFinished(usize, Result<(), String>),
    DismissConnectionStatus,
    MysqlTablesLoaded(usize, Result<Vec<MysqlTable>, String>),
    MysqlProcesslistLoaded(usize, Result<Vec<MysqlProcess>, String>),
    MysqlRoutinesLoaded(usize, Result<Vec<MysqlRoutine>, String>),
    MysqlUsersLoaded(usize, Result<Vec<MysqlUser>, String>),
    MysqlFilterTables(usize, String),
    MysqlOpenTableData(usize, String),
    MysqlTableDataLoaded(usize, String, Result<MysqlTableData, String>),
    MysqlTableMenuAction(usize, TableMenuAction),
    MysqlSelectTable(usize, usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Light,
}

impl ThemeMode {
    pub fn toggle(&mut self) {
        *self = match self {
            ThemeMode::Dark => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Dark,
        };
    }

    pub fn palette(&self) -> Palette {
        match self {
            ThemeMode::Light => Palette {
                background: Color::from_rgb8(0xf7, 0xf8, 0xfb),
                surface: Color::WHITE,
                surface_muted: Color::from_rgb8(0xee, 0xf0, 0xf4),
                border: Color::from_rgb8(0xd9, 0xde, 0xe7),
                text: Color::from_rgb8(0x1f, 0x24, 0x2f),
                text_muted: Color::from_rgb8(0x58, 0x60, 0x72),
                accent: Color::from_rgb8(0x42, 0x82, 0xff),
                accent_text: Color::WHITE,
                accent_soft: Color::from_rgb8(0xd9, 0xe7, 0xff),
                overlay: Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.55,
                },
            },
            ThemeMode::Dark => Palette {
                background: Color::from_rgb8(0x18, 0x1c, 0x24),
                surface: Color::from_rgb8(0x21, 0x26, 0x31),
                surface_muted: Color::from_rgb8(0x29, 0x2f, 0x3d),
                border: Color::from_rgb8(0x35, 0x3c, 0x4a),
                text: Color::from_rgb8(0xf1, 0xf5, 0xff),
                text_muted: Color::from_rgb8(0x9e, 0xa6, 0xb9),
                accent: Color::from_rgb8(0x66, 0x9b, 0xff),
                accent_text: Color::WHITE,
                accent_soft: Color::from_rgb8(0x2a, 0x3b, 0x59),
                overlay: Color {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 0.6,
                },
            },
        }
    }
}

#[derive(Clone, Copy)]
pub struct Palette {
    pub background: Color,
    pub surface: Color,
    pub surface_muted: Color,
    pub border: Color,
    pub text: Color,
    pub text_muted: Color,
    pub accent: Color,
    pub accent_text: Color,
    pub accent_soft: Color,
    pub overlay: Color,
}

pub fn default_font() -> Font {
    if cfg!(target_os = "macos") {
        Font::with_name("PingFang SC")
    } else if cfg!(target_os = "windows") {
        Font::with_name("Microsoft YaHei UI")
    } else if cfg!(any(target_os = "android", target_os = "linux")) {
        Font::with_name("Noto Sans CJK SC")
    } else if cfg!(target_os = "ios") {
        Font::with_name("PingFang TC")
    } else {
        Font::with_name("Noto Sans CJK SC")
    }
}
