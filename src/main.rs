mod settings;
mod db;
mod models;
mod schema;

use db::establish_connection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations};
// use roux::Reddit;
use settings::SETTINGS_INSTANCE;
// use teloxide::prelude::*;

pub const MIGRATIONS: EmbeddedMigrations  = embed_migrations!();

#[tokio::main]
async fn main() {
    let app_settings = &SETTINGS_INSTANCE;
    let db = establish_connection();
}
