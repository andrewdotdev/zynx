use zynx::prelude::*;

#[event_handler]
pub async fn on_ready(ctx: Ctx) {
    zynx::log!("OK", "Logged in as {}", ctx.user().username);
}
