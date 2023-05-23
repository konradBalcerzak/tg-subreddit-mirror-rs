pub mod models;
pub mod schema;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use crate::settings::SETTINGS_INSTANCE;

pub fn establish_connection() -> SqliteConnection {
    let database_url = &SETTINGS_INSTANCE.database.url;
    SqliteConnection::establish(database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
