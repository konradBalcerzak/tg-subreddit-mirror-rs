use super::{AppDialogue, Command, DispatcherSchema, State as SupState, TeloxideResult};
use crate::db::models::Channel;
use teloxide::prelude::*;

mod listeners {
    use diesel::SqliteConnection;
    use roux::Subreddit as SubredditApi;
    use std::sync::{Arc, Mutex};
    use teloxide::types::Me;

    use crate::{
        db::models::{ChannelSubreddit, NewChannelSubreddit, NewSubreddit, Subreddit},
        teloxide::{msg_reply, update_dialogue},
    };

    use super::*;
    pub(super) async fn on_sub_link(
        bot: Bot,
        dialogue: Dialogue<SupState, AppDialogue>,
        msg: Message,
        me: Me,
        conn: Arc<Mutex<SqliteConnection>>,
    ) -> TeloxideResult {
        use crate::teloxide::channel::helpers::{channel_list_message, get_channels_where_admins};

        let from_user = match msg.from() {
            Some(user) => user,
            None => return msg_reply("Couldn't recognize the user. Try again.", &bot, &msg).await,
        };
        let channels = get_channels_where_admins(&bot, conn, &from_user.id, &me.user.id).await?;
        if channels.is_empty() {
            return msg_reply(
                "No channels found. Try adding a new channel first",
                &bot,
                &msg,
            )
            .await;
        }
        msg_reply(
            format!(
                "Got it. Type the ID of the channel you want to link:\n\n{}",
                channel_list_message(channels)?
            ),
            &bot,
            &msg,
        )
        .await?;
        update_dialogue(&dialogue, SupState::Sub(State::LinkReceiveChannel)).await
    }

    pub(super) async fn on_sub_link_channel(
        bot: Bot,
        dialogue: Dialogue<SupState, AppDialogue>,
        msg: Message,
        conn: Arc<Mutex<SqliteConnection>>,
    ) -> TeloxideResult {
        use crate::db::schema::channel::dsl::*;
        use diesel::prelude::*;

        let channel_id: ChatId = match msg.text() {
            Some(text) => ChatId(text.parse()?),
            None => {
                return msg_reply(
                    "Please send a message with the id of the channel you wish to link",
                    &bot,
                    &msg,
                )
                .await;
            }
        };
        let found_channel: Result<Channel, _> = channel
            .filter(chat_id.eq(channel_id.0))
            .first(&mut *conn.lock().unwrap());
        let selected_channel = match found_channel {
            Ok(real_channel) => real_channel,
            Err(_) => {
                return msg_reply(
                    "Couldn't find the channel. Please send the of an already linked channel.",
                    &bot,
                    &msg,
                )
                .await;
            }
        };
        msg_reply(
            "Great. Now send the subreddit name (without the preceding /r/ part and without the trailing slashes).",
            &bot,
            &msg,
        )
        .await?;
        update_dialogue(
            &dialogue,
            SupState::Sub(State::LinkReceiveSub(selected_channel)),
        )
        .await
    }

    pub(super) async fn on_sub_link_sub(
        bot: Bot,
        dialogue: Dialogue<SupState, AppDialogue>,
        msg: Message,
        conn: Arc<Mutex<SqliteConnection>>,
        selected_channel: Channel,
    ) -> TeloxideResult {
        let sub_name = match msg.text() {
            Some(text) => text,
            None => return msg_reply("Please send a subreddit name.", &bot, &msg).await,
        };
        let sub_data = match SubredditApi::new(sub_name).about().await {
            Ok(sub_data) => sub_data,
            Err(error) => {
                return msg_reply(format!("Error: {}. Try again.", error), &bot, &msg).await
            }
        };
        let (sub_id, sub_name) = match (sub_data.id, sub_data.name) {
            (Some(id), Some(name)) => (id, name),
            _ => {
                return msg_reply(
                    "Error while fetching subreddit data. Try again.",
                    &bot,
                    &msg,
                )
                .await;
            }
        };
        let subreddit = match Subreddit::get_by_sub_id(&sub_id, &mut conn.lock().unwrap()) {
            Ok(db_subreddit) => Ok(db_subreddit),
            Err(_) => {
                let new_subreddit = NewSubreddit {
                    subreddit_id: sub_id.as_str(),
                    name: sub_name.as_str(),
                };
                new_subreddit.insert(&mut conn.lock().unwrap())
            }
        };
        let subreddit = match subreddit {
            Ok(subreddit) => subreddit,
            Err(_) => {
                return msg_reply(
                    "Error while trying to save the subreddit. Try again.",
                    &bot,
                    &msg,
                )
                .await
            }
        };
        let related_channels = ChannelSubreddit::are_related(
            &selected_channel,
            &subreddit,
            &mut conn.lock().unwrap(),
        )?;
        if !related_channels {
            ChannelSubreddit::insert(
                &NewChannelSubreddit::new(&selected_channel, &subreddit),
                &mut conn.lock().unwrap(),
            )?;
        }
        msg_reply("Subreddit successfully linked to the channel.", &bot, &msg).await?;
        update_dialogue(&dialogue, SupState::MainMenu).await
    }

