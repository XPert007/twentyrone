use std::env;
mod commands;
use serde::{Deserialize, Serialize};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::id::GuildId;
use serenity::prelude::*;
#[derive(Serialize, Deserialize, Clone)]
struct Server {
    id: GuildId,
    prefix: char,
}
struct Handler;
async fn save_servers(servers: &Vec<Server>) {
    let json = serde_json::to_string_pretty(servers).unwrap();

    tokio::fs::write("servers.tmp", json).await.unwrap();
    tokio::fs::rename("servers.tmp", "servers.json")
        .await
        .unwrap();
}
async fn load_servers() -> Vec<Server> {
    let data = tokio::fs::read_to_string("servers.json")
        .await
        .unwrap_or_else(|_| "[]".to_string());

    serde_json::from_str(&data).unwrap_or_default()
}
async fn append_server(new_server: Server) {
    let mut servers: Vec<Server> = load_servers().await;

    if !servers.iter().any(|s| s.id == new_server.id) {
        servers.push(new_server);
        save_servers(&servers).await;
    }
}
#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let mut prefix = '!';
        if msg.content.starts_with(prefix) {
            let mut args = msg.content.split_whitespace();
            let first = args.next().unwrap();
            let cmd = &first[1..];
            match cmd {
                "ping" => commands::ping::run(&ctx, &msg).await,
                "setprefix" => todo!(),
                _ => todo!(),
            }
        }
    }
    async fn cache_ready(&self, _: Context, guilds: Vec<GuildId>) {
        for guild in guilds {
            dbg!();
            let temp = Server {
                id: guild,
                prefix: '!',
            };
            dbg!();
            append_server(temp).await;
        }
    }
}

#[tokio::main]
async fn main() {
    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot.
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
