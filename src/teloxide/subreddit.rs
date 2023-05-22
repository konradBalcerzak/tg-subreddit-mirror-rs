use super::{AppDialogue, Command, DispatcherSchema, State as GlobalState, TeloxideResult};
use crate::db::models::Channel;
use teloxide::{dptree::case, prelude::*};

mod listeners {
    use super::*;
    pub(super) async fn on_sub_link(
        _: Bot,
        dialogue: Dialogue<GlobalState, AppDialogue>,
        _: Message,
    ) -> TeloxideResult {
        dialogue
            .update(GlobalState::Subreddit(State::SubLinkRecieveChannel))
            .await?;
        Ok(())
    }

    pub(super) async fn on_sub_link_channel(
        _: Bot,
        dialogue: Dialogue<GlobalState, AppDialogue>,
        _: Message,
    ) -> TeloxideResult {
        dialogue
            .update(GlobalState::Subreddit(State::SubLinkRecieveChannel))
            .await?;
        Ok(())
    }

    pub(super) async fn on_sub_link_sub(
        _: Bot,
        dialogue: Dialogue<GlobalState, AppDialogue>,
        _: Message,
    ) -> TeloxideResult {
        dialogue.update(GlobalState::MainMenu).await?;
        Ok(())
    }

    pub(super) async fn on_sub_unlink(
        _: Bot,
        dialogue: Dialogue<GlobalState, AppDialogue>,
        _: Message,
    ) -> TeloxideResult {
        dialogue
            .update(GlobalState::Subreddit(State::SubUnlinkRecieveChannel))
            .await?;
        Ok(())
    }

    pub(super) async fn on_sub_unlink_channel(
        _: Bot,
        dialogue: Dialogue<GlobalState, AppDialogue>,
        _: Message,
    ) -> TeloxideResult {
        dialogue
            .update(GlobalState::Subreddit(State::SubUnlinkRecieveChannel))
            .await?;
        Ok(())
    }

    pub(super) async fn on_sub_unlink_sub(
        _: Bot,
        dialogue: Dialogue<GlobalState, AppDialogue>,
        _: Message,
    ) -> TeloxideResult {
        dialogue.update(GlobalState::MainMenu).await?;
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
            case![GlobalState::MainMenu]
                .filter_command::<Command>()
                .branch(case![Command::LinkSubreddit].endpoint(listeners::on_sub_link))
                .branch(case![Command::UnlinkSubreddit].endpoint(listeners::on_sub_unlink)),
        )
        .branch(
            case![GlobalState::Subreddit(x)]
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
