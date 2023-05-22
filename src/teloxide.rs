mod channel;
mod subreddit;

use std::sync::{Arc, Mutex};

use dptree::case;
use teloxide::{
    dispatching::{dialogue, UpdateHandler},
    macros::BotCommands,
    prelude::*,
};

use crate::{db::establish_connection, settings::SETTINGS_INSTANCE};

#[derive(Clone, Default)]
pub enum DialogueState {
    #[default]
    MainMenu,
    ChannelLinkRecieve,
    ChannelUnlinkRecieve,
    ChannelUnlinkConfirm(crate::db::models::Channel),
    SubLinkRecieveChannel,
    SubLinkRecieveSub(crate::db::models::Channel),
    SubUnlinkRecieveChannel,
    SubUnlinkRecieveSub(crate::db::models::Channel),
}

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "These commands are supported:"
)]
enum Command {
    Help,
    Cancel,
    LinkChannel,
    UnlinkChannel,
    ListChannels,
    LinkSubreddit,
    UnlinkSubreddit,
}

pub type DispatcherSchema = UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>>;
pub type TeloxideResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
pub type AppDialogue = teloxide::dispatching::dialogue::InMemStorage<DialogueState>;

pub async fn setup_teloxide() {
    let conn = Arc::new(Mutex::new(establish_connection()));
    pretty_env_logger::init();
    let bot = Bot::new(&SETTINGS_INSTANCE.teloxide.token);
    let dispatcher = Dispatcher::builder(bot, dispatcher_schema()).dependencies(dptree::deps![
        dialogue::InMemStorage::<DialogueState>::new(),
        conn
    ]);
}

fn dispatcher_schema() -> DispatcherSchema {
    dialogue::enter::<Update, dialogue::InMemStorage<DialogueState>, DialogueState, _>()
        .branch(message_schema())
}

fn message_schema() -> DispatcherSchema {
    Update::filter_message()
        .branch(command_schema())
        .branch(channel::channel_schema())
        .branch(subreddit::subreddit_schema())
}

fn command_schema() -> DispatcherSchema {
    teloxide::filter_command::<Command, _>()
        .branch(case![Command::Cancel].endpoint(on_cancel))
        .branch(help_schema())
        .branch(main_menu_schema())
}

fn main_menu_schema() -> DispatcherSchema {
    case![DialogueState::MainMenu]
        .branch(case![Command::LinkChannel].endpoint(channel::on_channel_link))
        .branch(case![Command::UnlinkChannel].endpoint(channel::on_channel_unlink))
        .branch(case![Command::ListChannels].endpoint(channel::on_channel_list))
        .branch(case![Command::LinkSubreddit].endpoint(subreddit::on_sub_link))
        .branch(case![Command::UnlinkSubreddit].endpoint(subreddit::on_sub_unlink))
}

fn help_schema() -> DispatcherSchema {
    case![Command::Help]
        .branch(case![DialogueState::MainMenu].endpoint(on_help))
        .branch(case![DialogueState::ChannelLinkRecieve].endpoint(on_help))
        .branch(case![DialogueState::ChannelUnlinkRecieve].endpoint(on_help))
        .branch(case![DialogueState::ChannelUnlinkConfirm(channel)].endpoint(on_help))
        .branch(case![DialogueState::SubLinkRecieveChannel].endpoint(on_help))
        .branch(case![DialogueState::SubLinkRecieveSub(channel)].endpoint(on_help))
        .branch(case![DialogueState::SubUnlinkRecieveChannel].endpoint(on_help))
        .branch(case![DialogueState::SubUnlinkRecieveSub(channel)].endpoint(on_help))
}

pub async fn on_cancel(
    _: Bot,
    _: Dialogue<DialogueState, dialogue::InMemStorage<DialogueState>>,
    _: Message,
) -> TeloxideResult {
    todo!()
}

pub async fn on_help(
    _: Bot,
    _: Dialogue<DialogueState, dialogue::InMemStorage<DialogueState>>,
    _: Message,
) -> TeloxideResult {
    todo!()
}
