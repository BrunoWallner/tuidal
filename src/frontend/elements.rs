use std::mem;

use ratatui::{
    Frame,
    prelude::{self, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{self, Block, BorderType, Borders, ListItem, ListState},
};

use crate::frontend::item::{Item, Items};

fn get_border(is_selected: bool) -> (Style, BorderType) {
    let border_style = if is_selected {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::Gray)
    };

    // Custom border symbols (optional)
    let border_type = if is_selected {
        BorderType::Thick // Fancier border when selected
    } else {
        BorderType::Plain
    };
    (border_style, border_type)
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UiElement {
    ItemList = 0,
    StatusInfo = 1,
    InputField = 2,
}
impl UiElement {
    const COUNT: u8 = 3;
    pub fn next(self) -> Self {
        let value = (self as u8 + 1) % Self::COUNT;
        Self::from_u8(value)
    }
    pub fn prev(self) -> Self {
        let value = (self as u8 + Self::COUNT + 1) % Self::COUNT;
        Self::from_u8(value)
    }
    fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::ItemList,
            1 => Self::StatusInfo,
            2 => Self::InputField,
            _ => unreachable!(),
        }
    }
}

pub struct ItemList<'a> {
    list: widgets::List<'a>,
    pub items: Items,
    pub ui_state: ListState,
}

impl<'a> ItemList<'a> {
    pub fn new() -> Self {
        Self {
            list: widgets::List::new(None::<Span>),
            items: Items::new(),
            ui_state: ListState::default(),
        }
    }
    // Move selection up
    pub fn select_previous(&mut self) {
        let i = match self.ui_state.selected() {
            Some(i) if i > 0 => i - 1,
            _ => 0,
        };
        self.ui_state.select(Some(i));
    }

    // Move selection down
    pub fn select_next(&mut self) {
        let i = match self.ui_state.selected() {
            Some(i) if i + 1 < self.items.get().len() => i + 1,
            Some(i) => i,
            None => 0,
        };
        self.ui_state.select(Some(i));
    }

    pub fn set_items(&'a mut self, items: Items) {
        self.items = items;
        self.build_list();
    }

    pub fn get_widget(&mut self, focused: bool) -> &widgets::List<'_> {
        let (border_style, border_type) = get_border(focused);
        self.list = mem::take(&mut self.list).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Items")
                .border_style(border_style)
                .border_type(border_type),
        );
        &self.list
    }

    pub fn render(&'a mut self, frame: &mut Frame, area: Rect) {
        let (border_style, border_type) = get_border(true);
        self.list = mem::take(&mut self.list).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Items")
                .border_style(border_style)
                .border_type(border_type),
        );
        frame.render_stateful_widget(&self.list, area, &mut self.ui_state);
        // let widget = self.get_widget(true);
        // let state = &mut self.ui_state;
        // frame.render_stateful_widget(widget, area, state);
        // // frame.render_stateful_widget(widget, area, state);
    }

    fn build_list(&'a mut self) {
        let mut list_items = Vec::new();
        let it = self.items.get();
        for i in it.iter() {
            list_items.push(ListItem::new(i.to_line()))
        }
        let list = widgets::List::new(list_items);
        self.list = list;
    }

    pub fn collapse(&mut self) {
        self.items.collapse();
    }

    pub fn push(&mut self, items: Vec<Item>) {
        self.items.push(items);
        self.apply_selection();
    }

    fn apply_selection(&mut self) {
        let sel = match self.items.get().len() {
            now if now > 0 => match self.ui_state.selected() {
                Some(prev) => Some(now.min(prev)),
                None => Some(1),
            },
            _ => None,
        };
        self.ui_state.select(sel);
    }
    pub fn back(&mut self) {
        self.items.back();
        self.apply_selection();
    }

    pub fn from_search(&mut self, result: tidal::SearchResult) {
        self.items.from_search(result);
    }
}
