use super::schema::*;
use diesel::{
    deserialize::{FromSql, FromSqlRow},
    dsl::count,
    prelude::*,
    result::Error,
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

impl Channel {
    pub fn get_by_chat_id(chat_id: i64, conn: &mut SqliteConnection) -> Result<Channel, Error> {
        use crate::db::schema::channel::dsl as channel_dsl;
        channel_dsl::channel
            .filter(channel_dsl::chat_id.eq(&chat_id))
            .first::<Channel>(conn)
    }
    pub fn get_by_subreddit(
        related_subreddit: Subreddit,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<Channel>, Error> {
        use crate::db::schema::channel::dsl as channel_dsl;
        use diesel::prelude::*;
        ChannelSubreddit::belonging_to(&related_subreddit)
            .inner_join(channel_dsl::channel)
            .select(Channel::as_select())
            .load(conn)
    }
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

impl Subreddit {
    pub fn get_by_subreddit_id(
        subreddit_id: &String,
        conn: &mut SqliteConnection,
    ) -> Result<Subreddit, Error> {
        use crate::db::schema::subreddit::dsl as sub_dsl;
        sub_dsl::subreddit
            .filter(sub_dsl::subreddit_id.eq(&subreddit_id))
            .first::<Subreddit>(conn)
    }
    pub fn get_by_channel_id(
        related_channel: Channel,
        conn: &mut SqliteConnection,
    ) -> Result<Vec<Subreddit>, Error> {
        use crate::db::schema::subreddit::dsl as sub_dsl;
        use diesel::prelude::*;
        ChannelSubreddit::belonging_to(&related_channel)
            .inner_join(sub_dsl::subreddit)
            .select(Subreddit::as_select())
            .load(conn)
    }
}

#[derive(Insertable)]
#[diesel(table_name = subreddit)]
pub struct NewSubreddit<'a> {
    pub subreddit_id: &'a str,
    pub name: &'a str,
}

impl<'a> NewSubreddit<'a> {
    pub fn insert(self, conn: &mut SqliteConnection) -> QueryResult<Subreddit> {
        use crate::db::schema::subreddit::dsl::*;
        diesel::insert_into(subreddit).values(&self).execute(conn)?;
        subreddit.order(id.desc()).first::<Subreddit>(conn)
    }
}

#[derive(Identifiable, Selectable, Queryable, Associations, Debug)]
#[diesel(belongs_to(Subreddit))]
#[diesel(belongs_to(Channel))]
#[diesel(table_name = channel_subreddit)]
pub struct ChannelSubreddit {
    pub id: Option<i32>,
    pub channel_id: i32,
    pub subreddit_id: i32,
}

impl ChannelSubreddit {
    pub fn are_related(
        channel: &Channel,
        subreddit: &Subreddit,
        conn: &mut SqliteConnection,
    ) -> Result<bool, Error> {
        use crate::db::schema::{
            channel::dsl as channel_dsl, channel_subreddit::dsl as channel_sub_dsl,
            subreddit::dsl as sub_dsl,
        };
        channel_sub_dsl::channel_subreddit
            .inner_join(channel_dsl::channel)
            .inner_join(sub_dsl::subreddit)
            .select(count(channel_dsl::chat_id))
            .filter(channel_dsl::chat_id.eq(channel.chat_id))
            .filter(sub_dsl::subreddit_id.eq(&subreddit.subreddit_id))
            .first::<i64>(conn)
            .map(|count| count > 0)
    }
}

impl ChannelSubreddit {
    pub fn insert(
        new_relation: &NewChannelSubreddit,
        conn: &mut SqliteConnection,
    ) -> Result<ChannelSubreddit, Error> {
        use crate::db::schema::channel_subreddit::dsl as channel_sub_dsl;
        diesel::insert_into(channel_sub_dsl::channel_subreddit)
            .values(new_relation)
            .execute(conn)?;
        channel_sub_dsl::channel_subreddit
            .order(channel_sub_dsl::id.desc())
            .first::<ChannelSubreddit>(conn)
    }
}

#[derive(Insertable)]
#[diesel(table_name = channel_subreddit)]
pub struct NewChannelSubreddit {
    channel_id: i32,
    subreddit_id: i32,
}

impl NewChannelSubreddit {
    pub fn new(channel: &Channel, subreddit: &Subreddit) -> Self {
        NewChannelSubreddit {
            channel_id: channel.id,
            subreddit_id: subreddit.id,
        }
    }
}
