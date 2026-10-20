use zynx::prelude::*;

#[event_handler]
pub async fn on_message_reaction_add(ctx: Ctx) {
    zynx::log!(
        "LOG",
        "{} reacted with {:?} on {}",
        ctx.user_id(),
        ctx.emoji().base.name,
        ctx.channel_id()
    );
}
