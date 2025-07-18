use std::{error::Error, fs, io};

use tidal;

use crate::frontend::Frontend;

mod backend;
mod frontend;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let mut session = tidal::Session::new();

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

    let mut frontend = Frontend::new(session);
    frontend.listen().await?;

    save_auth(&frontend.session.oauth.unwrap())?;
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
