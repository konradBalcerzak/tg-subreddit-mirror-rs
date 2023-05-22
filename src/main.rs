mod db;
mod settings;
mod teloxide;

use crate::teloxide::setup_teloxide;
use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use roux::Reddit;
use settings::SETTINGS_INSTANCE;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[tokio::main]
async fn main() {
    setup_teloxide().await;
}
