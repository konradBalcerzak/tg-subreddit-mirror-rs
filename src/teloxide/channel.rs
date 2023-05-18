use teloxide::{prelude::*, dispatching::UpdateHandler};

use super::{BotDialogue, HandlerResult};

pub fn channel_schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use super::DialogueState;
    use dptree::case;
    dptree::entry()
        .branch(case![DialogueState::ChannelLinkRecieve]).endpoint(on_channel_link_msg)
        .branch(case![DialogueState::ChannelUnlinkRecieve]).endpoint(on_channel_unlink_msg)
        .branch(case![DialogueState::ChannelUnlinkConfirm(channel)]).endpoint(on_channel_unlink_confirm)
}

pub async fn on_channel_link(_: Bot, _: BotDialogue, _: Message) -> HandlerResult {
    todo!()
}

async fn on_channel_link_msg(_: Bot, _: BotDialogue, _: Message) -> HandlerResult {
    todo!()
}

pub async fn on_channel_unlink(_: Bot, _: BotDialogue, _: Message) -> HandlerResult {
    todo!()
}

async fn on_channel_unlink_msg(_: Bot, _: BotDialogue, _: Message) -> HandlerResult {
    todo!()
}


async fn on_channel_unlink_confirm(_: Bot, _: BotDialogue, _: Message) -> HandlerResult {
    todo!()
}
