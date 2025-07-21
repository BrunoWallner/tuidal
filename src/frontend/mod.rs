mod elements;
mod item;

use item::Items;
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::event,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};
use std::{
    error::Error,
    io::{self},
    sync::mpsc::{Sender, channel},
    time::Duration,
};

use tidal::{
    SearchType, Session, SessionError,
    media::{AudioQuality, Track},
};

use crate::{
    backend::{self, PlayerCtl},
    frontend::{
        elements::{ItemList, UiElement},
        item::Item,
    },
};
pub struct Frontend<'a> {
    pub session: Session,
    player_tx: Sender<PlayerCtl>,
    terminal: DefaultTerminal,
    ui: Ui<'a>,
    running: bool,
}

pub struct Ui<'a> {
    // items: Items,
    item_list: elements::ItemList<'a>,
    input_field: String,
    selected_element: UiElement,
}
impl<'a> Ui<'a> {
    pub fn new() -> Self {
        Self {
            // items: Items::new(),
            item_list: ItemList::new(),
            input_field: String::new(),
            selected_element: UiElement::ItemList,
        }
    }
}

impl<'a> Frontend<'a> {
    pub fn new(session: Session) -> Self {
        let terminal = ratatui::init();
        let (player_tx, player_rx) = channel();
        tokio::task::spawn_blocking(move || backend::init(player_rx).unwrap());
        let ui = Ui::new();
        Self {
            session,
            player_tx,
            ui,
            terminal,
            running: true,
        }
    }

    fn draw_ui(ui: &'a mut Ui<'a>, frame: &mut Frame) {
        let io = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100), Constraint::Min(3)])
            .split(frame.area());

        let list_and_info = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100), Constraint::Length(5)])
            .split(io[0]);

        // let mut list_items = Vec::new();
        // let it = ui.items.get().clone();
        // for i in it.iter() {
        //     list_items.push(ListItem::new(i.to_line()))
        // }

        // let is_selected = ui.selected_element == UiElement::ItemList;
        // let item_list = ui.item_list.get_widget(is_selected);
        // let item_list_state = &mut ui.item_list.items.ui_state;

        // let item_list = List::new(list_items)
        //     .block(
        //         Block::default()
        //             .borders(Borders::ALL)
        //             .title("Items")
        //             .border_style(border_style)
        //             .border_type(border_type),
        //     )
        //     .highlight_style(Style::default().fg(Color::Black).bg(Color::White));

        let right_widget =
            Paragraph::new("Right").block(Block::default().title("Right").borders(Borders::ALL));

        let input_widget = Paragraph::new(ui.input_field.clone())
            .block(Block::default().title("input").borders(Borders::ALL));

        // if ui.items.is_dirty {
        //     frame.render_widget(Clear, list_and_info[0]);
        //     ui.items.is_dirty = false;
        // }
        // frame.render_stateful_widget(
        //     item_list,
        //     list_and_info[0],
        //     // &mut ui.item_list.items.ui_state,
        //     item_list_state,
        // );
        ui.item_list.render(frame, list_and_info[0]);
        frame.render_widget(right_widget, list_and_info[1]);
        frame.render_widget(input_widget, io[1]);
    }

    async fn events(&mut self) -> Result<(), io::Error> {
        // let event_availalbe = event::poll(Duration::from_millis(10))?;
        // if !event_availalbe {
        //     return Ok(());
        // }
        // let event = event::read()?;
        // match event {
        //     event::Event::Key(key_event) => match key_event.code {
        //         event::KeyCode::Tab => self.ui.selected_element = self.ui.selected_element.next(),
        //         event::KeyCode::Char(c) => match c {
        //             'q' => self.running = false,
        //             c => match self.ui.selected_element {
        //                 UiElement::InputField => self.ui.input_field.push(c),
        //                 _ => (),
        //             },
        //         },
        //         event::KeyCode::Up => {
        //             self.ui.items.select_previous();
        //         }
        //         event::KeyCode::Down => self.ui.items.select_next(),
        //         event::KeyCode::Right => match self.ui.items.ui_state.selected() {
        //             Some(i) => {
        //                 let item = &self.ui.items.get()[i].clone();
        //                 let _ = self.goto(item).await;
        //             }
        //             None => (),
        //         },
        //         event::KeyCode::Left => self.ui.items.back(),
        //         event::KeyCode::Backspace => {
        //             self.ui.input_field.pop();
        //         }
        //         event::KeyCode::Enter => {
        //             let _ = self.issue_command().await;
        //         }
        //         _ => (),
        //     },
        //     _ => (),
        // };
        Ok(())
    }

    pub async fn listen(&mut self) -> Result<(), Box<dyn Error>> {
        // while self.running {
        //     self.events().await?;
        //     self.terminal
        //         .draw(|frame| Self::draw_ui(&mut self.ui, frame))
        //         .unwrap();
        // }
        // ratatui::restore();
        Ok(())
    }

    async fn issue_command(&mut self) -> Result<(), SessionError> {
        let params: String = self.ui.input_field.drain(..).collect();
        let params: Vec<_> = params.split(" ").collect();
        if params.is_empty() {
            return Ok(());
        }
        match params[0].to_lowercase().trim() {
            "search" | "s" => self.search(&params).await?,
            _ => (),
        }

        Ok(())
    }

    async fn search(&mut self, params: &[&str]) -> Result<(), SessionError> {
        if params.len() <= 1 {
            return Ok(());
        }
        let query = &params[1..].join(" ");
        let result = self
            .session
            .search(
                query,
                SearchType::Track | SearchType::Album | SearchType::Artist,
                15,
                0,
            )
            .await?;
        // self.ui.items.from_search(result);
        Ok(())
    }

    async fn play(&mut self, track: &Track) -> Result<(), SessionError> {
        let stream = self
            .session
            .get_track_stream(track, AudioQuality::HighLossless)
            .await?;
        let stream = tidal::media::Stream::init(stream, 1024 * 64, false)
            .await
            .unwrap();
        let _ = self.player_tx.send(PlayerCtl::Play(stream));
        Ok(())
    }

    async fn goto(&mut self, item: &Item) -> Result<(), SessionError> {
        match item {
            Item::Track(track) => self.play(track).await?,
            Item::Album(album) => {
                let tracks = self.session.get_album_tracks(album).await?;
                // let items = tracks.items.into_iter().map(|i| Item::Track(i)).collect();
                // self.ui.items.collapse();
                // self.ui.items.push(items);
            }
            Item::Artist(artist) => {
                let albums = self.session.get_artist_albums(artist).await?;
                // let items = albums.items.into_iter().map(|i| Item::Album(i)).collect();
                // self.ui.items.collapse();
                // self.ui.items.push(items);
            }
        }

        Ok(())
    }
}
