use super::schema::*;
use diesel::{
    deserialize::{FromSql, FromSqlRow},
    prelude::*,
    sql_types::{self, Text},
    sqlite::{Sqlite, SqliteValue},
};

#[derive(Queryable, Selectable, Identifiable, Clone)]
#[diesel(table_name = channel)]
pub struct Channel {
    pub id: i32,
    pub chat_id: i64,
    pub disabled: bool,
    pub title: String,
    pub username: Option<String>,
    pub invite_link: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = channel)]
pub struct NewChannel<'a> {
    pub chat_id: i64,
    pub title: &'a str,
    pub username: Option<&'a str>,
    pub invite_link: Option<&'a str>,
}

impl<'a> NewChannel<'a> {
    pub fn insert(self, conn: &mut SqliteConnection) -> QueryResult<Channel> {
        use crate::db::schema::channel::dsl::*;
        diesel::insert_into(channel).values(&self).execute(conn)?;
        channel.order(id.desc()).first(conn)
    }
}

#[derive(Clone, FromSqlRow)]
pub enum SortType {
    Hot,
    Rising,
    Top,
    Latest,
}

impl FromSql<sql_types::Text, Sqlite> for SortType
where
    String: FromSql<sql_types::Text, Sqlite>,
{
    fn from_sql(value: SqliteValue) -> diesel::deserialize::Result<Self> {
        let value = <String as FromSql<Text, Sqlite>>::from_sql(value)?
            .trim()
            .to_lowercase();
        Ok(match value.as_str() {
            "hot" => SortType::Hot,
            "rising" => SortType::Hot,
            "top" => SortType::Hot,
            "latest" => SortType::Hot,
            _ => return Err("Encountered unexpected subreddit sort state in database.".into()),
        })
    }
}
#[derive(Queryable, Selectable, Identifiable, Clone)]
#[diesel(table_name = subreddit)]
pub struct Subreddit {
    pub id: i32,
    pub disabled: bool,
    pub subreddit_id: String,
    pub name: String,
    pub sorting: SortType,
    pub post_limit: Option<i32>,
    pub respect_external_content_flag: bool,
    pub min_score: Option<i32>,
    pub allow_nsfw: bool,
    pub show_spoilers: bool,
    pub medias_only: bool,
}

#[derive(Insertable)]
#[diesel(table_name = subreddit)]
pub struct NewSubreddit<'a> {
    subreddit_id: &'a str,
    name: &'a str,
}

impl<'a> NewSubreddit<'a> {
    pub fn insert(self, conn: &mut SqliteConnection) -> QueryResult<Subreddit> {
        use crate::db::schema::subreddit::dsl::*;
        diesel::insert_into(subreddit).values(&self).execute(conn)?;
        subreddit.order(id.desc()).first::<Subreddit>(conn)
    }
}

#[derive(Identifiable, Selectable, Queryable, Associations, Debug)]
#[diesel(belongs_to(Channel))]
#[diesel(belongs_to(Subreddit))]
#[diesel(table_name = channel_subreddit)]
pub struct ChannelSubreddit {
    pub id: Option<i32>,
    pub channel_id: i32,
    pub subreddit_id: i32,
}
