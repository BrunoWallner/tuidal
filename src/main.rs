use shell_words::split;
use std::io::Write;
use std::sync::mpsc::channel;
use std::{error::Error, fs, io};
use tidal::media::{Album, Track};

use tidal::SearchType;
use tidal::{self};

use crate::player::PlayerCtl;
use colored::Colorize;

mod audio;
mod player;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let config = tidal::Config::new(
        tidal::media::AudioQuality::HighLossless,
        tidal::media::VideoQuality::AudioOnly,
        tidal::ItemLimit::try_from(100).unwrap(),
        false,
    );
    let mut session = tidal::Session::new(config);

    if let Some(oauth) = get_auth() {
        session.set_oauth(oauth);
    } else {
        session
            .oauth_login_simple(|link| println!("https://{}", link.verification_uri))
            .await?;
        if let Some(oauth) = &session.oauth {
            let _ = save_auth(oauth);
        }
        println!("logged in");
    }
    session.load_session_info().await?;

    listen(&mut session).await?;

    save_auth(&session.oauth.unwrap())?;
    Ok(())
}

fn get_auth() -> Option<tidal::auth::OAuth> {
    let string = fs::read_to_string("./auth.json").ok()?;
    serde_json::from_str(&string).ok()
}

fn save_auth(auth: &tidal::auth::OAuth) -> Result<(), io::Error> {
    let string = serde_json::to_string(auth)?;
    fs::write("./auth.json", string)
}

enum Item {
    Track(Track),
    Album(Album),
}

async fn listen(session: &mut tidal::Session) -> Result<(), Box<dyn Error>> {
    let mut items: Vec<Item> = Vec::new();
    let (player_tx, player_rx) = channel();
    tokio::task::spawn_blocking(move || player::init(player_rx).unwrap());
    loop {
        print!("{} ", ">".cyan());
        let _ = io::stdout().flush();
        let input = input()?;
        // let params: Vec<_> = input.split(" ").collect();
        let params = split(&input).unwrap();
        if params.is_empty() {
            continue;
        }
        match params[0].to_lowercase().trim() {
            "clear" => {
                clearscreen::ClearScreen::default()
                    .clear()
                    .expect("failed to clear the screen");
            }
            "exit" => break,
            "goto" | "g" => {
                if params.len() != 2 {
                    continue;
                }
                let Ok(index) = params[1].trim().parse::<usize>() else {
                    println!("failed to parse index");
                    continue;
                };
                if index >= items.len() {
                    println!("index too big");
                    continue;
                }
                let Item::Album(album) = &items[index] else {
                    println!("invalid type");
                    continue;
                };
                let tracks = session.get_album_tracks(album).await?;
                items.clear();
                for track in tracks.items {
                    items.push(Item::Track(track))
                }
                print_items(&items);
            }
            "search" | "s" => {
                if params.len() <= 1 {
                    continue;
                }
                let query = &params[1..].join(" ");
                let result = session
                    .search(query, SearchType::Track | SearchType::Album, 15, 0)
                    .await
                    .unwrap();
                items = to_items(result);
                // let _ = ClearScreen::default().clear();
                print_items(&items);
            }
            "play" | "p" => {
                if params.len() != 2 {
                    continue;
                }
                let Ok(index) = params[1].trim().parse::<usize>() else {
                    println!("failed to parse index");
                    continue;
                };
                if index >= items.len() {
                    println!("index too big");
                    continue;
                }
                let Item::Track(track) = &items[index] else {
                    println!("invalid type");
                    continue;
                };
                let stream = session.get_track_stream(track).await?;
                let stream = tidal::media::Stream::init(stream, 1024 * 64, false)
                    .await
                    .unwrap();
                let _ = player_tx.send(PlayerCtl::Play(stream));
            }
            _ => println!("invalid input"),
        }
    }
    Ok(())
}

fn print_items(items: &[Item]) {
    clearscreen::ClearScreen::default()
        .clear()
        .expect("failed to clear the screen");
    let mut formatted = Vec::new();
    for item in items.iter() {
        let artist = match item {
            Item::Track(track) => track
                .artists
                .get(0)
                .map(|artist| artist.name.as_str())
                .unwrap_or("?"),
            Item::Album(album) => album
                .artists
                .get(0)
                .map(|artist| artist.name.as_str())
                .unwrap_or("?"),
        };
        let mut title = match item {
            Item::Track(track) => track.title.as_str(),
            Item::Album(album) => album.title.as_str(),
        };
        if title.chars().count() > 38 {
            title = &title[..38]
        }
        let kind = match item {
            Item::Track(_) => "track",
            Item::Album(_) => "album",
        };
        formatted.push((kind, artist, title));
    }
    for (i, (kind, artist, title)) in formatted.iter().enumerate() {
        let mut artist = *artist;
        let kind_color = match *kind {
            "album" => "red",
            "track" => "green",
            _ => "white",
        };
        let kind = format!("[{}]", kind);
        if artist.chars().count() > 15 {
            artist = &artist[0..15];
        }

        let artist = format!("{}", artist);
        println!(
            "{i:<3} {:<5} {:<15} |{:<38}\t",
            kind.color(kind_color),
            artist.cyan(),
            title,
        );
    }
}

fn to_items(result: tidal::SearchResult) -> Vec<Item> {
    let mut output = Vec::new();
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
    output
}

fn input() -> Result<String, io::Error> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input)
}
