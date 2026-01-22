use core::fmt;
use std::collections::HashMap;
use std::collections::btree_map::Range;
use std::env;
use std::hash::Hash;
mod commands;
use rand::random;
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
use rand::prelude::*;
use serde::Serialize;
use serenity::model::guild::Guild;
use serenity::model::id::ChannelId;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};
struct Handler;

#[derive(Debug, Clone, Copy, serde::Deserialize)]
enum Suits {
    Hearts,
    Diamonds,
    Spades,
    Clubs,
}
//id is now redundant so remove that later

#[derive(Debug, Clone)]
struct Game {
    id: MessageId,
    players: Vec<UserId>,
    cards: HashMap<UserId, Vec<Card>>,
}

#[derive(Debug, Clone, Copy)]
struct Card {
    name: &'static str,
    value: i8,
    suit: Suits,
}
impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
struct GamesKey;
impl TypeMapKey for GamesKey {
    type Value = HashMap<MessageId, Game>;
}

impl Game {
    fn add_player(&mut self, id: UserId) {
        if !self.players.contains(&id) {
            self.players.push(id);
            self.cards.insert(id, Vec::new());
        } else {
            println!("you already exist");
        }
    }
    fn len(&self) -> usize {
        self.players.len()
    }
    fn add_card(&mut self, id: UserId, card: Card) {
        if self.players.contains(&id) {
            self.cards.get_mut(&id).unwrap().push(card);
        }
    }
}
fn value(n: &str) -> i8 {
    match n {
        "King" | "Queen" | "Jack" => 10,
        "Ace" => 1,
        _ => 0,
    }
}
fn get_name(n: i8) -> &'static str {
    match n {
        2 => "Two",
        3 => "Three",
        4 => "Four",
        5 => "Five",
        6 => "Six",
        7 => "Seven",
        8 => "Eight",
        9 => "Nine",
        10 => "Ten",
        _ => "This value shouldn't exist",
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
    for &suit in &suits {
        for i in 2..10 {
            cards.push(Card {
                name: get_name(i),
                value: i,
                suit,
            })
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

async fn blackjack(ctx: &Context, channel_id: ChannelId, n: usize) {
    let msg = send_and_react(
        ctx,
        channel_id,
        "React to this message to register for the game, the game will start in 60 seconds",
    )
    .await;

    let game = Game {
        id: msg.id,
        players: Vec::new(),
        cards: HashMap::new(),
    };

    {
        let mut data = ctx.data.write().await;
        let games = data.get_mut::<GamesKey>().unwrap();
        games.insert(msg.id, game);
    }

    let ctx = ctx.clone();
    let channel_id = msg.channel_id;
    let msg_id = msg.id;

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(10)).await;

        let game = {
            let mut data = ctx.data.write().await;
            let games = data.get_mut::<GamesKey>().unwrap();
            games.remove(&msg_id)
        };

        match game {
            Some(game) if game.len() == n => {
                channel_id.say(&ctx.http, "Game started!").await.unwrap();

                start_game(&ctx, channel_id, game).await;
            }
            Some(_) => {
                println!("{:?}", game);
                channel_id
                    .say(&ctx.http, "Not enough players.")
                    .await
                    .unwrap();
            }
            None => {
                todo!();
            }
        }
    });
}

async fn start_game(ctx: &Context, channel_id: ChannelId, mut game: Game) {
    let cards = gen_cards();

    for player in game.players.clone() {
        let mut rng = rand::rng();
        let one = cards[rng.random_range(0..cards.len())];
        let two = cards[rng.random_range(0..cards.len())];

        game.add_card(player, one);
        game.add_card(player, two);
    }
    //add a view card impl for game
    for &player in game.players.iter() {
        channel_id
            .say(
                &ctx.http,
                format!(
                    "<@{}> is holding cards {} of {:?} and {} of {:?}",
                    player,
                    game.cards.get(&player).unwrap()[0].name,
                    game.cards.get(&player).unwrap()[0].suit,
                    game.cards.get(&player).unwrap()[1].name,
                    game.cards.get(&player).unwrap()[1].suit,
                ),
            )
            .await
            .unwrap();
    }

    let mut system: Vec<Card> = Vec::new();
    let one = {
        let mut rng = rand::rng();
        cards[rng.random_range(0..cards.len())]
    };
    system.push(one);
    channel_id
        .say(&ctx, format!("System drew {} of {:?}", one.name, one.suit))
        .await
        .unwrap();
}

//since this is completely random we can ask players how many decks to shuffle for card counting
//and stuff
#[async_trait]
impl EventHandler for Handler {
    async fn reaction_add(&self, ctx: Context, reac: Reaction) {
        let mut data = ctx.data.write().await;
        let games = data.get_mut::<GamesKey>().unwrap();
        //ignore bot reaction do something
        if let Some(x) = games.get_mut(&reac.message_id) {
            x.add_player(reac.user_id.unwrap());
            println!("Player added");
        };
    }
    //adding functionality for reaction remove

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
                    if let Some(n) = args.next().and_then(|w| w.parse::<usize>().ok()) {
                        blackjack(&ctx, msg.channel_id, n).await;
                    } else {
                        msg.channel_id
                            .say(&ctx.http, format!("Usage: {}blackjack <number>", prefix))
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
    {
        let mut data = client.data.write().await;
        data.insert::<GamesKey>(HashMap::new());
    }
    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
