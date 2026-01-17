use std::env;
mod commands;
use serde::Deserialize;
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
use serde::Serialize;
#[derive(Clone, Copy)]
enum Suits {
    Hearts,
    Diamonds,
    Spades,
    Clubs,
}
#[derive(Serialize, Deserialize, Clone)]
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
    fn len(&self) -> usize {
        self.players.len()
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
async fn append_game(game: Game) {
    let mut games: Vec<Game> = load_games().await;

    if !games.iter().any(|g| g.id == game.id) {
        games.push(game);
        save_games(&games).await;
    }
}
async fn load_games() -> Vec<Game> {
    let data = tokio::fs::read_to_string("games.json")
        .await
        .unwrap_or_else(|_| "[]".to_string());

    serde_json::from_str(&data).unwrap_or_default()
}

async fn remove_game(game: &Game) {
    let mut games: Vec<Game> = load_games().await;

    if let Some(pos) = games.iter().position(|x| x.id == game.id) {
        games.remove(pos);
    }
    save_games(&games).await;
}
async fn save_games(games: &[Game]) {
    let json = serde_json::to_string_pretty(games).unwrap();

    tokio::fs::write("games.tmp", json).await.unwrap();
    tokio::fs::rename("games.tmp", "games.json").await.unwrap();
}

async fn blackjack(ctx: &Context, channel_id: ChannelId, n: usize) {
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
    append_game(current.clone()).await;
    countdown(60).await;
    let games = load_games().await;
    let mut sufficient_players = false;
    for game in games {
        if game.id == current.id {
            if n == game.len() {
                sufficient_players = true;
                channel_id.say(&ctx, "Game started").await.unwrap();
                remove_game(&current).await;
            }
        }
    }
    if !sufficient_players {
        channel_id.say(&ctx, "Not enough players").await.unwrap();
    }
}
#[async_trait]
impl EventHandler for Handler {
    async fn reaction_add(&self, _: Context, reac: Reaction) {
        let mut games = load_games().await;
        for game in games.iter_mut() {
            println!("{} is game id, {} is message_id", game.id, reac.message_id);
            if game.id == reac.message_id {
                if !game.players.contains(&reac.user_id.unwrap()) {
                    game.add_player(reac.user_id.unwrap());
                    save_games(&games).await;
                    println!("games saved with your name");
                }
                break;
            }
        }
    }

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
                        blackjack(&ctx, msg.channel_id, c as usize).await;
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
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
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
