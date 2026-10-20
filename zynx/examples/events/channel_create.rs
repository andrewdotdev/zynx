use zynx::prelude::*;

// The optional second argument is still available when you prefer a separate payload.
#[event_handler]
pub async fn on_channel_create(ctx: Ctx, channel: ChannelCreate) {
    zynx::log!(
        "LOG",
        "Channel {} ({:?}) in guild {}: {:?}",
        ctx.channel_id(),
        ctx.channel_name(),
        ctx.guild_id(),
        channel.channel
    );
}
