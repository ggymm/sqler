use gpui::{prelude::*, *};

use sqler_core::{ArcCache, DataSource};

use crate::app::SqlerApp;

pub struct DumpWindow {
    cache: ArcCache,
    parent: WeakEntity<SqlerApp>,

    table: String,
    schema: bool,
    source: DataSource,
}

pub struct DumpWindowBuilder {
    cache: Option<ArcCache>,
    parent: Option<WeakEntity<SqlerApp>>,

    table: Option<String>,
    schema: Option<bool>,
    source: Option<DataSource>,
}

impl DumpWindowBuilder {
    pub fn new() -> Self {
        Self {
            cache: None,
            parent: None,

            table: None,
            schema: None,
            source: None,
        }
    }

    pub fn cache(
        mut self,
        cache: ArcCache,
    ) -> Self {
        self.cache = Some(cache);
        self
    }

    pub fn parent(
        mut self,
        parent: WeakEntity<SqlerApp>,
    ) -> Self {
        self.parent = Some(parent);
        self
    }

    pub fn table(
        mut self,
        table: String,
    ) -> Self {
        self.table = Some(table);
        self
    }

    pub fn schema(
        mut self,
        schema: bool,
    ) -> Self {
        self.schema = Some(schema);
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
        cx: &mut Context<DumpWindow>,
    ) -> DumpWindow {
        let cache = self.cache.unwrap();
        let parent = self.parent.unwrap();

        let table = self.table.unwrap();
        let schema = self.schema.unwrap();
        let source = self.source.unwrap();

        let parent_for_release = parent.clone();
        let _ = cx.on_release(move |_, app| {
            if let Some(parent) = parent_for_release.upgrade() {
                let _ = parent.update(app, |app, _| {
                    app.close_window("dump");
                });
            }
        });

        DumpWindow {
            cache,
            parent,
            table,
            schema,
            source,
        }
    }
}

impl Render for DumpWindow {
    fn render(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
    }
}
