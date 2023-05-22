use std::error::Error;
use std::sync::{Arc, Mutex};

use diesel::{prelude::*, SqliteConnection};

use teloxide::{dispatching::UpdateHandler, prelude::*, types::Me};

use crate::{
    db::{self, models::Channel, schema::channel::dsl::*},
    teloxide::DialogueState,
};

use super::{AppDialogue, TeloxideResult};

pub fn channel_schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use dptree::case;
    dptree::entry()
        .branch(case![DialogueState::ChannelLinkRecieve].endpoint(on_channel_link_msg))
        .branch(case![DialogueState::ChannelUnlinkRecieve].endpoint(on_channel_unlink_msg))
        .branch(
            case![DialogueState::ChannelUnlinkConfirm(selected_channel)]
                .endpoint(on_channel_unlink_confirm),
        )
}

pub async fn on_channel_link(
    bot: Bot,
    dialogue: Dialogue<DialogueState, AppDialogue>,
    msg: Message,
) -> TeloxideResult {
    bot.send_message(msg.chat.id, "Got it. Forward a message from the channel here.\n Remember that this bot needs to be an administrator in that channel first.")
        .reply_to_message_id(msg.id)
        .await?;
    dialogue.update(DialogueState::ChannelLinkRecieve).await?;
    Ok(())
}

async fn on_channel_link_msg(
    bot: Bot,
    dialogue: Dialogue<DialogueState, AppDialogue>,
    msg: Message,
    me: Me,
    conn: Arc<Mutex<SqliteConnection>>,
) -> TeloxideResult {
    let forward_chat = match msg.forward_from_chat() {
        Some(chat) => chat,
        None => {
            bot.send_message(
                msg.chat.id,
                format!(
                    "This message is not a forward. Try again or use command /cancel@{}",
                    me.username()
                ),
            )
            .reply_to_message_id(msg.id)
            .await?;
            return Ok(());
        }
    };
    if !forward_chat.is_channel() {
        bot.send_message(
            msg.chat.id,
            format!(
                "This message is not forwarded from a channel. Try again or use command /cancel@{}",
                me.username()
            ),
        )
        .reply_to_message_id(msg.id)
        .await?;
        return Ok(());
    }
    let chat_admins = bot.get_chat_administrators(forward_chat.id).await?;
    if !chat_admins.iter().any(|admin| admin.user.id == me.id) {
        bot.send_message(msg.chat.id, format!("This bot is not an administrator in this channel. Try again or use command /cancel@{}", me.username()))
            .reply_to_message_id(msg.id)
            .await?;
        return Ok(());
    }
    let new_channel = db::insert_channel(
        &mut *conn.lock().unwrap(),
        forward_chat.id,
        forward_chat.title().unwrap_or_default(),
        forward_chat.username(),
        forward_chat.invite_link(),
    )?;
    bot.send_message(
        msg.chat.id,
        format!(
            "Added the channel {} (id: {}). Now you can add subreddits to this channel.",
            new_channel
                .username
                .or(new_channel.invite_link)
                .unwrap_or_default(),
            new_channel.chat_id
        ),
    )
    .await?;
    dialogue.update(DialogueState::MainMenu).await?;
    return Ok(());
}

pub async fn on_channel_unlink(
    bot: Bot,
    dialogue: Dialogue<DialogueState, AppDialogue>,
    msg: Message,
    me: Me,
    conn: Arc<Mutex<SqliteConnection>>,
) -> TeloxideResult {
    let from_user = match msg.from() {
        Some(user) => user,
        None => {
            bot.send_message(msg.chat.id, "Couldn't recognize the user. Try again.")
                .await?;
            return Ok(());
        }
    };
    let channels = get_channels_where_admins(&bot, conn, &from_user.id, &me.user.id).await?;
    let select_channel_msg =
        String::from("Okay. Type the ID of the channel you want to unkink:\n\n")
            + channel_list_message(channels)?.as_str();
    bot.send_message(msg.chat.id, select_channel_msg)
        .reply_to_message_id(msg.id)
        .await?;
    dialogue.update(DialogueState::ChannelUnlinkRecieve).await?;
    Ok(())
}

async fn on_channel_unlink_msg(
    bot: Bot,
    dialogue: Dialogue<DialogueState, AppDialogue>,
    msg: Message,
    conn: Arc<Mutex<SqliteConnection>>,
) -> TeloxideResult {
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
    dialogue
        .update(DialogueState::ChannelUnlinkConfirm(selected_channel))
        .await?;
    Ok(())
}

async fn on_channel_unlink_confirm(
    bot: Bot,
    dialogue: Dialogue<DialogueState, AppDialogue>,
    msg: Message,
    conn: Arc<Mutex<SqliteConnection>>,
    selected_channel: Channel,
) -> TeloxideResult {
    let deleted_rows = diesel::delete(channel)
        .filter(chat_id.eq(selected_channel.chat_id))
        .execute(&mut *conn.lock().unwrap())?;
    bot.send_message(
        msg.chat.id,
        if deleted_rows != 0 {
            "Successfully unlinked channel."
        } else {
            "Sorry, I couldn't unlink the channel. Try again later."
        },
    )
    .reply_to_message_id(msg.id)
    .await?;
    dialogue.update(DialogueState::MainMenu).await?;
    Ok(())
}

pub async fn on_channel_list(
    bot: Bot,
    msg: Message,
    conn: Arc<Mutex<SqliteConnection>>,
    me: Me,
) -> TeloxideResult {
    let user_id = match msg.from() {
        Some(user) => user.id,
        None => {
            bot.send_message(msg.chat.id, "Couldn't recognize the user. Try again.")
                .await?;
            return Ok(());
        }
    };
    let channels = get_channels_where_admins(&bot, conn, &user_id, &me.user.id).await?;
    bot.send_message(msg.chat.id, channel_list_message(channels)?)
        .reply_to_message_id(msg.id)
        .await?;
    Ok(())
}

fn channel_list_message(
    channels: Vec<Channel>,
) -> Result<String, Box<dyn Error + Send + Sync + 'static>> {
    let mut message_content = String::new();
    for available_channel in channels {
        message_content += format!(
            "Channel name: {}\nChannel id: {}\n\n",
            if let Some(uname) = available_channel.username {
                uname
            } else {
                String::from("(No channel name found)")
            },
            available_channel.chat_id
        )
        .as_str();
    }
    Ok(message_content)
}

async fn get_channels_where_admins(
    bot: &Bot,
    conn: Arc<Mutex<SqliteConnection>>,
    user_id: &UserId,
    bot_id: &UserId,
) -> Result<Vec<Channel>, Box<dyn Error + Send + Sync>> {
    let linked_channel_ids = channel
        .distinct()
        .select(chat_id)
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
    let available_channels = channel
        .filter(chat_id.eq_any(available_channels))
        .load::<Channel>(&mut *conn.lock().unwrap())?;
    return Ok(available_channels);
}
