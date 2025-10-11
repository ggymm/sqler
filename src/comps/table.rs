use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::text::{self, Renderer as _};
use iced::advanced::widget::{self, Tree, Widget};
use iced::advanced::{Clipboard, Renderer as _, Shell};
use iced::alignment;
use iced::event;
use iced::mouse;
use iced::window::RedrawRequest;
use iced::{Background, Border, Element, Event, Length, Pixels, Point, Rectangle, Shadow, Size};

use crate::app::Palette;

const HEADER_HEIGHT: f32 = 36.0;
const ROW_HEIGHT: f32 = 32.0;
const COLUMN_MIN_WIDTH: f32 = 80.0;
const RESIZE_HANDLE_RADIUS: f32 = 5.0;
const SCROLLBAR_HEIGHT: f32 = 12.0;

#[derive(Debug, Clone)]
pub struct TableColumn {
    pub title: String,
    pub width: f32,
    pub min_width: f32,
}

impl TableColumn {
    pub fn new(
        title: impl Into<String>,
        width: f32,
    ) -> Self {
        Self {
            title: title.into(),
            width: width.max(COLUMN_MIN_WIDTH),
            min_width: COLUMN_MIN_WIDTH,
        }
    }

    pub fn with_min_width(
        mut self,
        min_width: f32,
    ) -> Self {
        self.min_width = min_width.max(40.0);
        if self.width < self.min_width {
            self.width = self.min_width;
        }
        self
    }
}

#[derive(Debug, Clone)]
pub struct TableRow {
    pub cells: Vec<String>,
}

impl TableRow {
    pub fn new(cells: Vec<String>) -> Self {
        Self { cells }
    }
}

#[derive(Debug, Clone)]
struct DragState {
    column: usize,
    start_x: f32,
    initial_width: f32,
}

#[derive(Debug, Clone)]
enum ActiveDrag {
    Column(DragState),
    Horizontal,
}

#[derive(Debug, Default)]
struct InternalState {
    column_widths: Vec<f32>,
    scroll_x: f32,
    scroll_y: f32,
    viewport: Size,
    drag: Option<ActiveDrag>,
}

impl InternalState {
    fn new(columns: &[TableColumn]) -> Self {
        let column_widths = columns
            .iter()
            .map(|c| c.width.max(c.min_width).max(COLUMN_MIN_WIDTH))
            .collect();

        Self {
            column_widths,
            scroll_x: 0.0,
            scroll_y: 0.0,
            viewport: Size::ZERO,
            drag: None,
        }
    }

    fn ensure_columns(
        &mut self,
        columns: &[TableColumn],
    ) {
        if self.column_widths.len() != columns.len() {
            self.column_widths = columns
                .iter()
                .map(|c| c.width.max(c.min_width).max(COLUMN_MIN_WIDTH))
                .collect();
            self.scroll_x = 0.0;
        }
    }

    fn total_width(&self) -> f32 {
        self.column_widths.iter().sum()
    }

    fn clamp_scroll(
        &mut self,
        row_count: usize,
    ) {
        let content_height = row_count as f32 * ROW_HEIGHT;
        let max_y = (content_height - self.viewport.height).max(0.0);
        self.scroll_y = self.scroll_y.clamp(0.0, max_y);

        let max_x = (self.total_width() - self.viewport.width).max(0.0);
        self.scroll_x = self.scroll_x.clamp(0.0, max_x);
    }
}

pub struct DataTable<Message> {
    columns: Vec<TableColumn>,
    rows: Vec<TableRow>,
    palette: Palette,
    _marker: std::marker::PhantomData<Message>,
}