    pub(super) async fn on_sub_unlink(
        bot: Bot,
        dialogue: Dialogue<SupState, AppDialogue>,
        msg: Message,
        conn: Arc<Mutex<SqliteConnection>>,
        me: Me,
    ) -> TeloxideResult {
        use crate::teloxide::channel::helpers::{channel_list_message, get_channels_where_admins};

        let from_user = match msg.from() {
            Some(user) => user,
            None => return msg_reply("Couldn't recognize the user. Try again.", &bot, &msg).await,
        };
        let channels = get_channels_where_admins(&bot, conn, &from_user.id, &me.user.id).await?;
        if channels.is_empty() {
            return msg_reply(
                "No channels found. Try adding a new channel first",
                &bot,
                &msg,
            )
            .await;
        }
        msg_reply(
            format!(
                "Got it. Type the ID of the channel you want to unlink subreddit from:\n\n{}",
                channel_list_message(channels)?
            ),
            &bot,
            &msg,
        )
        .await?;
        update_dialogue(&dialogue, SupState::Sub(State::UnlinkReceiveChannel)).await
    }

    pub(super) async fn on_sub_unlink_channel(
        bot: Bot,
        dialogue: Dialogue<SupState, AppDialogue>,
        msg: Message,
        conn: Arc<Mutex<SqliteConnection>>,
    ) -> TeloxideResult {
        let channel_id: ChatId = match msg.text() {
            Some(text) => ChatId(text.parse()?),
            None => {
                return msg_reply(
                    "Please send a message with the id of the channel you wish to unlink",
                    &bot,
                    &msg,
                )
                .await
            }
        };
        let channel = Channel::get_by_chat_id(channel_id, &mut conn.lock().unwrap());
        let selected_channel = match channel {
            Ok(real_channel) => real_channel,
            Err(_) => {
                return msg_reply(
                    "Couldn't find the channel. Please send the of an already linked channel.",
                    &bot,
                    &msg,
                )
                .await;
            }
        };

        msg_reply(
            "Great. Now send the subreddit name (without the preceding /r/ part and without the trailing slashes).",
            &bot,
            &msg,
        )
        .await?;
        update_dialogue(
            &dialogue,
            SupState::Sub(State::UnlinkReceiveSub(selected_channel)),
        )
        .await
    }

    pub(super) async fn on_sub_unlink_sub() -> TeloxideResult {
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) enum State {
    LinkReceiveChannel,
    LinkReceiveSub(Channel),
    UnlinkReceiveChannel,
    UnlinkReceiveSub(Channel),
}

pub fn schema() -> DispatcherSchema {
    use dptree::case;
    Update::filter_message()
        .branch(
            case![SupState::MainMenu]
                .filter_command::<Command>()
                .branch(case![Command::LinkSubreddit].endpoint(listeners::on_sub_link))
                .branch(case![Command::UnlinkSubreddit].endpoint(listeners::on_sub_unlink)),
        )
        .branch(
            case![SupState::Sub(x)]
                .branch(case![State::LinkReceiveChannel].endpoint(listeners::on_sub_link_channel))
                .branch(
                    case![State::LinkReceiveSub(selected_channel)]
                        .endpoint(listeners::on_sub_link_sub),
                )
                .branch(
                    case![State::UnlinkReceiveChannel].endpoint(listeners::on_sub_unlink_channel),
                )
                .branch(
                    case![State::LinkReceiveSub(selected_channel)]
                        .endpoint(listeners::on_sub_unlink_sub),
                ),
        )
}
