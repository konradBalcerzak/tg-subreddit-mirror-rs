use diesel::prelude::*;
use crate::schema::*;

#[derive(Queryable, Identifiable)]
#[diesel(table_name = channel)]
pub struct Channel {
    pub id: i32,
    pub disabled: bool,
    pub title: String,
    pub username: Option<String>,
    pub invite_link: Option<String>,
}

#[derive(Queryable, Identifiable)]
#[diesel(table_name = subreddit)]
pub struct Subreddit {
    pub id: i32,
    pub disabled: bool,
    pub subreddit_id: String,
    pub name: String,
    pub sorting: String,
    pub post_limit: Option<i32>,
    pub respect_external_content_flag: Option<bool>,
    pub min_score: Option<i32>,
    pub allow_nsfw: Option<bool>,
    pub show_spoilers: Option<bool>,
    pub medias_only: Option<bool>,
    pub users_blacklist: Option<String>,
}

#[derive(Identifiable, Selectable, Queryable, Associations, Debug)]
#[diesel(belongs_to(Channel))]
#[diesel(belongs_to(Subreddit))]
#[diesel(table_name = channel_subreddit)]
pub struct ChannelSubreddit {
    pub id: i32,
    pub channel_id: i32,
    pub subreddit_id: i32,
}
