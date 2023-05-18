use teloxide::{prelude::*, dispatching::UpdateHandler};

use super::{BotDialogue, HandlerResult};

pub fn subreddit_schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    use super::DialogueState;
    use dptree::case;
    dptree::entry()
        .branch(case![DialogueState::SubLinkRecieveChannel]).endpoint(on_sub_link_channel)
        .branch(case![DialogueState::SubLinkRecieveSub(channel)]).endpoint(on_sub_link_sub)
        .branch(case![DialogueState::SubUnlinkRecieveChannel]).endpoint(on_sub_unlink_channel)
        .branch(case![DialogueState::SubUnlinkRecieveSub(channel)]).endpoint(on_sub_unlink_sub)
}

pub async fn on_sub_link(_: Bot, _: BotDialogue, _: Message) -> HandlerResult {
    todo!()
}

pub async fn on_sub_link_channel(_: Bot, _: BotDialogue, _: Message) -> HandlerResult {
    todo!()
}

async fn on_sub_link_sub(_: Bot, _: BotDialogue, _: Message) -> HandlerResult {
    todo!()
}

pub async fn on_sub_unlink(_: Bot, _: BotDialogue, _: Message) -> HandlerResult {
    todo!()
}

pub async fn on_sub_unlink_channel(_: Bot, _: BotDialogue, _: Message) -> HandlerResult {
    todo!()
}

async fn on_sub_unlink_sub(_: Bot, _: BotDialogue, _: Message) -> HandlerResult {
    todo!()
}
