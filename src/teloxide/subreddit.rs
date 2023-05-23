use super::{AppDialogue, Command, DispatcherSchema, State as SupState, TeloxideResult};
use crate::db::models::Channel;
use teloxide::{dptree::case, prelude::*};

mod listeners {
    use diesel::SqliteConnection;
    use roux::Subreddit as SubredditApi;
    use std::sync::{Arc, Mutex};
    use teloxide::types::Me;

    use crate::db::models::{ChannelSubreddit, NewChannelSubreddit, NewSubreddit, Subreddit};

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
            None => {
                bot.send_message(msg.chat.id, "Couldn't recognize the user. Try again.")
                    .await?;
                return Ok(());
            }
        };
        let channels = get_channels_where_admins(&bot, conn, &from_user.id, &me.user.id).await?;
        if channels.is_empty() {
            bot.send_message(
                msg.chat.id,
                "No channels found. Try adding a new channel first",
            )
            .await?;
            return Ok(());
        }
        let respond_message =
            String::from("Got it. Type the ID of the channel you want to unkink:\n\n");
        bot.send_message(
            msg.chat.id,
            respond_message + &channel_list_message(channels)?,
        )
        .reply_to_message_id(msg.id)
        .await?;
        dialogue
            .update(SupState::Sub(State::SubLinkRecieveChannel))
            .await?;
        Ok(())
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
                bot.send_message(
                    msg.chat.id,
                    "Please send a message with the id of the channel you wish to unlink",
                )
                .reply_to_message_id(msg.id)
                .await?;
                return Ok(());
            }
        };
        let found_channel: Result<Channel, _> = channel
            .filter(chat_id.eq(channel_id.0))
            .first(&mut *conn.lock().unwrap());
        let selected_channel = match found_channel {
            Ok(real_channel) => real_channel,
            Err(_) => {
                bot.send_message(
                    msg.chat.id,
                    "Couldn't find the channel. Please send the of an already linked channel.",
                )
                .reply_to_message_id(msg.id)
                .await?;
                return Ok(());
            }
        };
        bot.send_message(
            msg.chat.id,
            "Great. Now send the subreddit name (without the preceding /r/ part and without the trailing slashes).",
        )
        .reply_to_message_id(msg.id)
        .await?;
        dialogue
            .update(SupState::Sub(State::SubLinkRecieveSub(selected_channel)))
            .await?;
        Ok(())
    }

    pub(super) async fn on_sub_link_sub(
        bot: Bot,
        dialogue: Dialogue<SupState, AppDialogue>,
        msg: Message,
        conn: Arc<Mutex<SqliteConnection>>,
        selected_channel: Channel,
    ) -> TeloxideResult {
        let subreddit_name = match msg.text() {
            Some(text) => text,
            None => {
                bot.send_message(msg.chat.id, "Please send a subreddit name.")
                    .await?;
                return Ok(());
            }
        };
        let subreddit_data = match SubredditApi::new(subreddit_name).about().await {
            Ok(subreddit_data) => subreddit_data,
            Err(error) => {
                bot.send_message(
                    msg.chat.id,
                    format!("Error: {}. Try again.", error.to_string()),
                )
                .await?;
                return Ok(());
            }
        };
        let (subreddit_id, subreddit_name) = match (subreddit_data.id, subreddit_data.name) {
            (Some(id), Some(name)) => (id, name),
            _ => {
                bot.send_message(
                    msg.chat.id,
                    "Encountered an error fetching subreddit data. Try again.",
                )
                .await?;
                return Ok(());
            }
        };
        let db_subreddit =
            match Subreddit::get_by_subreddit_id(&subreddit_id, &mut *conn.lock().unwrap()) {
                Ok(db_subreddit) => Ok(db_subreddit),
                Err(_) => {
                    let new_subreddit = NewSubreddit {
                        subreddit_id: &subreddit_id.as_str(),
                        name: &subreddit_name.as_str(),
                    };
                    new_subreddit.insert(&mut *conn.lock().unwrap())
                }
            };
        let db_subreddit = match db_subreddit {
            Ok(db_subreddit) => db_subreddit,
            Err(_) => {
                bot.send_message(
                    msg.chat.id,
                    "Error while trying to save the subreddit. Try again.",
                )
                .await?;
                return Ok(());
            }
        };
        let related_channels = ChannelSubreddit::are_related(
            &selected_channel,
            &db_subreddit,
            &mut *conn.lock().unwrap(),
        )?;
        if !related_channels {
            ChannelSubreddit::insert(
                &NewChannelSubreddit::new(&selected_channel, &db_subreddit),
                &mut *conn.lock().unwrap(),
            )?;
        }
        bot.send_message(msg.chat.id, "Subreddit successfully linked to the channel.")
            .await?;
        dialogue.update(SupState::MainMenu).await?;
        Ok(())
    }

    pub(super) async fn on_sub_unlink() -> TeloxideResult {
        Ok(())
    }

    pub(super) async fn on_sub_unlink_channel() -> TeloxideResult {
        Ok(())
    }

    pub(super) async fn on_sub_unlink_sub() -> TeloxideResult {
        Ok(())
    }
}

#[derive(Clone)]
pub(super) enum State {
    SubLinkRecieveChannel,
    SubLinkRecieveSub(Channel),
    SubUnlinkRecieveChannel,
    SubUnlinkRecieveSub(Channel),
}

pub fn schema() -> DispatcherSchema {
    Update::filter_message()
        .branch(
            case![SupState::MainMenu]
                .filter_command::<Command>()
                .branch(case![Command::LinkSubreddit].endpoint(listeners::on_sub_link))
                .branch(case![Command::UnlinkSubreddit].endpoint(listeners::on_sub_unlink)),
        )
        .branch(
            case![SupState::Sub(x)]
                .branch(
                    case![State::SubLinkRecieveChannel].endpoint(listeners::on_sub_link_channel),
                )
                .branch(
                    case![State::SubLinkRecieveSub(selected_channel)]
                        .endpoint(listeners::on_sub_link_sub),
                )
                .branch(
                    case![State::SubUnlinkRecieveChannel]
                        .endpoint(listeners::on_sub_unlink_channel),
                )
                .branch(
                    case![State::SubLinkRecieveSub(selected_channel)]
                        .endpoint(listeners::on_sub_unlink_sub),
                ),
        )
}
