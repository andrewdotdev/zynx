mod events;

#[cfg(feature = "examples")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use zynx::client::Client;
    use zynx::models::gateway::GatewayIntents;
    use dotenvy::dotenv;

    dotenv().ok();
    let token = std::env::var("DISCORD_TOKEN")?;

    let intents = GatewayIntents::all();
    let mut client = Client::new(&token, intents);

    client.login().await?;
    Ok(())
}

#[cfg(not(feature = "examples"))]
fn main() {
    eprintln!("This example requires the `examples` feature");
    eprintln!("Usage: cargo run -p zynx --example zynx-example --features examples");
}
