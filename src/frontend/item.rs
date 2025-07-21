use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::ListState,
};
use tidal::media::{Album, Artist, Track};

#[derive(Clone, Debug)]
pub enum Item {
    Track(Track),
    Album(Album),
    Artist(Artist),
}
// impl Display for Item {
impl Item {
    pub fn to_line(&self) -> Line {
        match self {
            Item::Track(track) => {
                let artist = track
                    .artists
                    .get(0)
                    .map(|a| a.name.chars().take(14).collect::<String>())
                    .unwrap_or_else(|| "?".to_string());
                let title = track.title.chars().take(35).collect::<String>();
                Line::from(vec![
                    Span::styled(
                        format!("{:<9}", "[track]"),
                        Style::default().fg(Color::Green),
                    ),
                    Span::styled(format!("{artist:<14}"), Style::default().fg(Color::Magenta)),
                    Span::raw(" "),
                    Span::styled(format!("{title:<35}"), Style::default()),
                ])
            }
            Item::Album(album) => {
                let artist = album
                    .artists
                    .get(0)
                    .map(|a| a.name.chars().take(14).collect::<String>())
                    .unwrap_or_else(|| "?".to_string());
                let title = album.title.chars().take(35).collect::<String>();

                // write!(f, "{:<9} {:<14} | {:<35}", "[album]".red(), artist, title)
                Line::from(vec![
                    Span::styled(
                        format!("{:<9}", "[album]"),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::styled(format!("{artist:<14}"), Style::default().fg(Color::Magenta)),
                    Span::raw(" "),
                    Span::styled(format!("{title:<35}"), Style::default()),
                ])
            }
            Item::Artist(artist) => {
                let artist = artist.name.chars().take(40).collect::<String>();

                // write!(f, "{:<9} {:<40}", "[artist]".magenta(), artist)
                Line::from(vec![
                    Span::styled(
                        format!("{:<9}", "[artist]"),
                        Style::default().fg(Color::Magenta),
                    ),
                    Span::styled(format!("{artist:<40}"), Style::default()),
                ])
                // todo!()
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
            // is_dirty: false,
            time: 0,
            // ui_state: ListState::default(),
        }
    }

    // Move selection up
    // pub fn select_previous(&mut self) {
    //     let i = match self.ui_state.selected() {
    //         Some(i) if i > 0 => i - 1,
    //         _ => 0,
    //     };
    //     self.ui_state.select(Some(i));
    // }

    // // Move selection down
    // pub fn select_next(&mut self) {
    //     let i = match self.ui_state.selected() {
    //         Some(i) if i + 1 < self.get().len() => i + 1,
    //         Some(i) => i,
    //         None => 0,
    //     };
    //     self.ui_state.select(Some(i));
    // }

    pub fn collapse(&mut self) {
        self.item_history.drain((self.time + 1)..);
    }
    pub fn get(&self) -> &Vec<Item> {
        &self.item_history[self.time]
    }
    pub fn push(&mut self, items: Vec<Item>) {
        self.item_history.push(items);
        self.time = self.item_history.len() - 1;
        // self.apply_selection();
    }

    pub fn back(&mut self) {
        if self.time >= 1 {
            self.time -= 1;
            self.item_history.pop();
            // self.apply_selection();
        }
    }

    pub fn from_search(&mut self, result: tidal::SearchResult) {
        let mut output = Vec::new();
        if let Some(artists) = result.artists {
            for artist in artists.items {
                output.push(Item::Artist(artist))
            }
        }
        if let Some(albums) = result.albums {
            for album in albums.items {
                output.push(Item::Album(album))
            }
        }
        if let Some(tracks) = result.tracks {
            for track in tracks.items {
                output.push(Item::Track(track))
            }
        }
        self.push(output);
    }
}
