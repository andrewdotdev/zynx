use zynx::prelude::*;

#[event_handler]
pub async fn on_message_create(ctx: Ctx) {
    if ctx.is_bot() {
        return;
    }
    if ctx.text() == "!ping"
        && let Err(error) = ctx.reply("Pong!").await
    {
        zynx::log!("ERR", "Could not reply: {error}");
    }

    if ctx.text() == "!api"
        && let Err(error) = ctx.reply("Eureka!").await
    {
        zynx::log!("ERR", "Could not reply: {error}");
    }
}
