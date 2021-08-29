use anyhow::{anyhow, Result};
use egg_mode::{
    list::{List, ListID},
    tweet::Tweet,
};
use tokio::time::{sleep, Duration};

use crate::{ui::UI, MyConfig};

const AUTH_TOKENS_FILE: &'static str = ".twitter_cli_oauth";
//const CONSUMER_KEY: &'static str = include_str!("consumer_key.in");
//const CONSUMER_SECRET: &'static str = include_str!("consumer_secret.in");

pub(crate) struct Twitter {
    token: egg_mode::Token,
    user_id: u64,
    screen_name: String,
}

impl Twitter {
    pub(crate) async fn init(cfg: &MyConfig) -> Result<Self> {
        let (token, user_id, screen_name) = match fetch_login(&cfg).await {
            Ok((token, user_id, screen_name)) => (token, user_id, screen_name),
            Err(_) => login(&cfg).await?,
        };

        let standard_font = figlet_rs::FIGfont::standand().unwrap();
        let figure = standard_font.convert(&("@".to_string() + &screen_name));
        println!("{}", figure.unwrap());

        Ok(Twitter {
            token,
            user_id,
            screen_name,
        })
    }

    pub(crate) async fn me(&self, ui: &UI) -> Result<()> {
        // TODO: include likes
        // egg_mode::tweet::liked_by<T: Into<UserID>>(acct: T, token: &Token) -> Timeline
        let tweets = egg_mode::tweet::user_timeline(self.user_id, true, true, &self.token)
            .with_page_size(15);
        let (_tweets, feed) = tweets.start().await?;
        self.print_feed(&ui, feed.iter().rev()).await;
        Ok(())
    }

    pub(crate) async fn list(&self, ui: &UI, list_name: &str) -> Result<()> {
        let lists = egg_mode::list::list(self.user_id, true, &self.token).await?;
        let list: &List = (*lists)
            .iter()
            .find(|l| l.name == list_name)
            .ok_or(anyhow!("List not found"))?;
        let tweets = egg_mode::list::statuses(ListID::from_id(list.id), true, &self.token)
            .with_page_size(15);
        let (_tweets, feed) = tweets.start().await?;
        self.print_feed(&ui, feed.iter().rev()).await;
        Ok(())
    }

    // pub(crate) async fn home(&self, ui: &UI) -> Result<()> {
    //     let home = egg_mode::tweet::home_timeline(&self.token).with_page_size(15);
    //     let (_home, feed) = home.start().await?;
    //     self.print_feed(&ui, feed.iter().rev()).await;
    //     Ok(())
    // }

    pub(crate) async fn home_stream(&self, ui: &UI) -> Result<()> {
        let mut home = egg_mode::tweet::home_timeline(&self.token).with_page_size(15);
        let tmp = home.start().await?;
        home = tmp.0;
        let mut feed = tmp.1;
        self.print_feed(&ui, feed.iter().rev()).await;

        loop {
            let tmp = home.newer(None).await?;
            // TODO: handle twitter's backoff response properly
            home = tmp.0;
            feed = tmp.1;
            // let mut max_id = home.max_id;
            self.print_feed(&ui, feed.iter().rev()).await;
            sleep(Duration::from_millis(120000)).await;

            // timeline.reset()
            // let (home, _new_posts) = timeline.older(Some(max_id)).await.unwrap();
        }
        Ok(())
    }

    pub(crate) async fn print_feed<'a, I>(&self, ui: &UI, feed: I)
    where
        I: Iterator<Item = &'a Tweet>,
    {
        for status in feed {
            ui.print_tweet(status).await;
        }
    }
}

fn auth_tokens_file_path() -> std::path::PathBuf {
    let mut path = std::env::home_dir().unwrap();
    path.push(AUTH_TOKENS_FILE);
    path
}

async fn fetch_login(cfg: &MyConfig) -> Result<(egg_mode::auth::Token, u64, String)> {
    let file = std::fs::File::open(auth_tokens_file_path())?;
    let reader = std::io::BufReader::new(file);
    let access_token: egg_mode::auth::KeyPair = serde_json::from_reader(reader)?;

    let con_token = egg_mode::KeyPair::new(
        cfg.consumer_key.trim().to_string(),
        cfg.consumer_secret.trim().to_string(),
    );
    let token = egg_mode::Token::Access {
        consumer: con_token,
        access: access_token,
    };
    let user: &egg_mode::user::TwitterUser = &*egg_mode::auth::verify_tokens(&token).await?;
    // println!("Got user {:?}", &user);
    // TODO: if tokens don't verify, should probably delete the file
    // println!("We've hit an error using your old tokens: {:?}", err);
    // println!("We'll have to reauthenticate before continuing.");
    // std::fs::remove_file(auth_tokens_file_path()).unwrap();
    Ok((token, user.id, user.screen_name.to_string()))
}

async fn save_login(token: &egg_mode::auth::Token) -> Result<()> {
    if let egg_mode::Token::Access {
        access: tok,
        consumer: _,
    } = token
    {
        use std::fs::OpenOptions;
        let file = &OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(auth_tokens_file_path())?;
        serde_json::to_writer(file, tok)?;
    }
    Ok(())
}

// token can be given to any egg_mode method that asks for a token
// user_id and screen_name refer to the user who signed in
async fn login(cfg: &MyConfig) -> Result<(egg_mode::auth::Token, u64, String)> {
    let con_token = egg_mode::KeyPair::new(
        cfg.consumer_key.trim().to_string(),
        cfg.consumer_secret.trim().to_string(),
    );

    // "oob" is needed for PIN-based auth; see docs for `request_token` for more info
    println!("Fetching request token");
    let request_token = egg_mode::auth::request_token(&con_token, "oob")
        .await
        .unwrap();
    println!("Getting authorization URL");
    let auth_url = egg_mode::auth::authorize_url(&request_token);

    // give auth_url to the user, they can sign in to Twitter and accept your app's permissions.
    println!(
        "Please open {} and grant access, then paste the PIN back here:",
        &auth_url
    );

    // they'll receive a PIN in return, they need to give this to your application
    let mut verifier = String::new();
    std::io::stdin().read_line(&mut verifier)?;

    // note this consumes con_token; if you want to sign in multiple accounts, clone it here

    let (token, user_id, screen_name) =
        egg_mode::auth::access_token(con_token, &request_token, verifier).await?;

    // println!(
    // "Got token {:?}, id {}, name {}",
    // &token, &user_id, &screen_name
    // );

    save_login(&token).await?;

    Ok((token, user_id, screen_name))
}
