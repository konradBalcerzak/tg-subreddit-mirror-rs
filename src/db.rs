pub mod models;
pub mod schema;

use diesel::sqlite::SqliteConnection;
use diesel::prelude::*;
use teloxide::types::ChatId;

use crate::settings::SETTINGS_INSTANCE;

use self::models::{Channel, NewChannel};

pub fn establish_connection() -> SqliteConnection {

    let database_url = &SETTINGS_INSTANCE.database.url;
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn insert_channel(conn: &mut SqliteConnection, chat_id: ChatId, title: &str, username: Option<&str>, invite_link: Option<&str>) -> QueryResult<Channel> {
    use schema::channel::dsl;
    let new_channel = NewChannel {
        chat_id: chat_id.0,
        title,
        username,
        invite_link,
    };
    diesel::insert_into(dsl::channel)
        .values(&new_channel)
        .execute(conn)?;
    dsl::channel.order(dsl::id.desc()).first(conn)
}
