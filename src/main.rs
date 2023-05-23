mod db;
mod reddit_bot;
mod settings;
mod teloxide;

use crate::teloxide::setup_teloxide;
use db::establish_connection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use settings::SETTINGS_INSTANCE;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[tokio::main]
async fn main() {
    let reddit_bot = reddit_bot::setup_roux(&SETTINGS_INSTANCE.reddit)
        .await
        .expect("Couldn't instanciate Reddit API connection");
    let db = establish_connection();
    setup_teloxide(reddit_bot, db).await;
}
