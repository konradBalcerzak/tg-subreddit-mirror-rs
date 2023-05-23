mod channel;
mod subreddit;

use std::sync::{Arc, Mutex};

use diesel::SqliteConnection;
use teloxide::{
    dispatching::{dialogue, UpdateHandler},
    macros::BotCommands,
    prelude::*,
};

use crate::settings::SETTINGS_INSTANCE;

#[derive(Clone, Default)]
pub(self) enum State {
    #[default]
    MainMenu,
    Channel(channel::State),
    Sub(subreddit::State),
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

pub(self) type DispatcherSchema = UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>>;
pub(self) type TeloxideResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
pub(self) type AppDialogue = teloxide::dispatching::dialogue::InMemStorage<State>;

pub async fn setup_teloxide(reddit_bot: roux::Me, conn: SqliteConnection) {
    pretty_env_logger::init();
    let bot = Bot::new(&SETTINGS_INSTANCE.teloxide.token);
    let dispatcher = Dispatcher::builder(bot, dispatcher_schema()).dependencies(dptree::deps![
        dialogue::InMemStorage::<State>::new(),
        Arc::new(Mutex::new(conn)),
        Arc::new(Mutex::new(reddit_bot))
    ]);
}

fn dispatcher_schema() -> DispatcherSchema {
    dialogue::enter::<Update, dialogue::InMemStorage<State>, State, _>().branch(message_schema())
}

fn message_schema() -> DispatcherSchema {
    Update::filter_message()
        .branch(channel::schema())
        .branch(subreddit::schema())
}
