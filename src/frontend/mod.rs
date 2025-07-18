mod item;

use item::Items;
use ratatui::{DefaultTerminal, Frame, crossterm::event, text::Text};
use std::{
    error::Error,
    io::{self, Write},
    sync::mpsc::{Sender, channel},
};

use colored::Colorize;
use tidal::{SearchType, Session, SessionError, media::AudioQuality};

use crate::{
    backend::{self, PlayerCtl},
    frontend::item::Item,
};
pub struct Frontend {
    pub session: Session,
    player_tx: Sender<PlayerCtl>,
    terminal: DefaultTerminal,
    ui: Ui,
}

pub struct Ui {
    items: Items,
}

impl Frontend {
    pub fn new(session: Session) -> Self {
        let terminal = ratatui::init();
        let (player_tx, player_rx) = channel();
        tokio::task::spawn_blocking(move || backend::init(player_rx).unwrap());
        let items = Items::new();
        let ui = Ui { items };
        Self {
            session,
            player_tx,
            ui,
            terminal,
        }
    }

    fn draw_ui(ui: &Ui, frame: &mut Frame) {
        let text = Text::raw("test");
        frame.render_widget(text, frame.area());
    }

    fn events(&mut self) -> Result<bool, io::Error> {
        let event = event::read()?;
        match event {
            event::Event::Key(key_event) => match key_event.code {
                event::KeyCode::Char(c) => match c {
                    'q' => return Ok(true),
                    _ => (),
                },
                _ => (),
            },
            _ => (),
        };
        Ok(false)
    }

    pub async fn listen(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            // // let draw = move |frame| self.draw_ui(frame);
            // self.terminal.draw(|frame| self.draw_ui(frame));

            self.terminal
                .draw(|frame| Self::draw_ui(&self.ui, frame))
                .unwrap();

            if self.events()? {
                break;
            }

            // print!("{} ", ">".cyan());
            // let _ = io::stdout().flush();
            // let input = Self::input()?;
            // let params: Vec<_> = input.split(" ").collect();
            // // let params = split(&input).unwrap();
            // if params.is_empty() {
            //     continue;
            // }
            // match params[0].to_lowercase().trim() {
            //     "clear" => {
            //         clearscreen::ClearScreen::default()
            //             .clear()
            //             .expect("failed to clear the screen");
            //     }
            //     "exit" => break,
            //     "goto" | "g" => self.goto(&params).await?,
            //     "search" | "s" => self.search(&params).await?,
            //     "play" | "p" => self.play(&params).await?,
            //     "forward" | "f" => {
            //         self.ui.items.forward();
            //         self.ui.items.print();
            //     }
            //     "back" | "b" => {
            //         self.ui.items.back();
            //         self.ui.items.print();
            //     }
            //     _ => println!("invalid input"),
            // }
        }
        Ok(())
    }

    // fn input() -> Result<String, io::Error> {
    //     let mut input = String::new();
    //     io::stdin().read_line(&mut input)?;
    //     Ok(input)
    // }

    // async fn search(&mut self, params: &[&str]) -> Result<(), SessionError> {
    //     if params.len() <= 1 {
    //         return Ok(());
    //     }
    //     let query = &params[1..].join(" ");
    //     let result = self
    //         .session
    //         .search(
    //             query,
    //             SearchType::Track | SearchType::Album | SearchType::Artist,
    //             15,
    //             0,
    //         )
    //         .await?;
    //     self.ui.items.from_search(result);
    //     self.ui.items.print();
    //     Ok(())
    // }

    // async fn play(&mut self, params: &[&str]) -> Result<(), SessionError> {
    //     if params.len() != 2 {
    //         return Ok(());
    //     }
    //     let Ok(index) = params[1].trim().parse::<usize>() else {
    //         println!("failed to parse index");
    //         return Ok(());
    //     };
    //     if index >= self.ui.items.get().len() {
    //         println!("index too big");
    //         return Ok(());
    //     }
    //     let Frontend { session, .. } = self;
    //     let Item::Track(track) = &self.ui.items.get()[index] else {
    //         println!("invalid type");
    //         return Ok(());
    //     };
    //     let stream = session
    //         .get_track_stream(track, AudioQuality::HighLossless)
    //         .await?;
    //     let stream = tidal::media::Stream::init(stream, 1024 * 64, false)
    //         .await
    //         .unwrap();
    //     let _ = self.player_tx.send(PlayerCtl::Play(stream));
    //     Ok(())
    // }

    // async fn goto(&mut self, params: &[&str]) -> Result<(), SessionError> {
    //     if params.len() != 2 {
    //         return Ok(());
    //     }
    //     let Ok(index) = params[1].trim().parse::<usize>() else {
    //         println!("failed to parse index");
    //         return Ok(());
    //     };
    //     if index >= self.ui.items.get().len() {
    //         println!("index too big");
    //         return Ok(());
    //     }
    //     let item = &self.ui.items.get()[index];
    //     match item {
    //         Item::Track(_track) => {
    //             println!("only Artist and Album is supported to go to")
    //         }
    //         Item::Album(album) => {
    //             let tracks = self.session.get_album_tracks(album).await?;
    //             let items = tracks.items.into_iter().map(|i| Item::Track(i)).collect();
    //             self.ui.items.collapse();
    //             self.ui.items.push(items);
    //             self.ui.items.print();
    //         }
    //         Item::Artist(artist) => {
    //             let albums = self.session.get_artist_albums(artist).await?;
    //             let items = albums.items.into_iter().map(|i| Item::Album(i)).collect();
    //             self.ui.items.collapse();
    //             self.ui.items.push(items);
    //             self.ui.items.print();
    //         }
    //     }

    //     Ok(())
    // }
}
