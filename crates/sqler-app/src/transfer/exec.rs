use gpui::{prelude::*, *};

use sqler_core::DataSource;

pub struct ExecWindow {
    table: String,
    source: DataSource,
}

pub struct ExecWindowBuilder {
    table: Option<String>,
    source: Option<DataSource>,
}

impl ExecWindowBuilder {
    pub fn new() -> Self {
        Self {
            table: None,
            source: None,
        }
    }

    pub fn table(
        mut self,
        table: String,
    ) -> Self {
        self.table = Some(table);
        self
    }

    pub fn source(
        mut self,
        source: DataSource,
    ) -> Self {
        self.source = Some(source);
        self
    }

    pub fn build(
        self,
        _window: &mut Window,
        _cx: &mut Context<ExecWindow>,
    ) -> ExecWindow {
        let table = self.table.unwrap();
        let source = self.source.unwrap();

        ExecWindow { table, source }
    }
}

impl Render for ExecWindow {
    fn render(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
    }
}
