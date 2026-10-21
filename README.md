<p align="center">
  <img src="assets/zynx.png" alt="Zynx logo" width="180">
</p>
<h1></h1>

<p align="center">
  An experimental Rust library for Discord bots, with typed Gateway events,
  async handlers, and an optional <code>#[event_handler]</code> attribute for registration.
  The workspace separates the runtime library from its procedural macros.
</p>

> [!WARNING]
> This is an early development release. APIs may change, REST support is limited,
> and the project is not yet recommended for production use.

## Run from source

Install the current stable Rust toolchain, then run these commands from the
workspace root:

```sh
cp .env.example .env
# Set DISCORD_TOKEN in .env to your bot token.
cargo run -p zynx --example zynx-example --features examples
```

The example responds to `!ping` in guild channels. Enable the **Message Content
Intent** for your bot in the Discord Developer Portal and grant it permission to
view the channel and send messages. Keep your token out of source control.

The workspace uses Rust edition 2024. CI tests current stable Rust; an older
minimum supported Rust version is not currently guaranteed.

## Minimal bot

For a separate project, depend on this checkout and Tokio:

```toml
[dependencies]
zynx = { path = "../zynx/zynx" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Adjust the path to the runtime crate in your local checkout. The example below
reads `DISCORD_TOKEN` from the process environment; the bundled example also
loads it from `.env`.

```rust,no_run
use zynx::{client::Client, models::gateway::GatewayIntents};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = std::env::var("DISCORD_TOKEN")?;
    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::new(token, intents);
    client.on_message_create(|ctx, _message| async move {
        if !ctx.is_bot() && ctx.text() == "!ping" {
            if let Err(error) = ctx.reply("Pong!").await {
                eprintln!("Could not reply: {error}");
            }
        }
    });
    client.login().await?;
    Ok(())
}
```

You can alternatively register free async functions with the re-exported macro.
Parameter types must be explicit; no direct `inventory` or `serde_json` dependency
is needed to use the macro:

```rust
use zynx::prelude::*;

#[event_handler]
async fn on_ready(ctx: Ctx) {
    println!("Logged in as {}", ctx.user().username);
}
```

## Looking forward to implement

- Caching, sharding/session-start budgeting, a complete REST client, builders,
  and voice transport.

## Development

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features
cargo test --workspace --all-targets
cargo test --workspace --all-targets --all-features
cargo test --workspace --doc --all-features
```

Tests use in-memory WebSockets and local mock HTTP servers; no bot token or
Discord connection is required. The crate README examples are compiled as
documentation tests.

See the repository's [contribution guide](https://github.com/andrewdotdev/zynx/blob/master/CONTRIBUTING.md).
The runtime and macros use Apache-2.0; the adapted API models retain their MIT
license and upstream attribution. See [NOTICE](https://github.com/andrewdotdev/zynx/blob/master/NOTICE.md).
