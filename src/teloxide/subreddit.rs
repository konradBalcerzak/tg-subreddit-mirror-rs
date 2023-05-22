use super::{AppDialogue, Command, DispatcherSchema, State as SupState, TeloxideResult};
use crate::db::models::Channel;
use teloxide::{dptree::case, prelude::*};

mod listeners {
    use diesel::SqliteConnection;
    use std::sync::{Arc, Mutex};
    use teloxide::types::Me;

    use super::*;
    pub(super) async fn on_sub_link(
        _: Bot,
        dialogue: Dialogue<SupState, AppDialogue>,
        _: Message,
    ) -> TeloxideResult {
        dialogue
            .update(SupState::Sub(State::SubLinkRecieveChannel))
            .await?;
        Ok(())
    }

    pub(super) async fn on_sub_link_channel(
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

    pub(super) async fn on_sub_link_sub(
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

    pub(super) async fn on_sub_unlink(
        bot: Bot,
        dialogue: Dialogue<SupState, AppDialogue>,
        msg: Message,
        conn: Arc<Mutex<SqliteConnection>>,
        selected_channel: Channel,
    ) -> TeloxideResult {
        dialogue
            .update(SupState::Sub(State::SubUnlinkRecieveChannel))
            .await?;
        Ok(())
    }

    pub(super) async fn on_sub_unlink_channel(
        _: Bot,
        dialogue: Dialogue<SupState, AppDialogue>,
        _: Message,
    ) -> TeloxideResult {
        dialogue
            .update(SupState::Sub(State::SubUnlinkRecieveChannel))
            .await?;
        Ok(())
    }

    pub(super) async fn on_sub_unlink_sub(
        _: Bot,
        dialogue: Dialogue<SupState, AppDialogue>,
        _: Message,
    ) -> TeloxideResult {
        dialogue.update(SupState::MainMenu).await?;
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
