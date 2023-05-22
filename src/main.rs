mod db;
mod redditBot;
mod settings;
mod teloxide;

use crate::teloxide::setup_teloxide;
use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use settings::SETTINGS_INSTANCE;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[tokio::main]
async fn main() {
    let reddit_bot = redditBot::setup_roux(&SETTINGS_INSTANCE.reddit);
    setup_teloxide(reddit_bot).await;
}