pub fn data_table<Message>(
    columns: Vec<TableColumn>,
    rows: Vec<TableRow>,
    palette: Palette,
) -> DataTable<Message> {
    DataTable {
        columns,
        rows,
        palette,
        _marker: std::marker::PhantomData,
    }
}

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for DataTable<Message> {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(
        &self,
        tree: &mut Tree,
        _renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = limits.resolve(self.size().width, self.size().height, Size::ZERO);

        let state = tree.state.downcast_mut::<InternalState>();
        state.viewport = Size::new(size.width, (size.height - HEADER_HEIGHT - SCROLLBAR_HEIGHT).max(0.0));
        state.ensure_columns(&self.columns);
        state.clamp_scroll(self.rows.len());

        layout::Node::new(size)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        _theme: &iced::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<InternalState>();

        let header_background = renderer::Quad {
            bounds: Rectangle {
                x: bounds.x,
                y: bounds.y,
                width: bounds.width,
                height: HEADER_HEIGHT,
            },
            border: Border::default(),
            shadow: Shadow::default(),
        };
        renderer.fill_quad(header_background, Background::Color(self.palette.surface));

        let mut column_x = bounds.x - state.scroll_x;
        for (index, column) in self.columns.iter().enumerate() {
            let width = state.column_widths.get(index).copied().unwrap_or(column.width);
            let text_bounds = Size::new((width - 24.0).max(0.0), HEADER_HEIGHT);

            renderer.fill_text(
                text::Text {
                    content: column.title.clone(),
                    bounds: text_bounds,
                    size: Pixels(14.0),
                    line_height: text::LineHeight::Relative(1.0),
                    font: Default::default(),
                    horizontal_alignment: alignment::Horizontal::Left,
                    vertical_alignment: alignment::Vertical::Center,
                    shaping: text::Shaping::Basic,
                    wrapping: text::Wrapping::None,
                },
                Point::new(column_x + 12.0, bounds.y + HEADER_HEIGHT / 2.0),
                self.palette.text,
                Rectangle {
                    x: column_x,
                    y: bounds.y,
                    width,
                    height: HEADER_HEIGHT,
                },
            );

            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: column_x + width - 0.5,
                        y: bounds.y + 6.0,
                        width: 1.0,
                        height: HEADER_HEIGHT - 12.0,
                    },
                    border: Border::default(),
                    shadow: Shadow::default(),
                },
                Background::Color(self.palette.border),
            );

            column_x += width;
        }

        let body_top = bounds.y + HEADER_HEIGHT;
        let body_bounds = Rectangle {
            x: bounds.x,
            y: body_top,
            width: bounds.width,
            height: state.viewport.height,
        };

        renderer.with_layer(body_bounds, |renderer| {
            let start_row = (state.scroll_y / ROW_HEIGHT).floor() as usize;
            let mut row_y = body_top - (state.scroll_y % ROW_HEIGHT);
            let max_rows = ((state.viewport.height / ROW_HEIGHT).ceil() as usize) + 1;

            for row_index in start_row..(start_row + max_rows).min(self.rows.len()) {
                if row_y > body_bounds.y + body_bounds.height {
                    break;
                }

                let background = if row_index % 2 == 0 {
                    self.palette.surface
                } else {
                    self.palette.surface_muted
                };

                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x,
                            y: row_y,
                            width: bounds.width,
                            height: ROW_HEIGHT,
                        },
                        border: Border::default(),
                        shadow: Shadow::default(),
                    },
                    Background::Color(background),
                );

                let mut cell_x = bounds.x - state.scroll_x;
                let row = &self.rows[row_index];

                for (col_index, width) in state.column_widths.iter().enumerate() {
                    if let Some(value) = row.cells.get(col_index) {
                        renderer.fill_text(
                            text::Text {
                                content: value.clone(),
                                bounds: Size::new((width - 24.0).max(0.0), ROW_HEIGHT),
                                size: Pixels(13.0),
                                line_height: text::LineHeight::Relative(1.0),
                                font: Default::default(),
                                horizontal_alignment: alignment::Horizontal::Left,
                                vertical_alignment: alignment::Vertical::Center,
                                shaping: text::Shaping::Basic,
                                wrapping: text::Wrapping::None,
                            },
                            Point::new(cell_x + 12.0, row_y + ROW_HEIGHT / 2.0),
                            self.palette.text,
                            Rectangle {
                                x: cell_x,
                                y: row_y,
                                width: *width,
                                height: ROW_HEIGHT,
                            },
                        );
                    }

                    cell_x += *width;
                }

                row_y += ROW_HEIGHT;
            }
        });

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x,
                    y: bounds.y + HEADER_HEIGHT - 0.5,
                    width: bounds.width,
                    height: 1.0,
                },
                border: Border::default(),
                shadow: Shadow::default(),
            },
            Background::Color(self.palette.border),
        );

        if let Some((track_bounds, thumb_width, available, max_scroll_x)) = horizontal_metrics(state, bounds) {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: track_bounds,
                    border: Border::default(),
                    shadow: Shadow::default(),
                },
                Background::Color(self.palette.surface_muted),
            );

            let ratio = if max_scroll_x > 0.0 {
                (state.scroll_x / max_scroll_x).clamp(0.0, 1.0)
            } else {
                0.0
            };

            let thumb_x = track_bounds.x + ratio * available;

            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: thumb_x,
                        y: track_bounds.y + 2.0,
                        width: thumb_width,
                        height: track_bounds.height - 4.0,
                    },
                    border: Border {
                        color: self.palette.border,
                        width: 1.0,
                        radius: 4.0.into(),
                    },
                    shadow: Shadow::default(),
                },
                Background::Color(self.palette.surface),
            );
        }
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &iced::Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        let bounds = layout.bounds();
        let state = tree.state.downcast_mut::<InternalState>();

        match event {
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                match delta {
                    mouse::ScrollDelta::Lines { x, y } => {
                        state.scroll_x -= x * COLUMN_MIN_WIDTH * 0.5;
                        state.scroll_y -= y * ROW_HEIGHT * 0.5;
                    }
                    mouse::ScrollDelta::Pixels { x, y } => {
                        state.scroll_x -= x;
                        state.scroll_y -= y;
                    }
                }
                state.clamp_scroll(self.rows.len());
                shell.request_redraw(RedrawRequest::NextFrame);
                event::Status::Captured
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let Some(position) = cursor.position_in(bounds) else {
                    return event::Status::Ignored;
                };

                if position.y <= HEADER_HEIGHT && self.columns.len() > 1 {
                    let absolute_x = position.x + state.scroll_x;
                    let mut acc = 0.0;
                    for column in 0..(self.columns.len() - 1) {
                        acc += state.column_widths[column];
                        if (absolute_x - acc).abs() <= RESIZE_HANDLE_RADIUS {
                            state.drag = Some(ActiveDrag::Column(DragState {
                                column,
                                start_x: absolute_x,
                                initial_width: state.column_widths[column],
                            }));
                            return event::Status::Captured;
                        }
                    }
                }

                if let Some((track_bounds, thumb_width, available, max_scroll_x)) = horizontal_metrics(state, bounds) {
                    if position.y >= track_bounds.y && position.y <= track_bounds.y + track_bounds.height {
                        let target = (position.x - track_bounds.x - thumb_width / 2.0).clamp(0.0, available);
                        if available > 0.0 {
                            state.scroll_x = (target / available) * max_scroll_x;
                            state.clamp_scroll(self.rows.len());
                            shell.request_redraw(RedrawRequest::NextFrame);
                        }
                        state.drag = Some(ActiveDrag::Horizontal);
                        return event::Status::Captured;
                    }
                }

                event::Status::Ignored
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if state.drag.take().is_some() {
                    return event::Status::Captured;
                }
                event::Status::Ignored
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                match state.drag {
                    Some(ActiveDrag::Column(ref mut drag)) => {
                        let relative = Point::new(position.x - bounds.x, position.y - bounds.y);
                        let absolute_x = relative.x + state.scroll_x;
                        let column = drag.column;
                        let new_width = (drag.initial_width + absolute_x - drag.start_x)
                            .max(self.columns[column].min_width)
                            .max(COLUMN_MIN_WIDTH);
                        state.column_widths[column] = new_width;
                        state.clamp_scroll(self.rows.len());
                        shell.request_redraw(RedrawRequest::NextFrame);
                        return event::Status::Captured;
                    }
                    Some(ActiveDrag::Horizontal) => {
                        if let Some((track_bounds, thumb_width, available, max_scroll_x)) =
                            horizontal_metrics(state, bounds)
                        {
                            let target = (position.x - track_bounds.x - thumb_width / 2.0).clamp(0.0, available);
                            if available > 0.0 {
                                state.scroll_x = (target / available) * max_scroll_x;
                                state.clamp_scroll(self.rows.len());
                                shell.request_redraw(RedrawRequest::NextFrame);
                            }
                            return event::Status::Captured;
                        }
                    }
                    None => {}
                }

                event::Status::Ignored
            }
            _ => event::Status::Ignored,
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if self.columns.len() <= 1 {
            return mouse::Interaction::Idle;
        }

        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<InternalState>();

        if let Some(position) = cursor.position_in(bounds) {
            if position.y <= HEADER_HEIGHT {
                let absolute_x = position.x + state.scroll_x;
                let mut acc = 0.0;
                for column in 0..(self.columns.len() - 1) {
                    acc += state.column_widths[column];
                    if (absolute_x - acc).abs() <= RESIZE_HANDLE_RADIUS {
                        return mouse::Interaction::ResizingHorizontally;
                    }
                }
            }

            if let Some((track_bounds, ..)) = horizontal_metrics(state, bounds) {
                if position.y >= track_bounds.y && position.y <= track_bounds.y + track_bounds.height {
                    return if matches!(state.drag, Some(ActiveDrag::Horizontal)) {
                        mouse::Interaction::Grabbing
                    } else {
                        mouse::Interaction::Grab
                    };
                }
            }
        }

        mouse::Interaction::Idle
    }

    fn diff(
        &self,
        tree: &mut Tree,
    ) {
        tree.state.downcast_mut::<InternalState>().ensure_columns(&self.columns);
    }

    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<InternalState>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(InternalState::new(&self.columns))
    }
}

fn horizontal_metrics(
    state: &InternalState,
    bounds: Rectangle,
) -> Option<(Rectangle, f32, f32, f32)> {
    if state.viewport.width <= 0.0 {
        return None;
    }

    let total_width = state.total_width();
    let max_scroll_x = (total_width - state.viewport.width).max(0.0);
    if max_scroll_x <= 0.0 {
        return None;
    }

    let track_bounds = Rectangle {
        x: bounds.x,
        y: bounds.y + HEADER_HEIGHT + state.viewport.height,
        width: bounds.width,
        height: SCROLLBAR_HEIGHT,
    };

    if track_bounds.height <= 0.0 {
        return None;
    }

    let visible_ratio = (state.viewport.width / total_width).clamp(0.05, 1.0);
    let thumb_width = (track_bounds.width * visible_ratio).clamp(24.0, track_bounds.width);
    let available = (track_bounds.width - thumb_width).max(1.0);

    Some((track_bounds, thumb_width, available, max_scroll_x))
}

impl<Message: 'static> From<DataTable<Message>> for Element<'static, Message> {
    fn from(table: DataTable<Message>) -> Self {
        Element::new(table)
    }
}
