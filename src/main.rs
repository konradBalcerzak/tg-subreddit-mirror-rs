mod settings;
mod db;
mod teloxide;

use diesel_migrations::{embed_migrations, EmbeddedMigrations};
// use roux::Reddit;
use crate::teloxide::setup_teloxide;

pub const MIGRATIONS: EmbeddedMigrations  = embed_migrations!();

#[tokio::main]
async fn main() {
    setup_teloxide().await;
}