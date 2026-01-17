use std::env;
use std::vec;
mod commands;
use serenity::all::Http;
use serenity::all::MessageId;
use serenity::all::Reaction;
use serenity::all::ReactionType;
use serenity::all::Ready;
use serenity::all::UserId;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::id::GuildId;
use serenity::prelude::*;
mod utils;
use crate::utils::servers::Server;
use crate::utils::servers::append_server;
use crate::utils::servers::load_servers;
use serenity::model::guild::Guild;
use serenity::model::id::ChannelId;
use tokio::time::{Duration, sleep};
struct Handler;

#[derive(Clone, Copy)]
enum Suits {
    Hearts,
    Diamonds,
    Spades,
    Clubs,
}

struct Game {
    id: MessageId,
    players: Vec<UserId>,
}

struct Card {
    name: &'static str,
    value: i8,
    suit: Suits,
}
impl Game {
    fn add_player(&mut self, id: UserId) {
        self.players.push(id);
    }
}
fn value(n: &str) -> i8 {
    match n {
        "King" | "Queen" | "Jack" => 10,
        "Ace" => 1,
        _ => 0,
    }
}
fn gen_cards() -> Vec<Card> {
    let names = ["King", "Queen", "Ace", "Jack"];
    let suits = [Suits::Hearts, Suits::Diamonds, Suits::Spades, Suits::Clubs];

    let mut cards = Vec::new();

    for &suit in &suits {
        for &name in &names {
            cards.push(Card {
                name,
                value: value(name),
                suit,
            });
        }
    }

    cards
}

async fn send_and_react(ctx: &Context, channel_id: ChannelId, content: &str) -> Message {
    let msg = channel_id
        .say(&ctx.http, content)
        .await
        .expect("Failed to send message");

    msg.react(&ctx.http, ReactionType::Unicode("🃏".to_string()))
        .await
        .expect("Failed to react");
    return msg;
}
async fn countdown(mut seconds: u64) {
    while seconds > 0 {
        println!("{}", seconds);
        sleep(Duration::from_secs(1)).await;
        seconds -= 1;
    }
    println!("Done!");
}
async fn create_game(game: Game) {
    let json = serde_json::to_string_pretty(game).unwrap();

    tokio::fs::write("games.tmp", json).await.unwrap();
    tokio::fs::rename("games.tmp", "servers.json")
        .await
        .unwrap();
}

async fn blackjack(ctx: &Context, channel_id: ChannelId, n: i8) {
    let msg_id = send_and_react(
        ctx,
        channel_id,
        "React to this message to register for the game, the game will start in 60 seconds",
    )
    .await;
    let current: Game = Game {
        id: msg_id.id,
        players: Vec::new(),
    };
    create_game(current);
    countdown(60).await;
}
#[async_trait]
impl EventHandler for Handler {
    async fn reaction_add(&self, ctx: Context, reac: Reaction) {
        reac.message_id;
    }
    //add the player to the game
    async fn message(&self, ctx: Context, msg: Message) {
        let servers = load_servers().await;
        let prefix = msg
            .guild_id
            .and_then(|gid| servers.iter().find(|s| s.id == gid))
            .map(|s| s.prefix.clone())
            .unwrap_or('!');
        if msg.content.starts_with(prefix) {
            let mut args = msg.content.split_whitespace();
            let first = args.next().unwrap();
            let cmd = &first[1..];
            match cmd {
                "ping" => commands::ping::run(&ctx, &msg).await,
                "setprefix" => commands::setprefix::run(args, msg.clone()).await,
                "blackjack" => {
                    if let Some(c) = args.next().and_then(|w| w.chars().next()) {
                        blackjack(&ctx, msg.channel_id, c as i8).await;
                    } else {
                        msg.channel_id
                            .say(
                                &ctx,
                                format!(
                                    "Please use the proper format example \"{}blackjack 5\"",
                                    prefix
                                ),
                            )
                            .await
                            .unwrap();
                    }
                }
                "react" => {
                    msg.react(ctx.http, ReactionType::Unicode("🔥".to_string()))
                        .await
                        .unwrap();
                }
                "rns" => {
                    let _ = send_and_react(&ctx, msg.channel_id, "test").await;
                }
                _ => (),
            }
        }
    }
    async fn guild_create(&self, _: Context, guild: Guild, is_new: Option<bool>) {
        if is_new.unwrap() == true {
            let temp = Server {
                id: guild.id,
                prefix: '!',
            };
            append_server(temp).await;
        } else {
            ()
        }
    }
    async fn cache_ready(&self, _: Context, guilds: Vec<GuildId>) {
        for guild in guilds {
            let temp = Server {
                id: guild,
                prefix: '!',
            };
            append_server(temp).await;
        }
    }
    async fn ready(&self, _: Context, _: Ready) {
        println!("Bot started");
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
