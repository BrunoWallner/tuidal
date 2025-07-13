use clearscreen::ClearScreen;
use shell_words::split;
use std::io::Write;
use std::{error::Error, fs, io};
use tidal::media::Track;

use tidal::SearchType;
use tidal::{self};

mod play;

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

async fn listen(session: &mut tidal::Session) -> Result<(), Box<dyn Error>> {
    let mut tracks: Vec<Track> = Vec::new();
    loop {
        print!("> ");
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
            "search" => {
                if params.len() <= 1 {
                    continue;
                }
                let query = &params[1..].join(" ");
                let result = session
                    .search(query, SearchType::Track, 15, 0)
                    .await
                    .unwrap();
                if let Some(t) = result.tracks {
                    let _ = ClearScreen::default().clear();
                    tracks = t.items;
                    print_tracks(&tracks);
                }
            }
            "stream" => {
                if params.len() != 2 {
                    continue;
                }
                let Ok(index) = params[1].trim().parse::<usize>() else {
                    println!("failed to parse index");
                    continue;
                };
                if index >= tracks.len() {
                    println!("index too big");
                    continue;
                }
                let track = &tracks[index];
                let stream = session.get_track_stream(track).await?;
                // let stream = session.get_track_stream_url(track).await?;
                let stream = tidal::media::Stream::init(stream, 1024 * 64).await.unwrap();
                tokio::task::spawn_blocking(move || {
                    play::stream(stream).unwrap();
                    println!("ending...");
                });
            }
            _ => println!("invalid input"),
        }
    }
    println!("exiting");
    Ok(())
}

fn print_tracks(tracks: &[Track]) {
    for (i, track) in tracks.iter().enumerate() {
        let artist = track
            .artists
            .get(0)
            .map(|artist| artist.name.as_str())
            .unwrap_or("?");
        let title = if track.title.chars().count() <= 46 {
            &track.title
        } else {
            &track.title[..46]
        };
        println!("{i:<3}: {:<48}\t | {}", title, artist);
    }
}

fn input() -> Result<String, io::Error> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input)
}
