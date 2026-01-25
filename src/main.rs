use serenity::all::Interaction;
use std::collections::HashMap;
use std::env;
mod commands;
use serenity::all::MessageId;
use serenity::all::Reaction;
use serenity::all::ReactionType;
use serenity::all::Ready;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::id::GuildId;
use serenity::prelude::*;
mod utils;
use crate::utils::servers::Server;
use crate::utils::servers::append_server;
use crate::utils::servers::load_servers;
use commands::blackjack::GamesKey;
use serenity::model::guild::Guild;
use serenity::prelude::Context;
struct Handler;

//since this is completely random we can ask players how many decks to shuffle for card counting
//and stuff
#[async_trait]
impl EventHandler for Handler {
    async fn reaction_add(&self, ctx: Context, reac: Reaction) {
        let mut data = ctx.data.write().await;
        let games = data.get_mut::<GamesKey>().unwrap();
        //ignore bot reaction do something

        if reac.user_id.unwrap() == 888501593266348032 {
            return;
        }
        if let Some(x) = games.get_mut(&reac.message_id) {
            if x.is_playing == false {
                x.add_player(reac.user_id.unwrap());
                println!("Player added");
            };
        }
    }
    async fn reaction_remove(&self, ctx: Context, reac: Reaction) {
        let mut data = ctx.data.write().await;
        let games = data.get_mut::<GamesKey>().unwrap();
        //ignore bot reaction do something

        if reac.user_id.unwrap() == 888501593266348032 {
            return;
        }
        if let Some(x) = games.get_mut(&reac.message_id) {
            if x.is_playing == false {
                x.remove_player(reac.user_id.unwrap());
                println!("Player added");
            };
        }
    }
    //adding functionality for reaction remove
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let interaction2 = interaction.clone();
        let custom_id = interaction
            .clone()
            .message_component()
            .unwrap()
            .data
            .custom_id;
        let user = interaction.message_component().unwrap().user.id.clone();
        let mut parts = custom_id.split(':');
        let action = parts.next().unwrap();
        let channel_id = interaction2.message_component().unwrap().channel_id.clone();
        let msg_id: MessageId = parts.next().unwrap().parse().unwrap();
        println!("{:?}", action);
        println!("{}", msg_id);
        match action {
            "hit" => {
                let mut data = ctx.data.write().await;
                let games = data.get_mut::<GamesKey>().unwrap();
                let game = games.get_mut(&msg_id).unwrap();
                let card = game.hit(user);
                channel_id
                    .say(
                        ctx.http,
                        format!("<@{}> drew {} of {:?}", user, card.name, card.suit),
                    )
                    .await
                    .unwrap();
            }
            "stand" => {
                let mut data = ctx.data.write().await;
                let games = data.get_mut::<GamesKey>().unwrap();
                let game = games.get_mut(&msg_id).unwrap();
                game.stand(user);
            }
            _ => {}
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
                    commands::blackjack::run(&ctx, msg.channel_id).await;
                }
                "react" => {
                    msg.react(ctx.http, ReactionType::Unicode("🔥".to_string()))
                        .await
                        .unwrap();
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
