# zynx-macros

Experimental procedural macros for `zynx`. The runtime crate
re-exports `event_handler`; applications do not need to depend on this crate,
`inventory`, or `serde_json` directly to register handlers.

```rust
use zynx::prelude::*;

#[event_handler]
async fn on_ready(ctx: Ctx) {
    println!("Logged in as {}", ctx.user().username);
}

#[event_handler]
async fn on_message_create(ctx: Ctx) {
    if !ctx.is_bot() && ctx.text() == "!ping" {
        if let Err(error) = ctx.reply("Pong!").await {
            eprintln!("Could not reply: {error}");
        }
    }
}
```

Handlers must be free async functions returning `()`. Their names start with
`on_` and map to Gateway event names: `on_ready` maps to `READY`.

A second owned payload argument remains supported:

```rust
use zynx::prelude::*;

#[event_handler]
async fn on_message_create(ctx: Ctx, message: MessageCreate) {
    println!("{}: {}", ctx.channel_id(), message.message.content);
}
```

