use std::fmt::Display;

use colored::Colorize;
use tidal::media::{Album, Artist, Track};

#[derive(Clone, Debug)]
pub enum Item {
    Track(Track),
    Album(Album),
    Artist(Artist),
}
impl Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Item::Track(track) => {
                let artist = track
                    .artists
                    .get(0)
                    .map(|a| a.name.chars().take(14).collect::<String>())
                    .unwrap_or_else(|| "?".to_string());
                let title = track.title.chars().take(35).collect::<String>();

                write!(f, "{:<9} {:<14} | {:<35}", "[track]".green(), artist, title)
            }
            Item::Album(album) => {
                let artist = album
                    .artists
                    .get(0)
                    .map(|a| a.name.chars().take(14).collect::<String>())
                    .unwrap_or_else(|| "?".to_string());
                let title = album.title.chars().take(35).collect::<String>();

                write!(f, "{:<9} {:<14} | {:<35}", "[album]".red(), artist, title)
            }
            Item::Artist(artist) => {
                let artist = artist.name.chars().take(40).collect::<String>();

                write!(f, "{:<9} {:<40}", "[artist]".magenta(), artist)
            }
        }
    }
}
pub struct Items {
    item_history: Vec<Vec<Item>>,
    time: usize,
}
impl Items {
    pub fn new() -> Self {
        Self {
            item_history: vec![vec![]],
            time: 0,
        }
    }
    pub fn print(&mut self) {
        let _ = clearscreen::ClearScreen::default().clear();
        for (i, item) in self.get().iter().enumerate() {
            println!("{i:>3}: {}", item);
        }
    }
    pub fn collapse(&mut self) {
        self.item_history.drain((self.time + 1)..);
    }
    pub fn get(&self) -> &Vec<Item> {
        &self.item_history[self.time]
    }
    pub fn push(&mut self, items: Vec<Item>) {
        self.item_history.push(items);
        self.time = self.item_history.len() - 1;
    }
    pub fn forward(&mut self) {
        if self.time < self.item_history.len() - 1 {
            self.time += 1;
        }
    }
    pub fn back(&mut self) {
        if self.time >= 1 {
            self.time -= 1;
        }
    }

    pub fn from_search(&mut self, result: tidal::SearchResult) {
        let mut output = Vec::new();
        if let Some(albums) = result.albums {
            for album in albums.items {
                output.push(Item::Album(album))
            }
        }
        if let Some(artists) = result.artists {
            for artist in artists.items {
                output.push(Item::Artist(artist))
            }
        }
        if let Some(tracks) = result.tracks {
            for track in tracks.items {
                output.push(Item::Track(track))
            }
        }
        // self.items = output;
        self.push(output);
    }
}
