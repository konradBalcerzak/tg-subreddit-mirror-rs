use crate::db::models::Channel;

use super::DispatcherSchema;
use diesel::SqliteConnection;
use std::sync::{Arc, Mutex};
use teloxide::prelude::*;

pub mod helpers {
    use super::*;
    use std::error::Error;

    pub(crate) fn channel_list_message(
        channels: Vec<Channel>,
    ) -> Result<String, Box<dyn Error + Send + Sync + 'static>> {
        let mut message_content = String::new();
        for available_channel in channels {
            message_content += format!(
                "Channel name: {}\nChannel id: {}\n\n",
                available_channel.title, available_channel.chat_id
            )
            .as_str();
        }
        Ok(message_content)
    }

    pub(crate) async fn get_channels_where_admins(
        bot: &Bot,
        conn: Arc<Mutex<SqliteConnection>>,
        user_id: &UserId,
        bot_id: &UserId,
    ) -> Result<Vec<Channel>, Box<dyn Error + Send + Sync>> {
        use crate::db::schema::channel::dsl::*;
        use diesel::prelude::*;
        let linked_channel_ids = channel
            .select(chat_id)
            .distinct()
            .load::<i64>(&mut *conn.lock().unwrap())?;
        let mut available_channels: Vec<i64> = Vec::with_capacity(linked_channel_ids.capacity());
        for channel_id in linked_channel_ids {
            let admins: Vec<_> = bot
                .get_chat_administrators(ChatId(channel_id))
                .await?
                .iter()
                .map(|admin| admin.user.id)
                .collect();
            if admins.contains(user_id) && admins.contains(bot_id) {
                available_channels.push(channel_id);
            }
        }
        channel
            .filter(chat_id.eq_any(available_channels))
            .load::<Channel>(&mut *conn.lock().unwrap())
            .map_err(|x| x.into())
    }
}

mod listeners {
    use super::*;
    use crate::{
        db::models::NewChannel,
        teloxide::{msg_reply, update_dialogue, AppDialogue, State as SupState, TeloxideResult},
    };
    use teloxide::{
        types::{Me, Message},
        Bot,
    };

    pub(super) async fn on_channel_link(
        bot: Bot,
        dialogue: Dialogue<SupState, AppDialogue>,
        msg: Message,
    ) -> TeloxideResult {
        msg_reply("Got it. Forward a message from the channel here.\n Remember that this bot needs to be an administrator in that channel first.", &bot, &msg).await?;
        update_dialogue(&dialogue, SupState::Channel(State::LinkReceiveChannel)).await
    }

    pub(super) async fn on_channel_link_msg(
        bot: Bot,
        dialogue: Dialogue<SupState, AppDialogue>,
        msg: Message,
        me: Me,
        conn: Arc<Mutex<SqliteConnection>>,
    ) -> TeloxideResult {
        let forward_chat = match msg.forward_from_chat() {
            Some(chat) => chat,
            None => {
                return msg_reply(
                    format!(
                        "This message is not a forward. Try again or use command /cancel@{}",
                        me.username()
                    ),
                    &bot,
                    &msg,
                )
                .await
            }
        };
        if !forward_chat.is_channel() {
            return msg_reply(
                    format!(
                        "This message is not forwarded from a channel. Try again or use command /cancel@{}",
                        me.username()
                    ),
                    &bot,
                    &msg,
                )
                .await;
        }
        let chat_admins = bot.get_chat_administrators(forward_chat.id).await?;
        if !chat_admins.iter().any(|admin| admin.user.id == me.id) {
            return msg_reply(
                    format!("This bot is not an administrator in this channel. Try again or use command /cancel@{}", me.username()),
                    &bot,
                    &msg,
                )
                .await;
        }
        let new_channel: NewChannel = NewChannel::new(
            forward_chat.id.0,
            forward_chat.title().unwrap_or_default(),
            forward_chat.username(),
            forward_chat.invite_link(),
        );
        let channel = new_channel.insert(&mut conn.lock().unwrap())?;
        msg_reply(
            format!(
                "Added the channel {} (id: {}). Now you can add subreddits to this channel.",
                channel.title, channel.chat_id
            ),
            &bot,
            &msg,
        )
        .await?;
        update_dialogue(&dialogue, SupState::MainMenu).await
    }

    pub(super) async fn on_channel_unlink(
        bot: Bot,
        dialogue: Dialogue<SupState, AppDialogue>,
        msg: Message,
        me: Me,
        conn: Arc<Mutex<SqliteConnection>>,
    ) -> TeloxideResult {
        use super::helpers::{channel_list_message, get_channels_where_admins};

        let from_user = match msg.from() {
            Some(user) => user,
            None => return msg_reply("Couldn't recognize the user. Try again.", &bot, &msg).await,
        };
        let channels = get_channels_where_admins(&bot, conn, &from_user.id, &me.user.id).await?;
        msg_reply(
            format!(
                "Okay. Type the ID of the channel you want to unlink:\n\n{}",
                channel_list_message(channels)?
            ),
            &bot,
            &msg,
        )
        .await?;
        update_dialogue(&dialogue, SupState::Channel(State::UnlinkReceiveChannel)).await
    }

    pub(super) async fn on_channel_unlink_msg(
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
                .await;
            }
        };
        let channel = Channel::get_by_chat_id(channel_id, &mut conn.lock().unwrap());
        let channel = match channel {
            Ok(channel) => channel,
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
            format!(
                "Are you sure you want to remove channel \"{}\" (Id: {})? Type the channel title to remove it",
                channel.title, channel.chat_id
            ),
            &bot,
            &msg,
        )
        .await?;
        update_dialogue(&dialogue, SupState::Channel(State::UnlinkConfirm(channel))).await
    }

    pub(super) async fn on_channel_unlink_confirm(
        bot: Bot,
        dialogue: Dialogue<SupState, AppDialogue>,
        msg: Message,
        conn: Arc<Mutex<SqliteConnection>>,
        channel: Channel,
    ) -> TeloxideResult {
        if msg.text().unwrap_or("") != channel.title {
            return msg_reply("Cancelled unlinking channel.", &bot, &msg).await;
        }
        let deleted_rows = Channel::delete(channel.chat_id, &mut conn.lock().unwrap())?;
        msg_reply(
            if deleted_rows != 0 {
                "Successfully unlinked channel."
            } else {
                "Sorry, I couldn't unlink the channel. Try again later."
            },
            &bot,
            &msg,
        )
        .await?;
        update_dialogue(&dialogue, SupState::MainMenu).await
    }

    pub(super) async fn on_channel_list(
        bot: Bot,
        msg: Message,
        conn: Arc<Mutex<SqliteConnection>>,
        me: Me,
    ) -> TeloxideResult {
        use super::helpers::{channel_list_message, get_channels_where_admins};

        let user_id = match msg.from() {
            Some(user) => user.id,
            None => return msg_reply("Couldn't recognize the user. Try again.", &bot, &msg).await,
        };
        let channels = get_channels_where_admins(&bot, conn, &user_id, &me.user.id).await?;
        msg_reply(channel_list_message(channels)?, &bot, &msg).await
    }
}

#[derive(Clone)]
pub enum State {
    LinkReceiveChannel,
    UnlinkReceiveChannel,
    UnlinkConfirm(Channel),
}

pub fn schema() -> DispatcherSchema {
    use super::{Command, State as SupState};
    use teloxide::dptree::case;
    use teloxide::prelude::*;
    Update::filter_message()
        .branch(
            case![SupState::MainMenu]
                .filter_command::<Command>()
                .branch(case![Command::ListChannels].endpoint(listeners::on_channel_list))
                .branch(case![Command::LinkChannel].endpoint(listeners::on_channel_link))
                .branch(case![Command::UnlinkChannel].endpoint(listeners::on_channel_unlink)),
        )
        .branch(
            case![SupState::Channel(x)]
                .branch(case![State::LinkReceiveChannel].endpoint(listeners::on_channel_link_msg))
                .branch(
                    case![State::UnlinkReceiveChannel].endpoint(listeners::on_channel_unlink_msg),
                )
                .branch(
                    case![State::UnlinkConfirm(selected_channel)]
                        .endpoint(listeners::on_channel_unlink_confirm),
                ),
        )
}
