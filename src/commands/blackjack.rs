use core::fmt;
use rand::prelude::*;
use serenity::all::CreateButton;
use serenity::all::MessageId;
use serenity::all::ReactionType;
use serenity::all::UserId;
use serenity::builder::CreateMessage;
use serenity::model::channel::Message;
use serenity::model::id::ChannelId;
use serenity::prelude::Context;
use serenity::prelude::*;
use std::collections::HashMap;
use tokio::time::Duration;
#[derive(Debug, Clone, Copy, serde::Deserialize)]
pub enum Suits {
    Hearts,
    Diamonds,
    Spades,
    Clubs,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Status {
    Hit,
    Stand,
    None,
}
#[derive(Debug, Clone)]
pub struct Game {
    pub players: Vec<UserId>,
    pub cards: HashMap<UserId, Vec<Card>>,
    pub has_clicked: HashMap<UserId, bool>,
    pub status: HashMap<UserId, Status>,
    pub is_playing: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct Card {
    pub name: &'static str,
    value: i8,
    pub suit: Suits,
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
pub struct GamesKey;
impl TypeMapKey for GamesKey {
    type Value = HashMap<MessageId, Game>;
}
impl Game {
    pub fn add_player(&mut self, id: UserId) {
        if !self.players.contains(&id) {
            self.players.push(id);
            self.cards.insert(id, Vec::new());
            self.has_clicked.insert(id, false);
            self.status.insert(id, Status::None);
        } else {
            println!("you already exist");
        }
    }

    pub fn remove_player(&mut self, id: UserId) {
        if let Some(pos) = self.players.iter().position(|x| *x == id) {
            self.players.remove(pos);
            self.cards.remove(&id);
            self.has_clicked.remove(&id);
            self.status.remove(&id);
        }
    }

    pub fn add_card(&mut self, id: UserId) {
        if self.players.contains(&id) {
            let cards = gen_cards();
            let mut rng = rand::rng();
            let one = cards[rng.random_range(0..cards.len())];
            self.cards.get_mut(&id).unwrap().push(one);
        }
    }

    pub fn hit(&mut self, id: UserId) -> Card {
        let cards = gen_cards();
        let mut rng = rand::rng();
        let one = cards[rng.random_range(0..cards.len())];

        if let Some(hand) = self.cards.get_mut(&id) {
            hand.push(one);
        }

        if let Some(clicked) = self.has_clicked.get_mut(&id) {
            *clicked = true;
        }

        if let Some(status) = self.status.get_mut(&id) {
            *status = Status::Hit;
        }

        return one;
    }

    pub fn stand(&mut self, id: UserId) {
        if self.players.contains(&id) {
            if let Some(clicked) = self.has_clicked.get_mut(&id) {
                *clicked = true;
            }

            if let Some(status) = self.status.get_mut(&id) {
                *status = Status::Stand;
            }
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
pub async fn run(ctx: &Context, channel_id: ChannelId) {
    let msg = send_and_react(
        ctx,
        channel_id,
        "React to this message to register for the game, the game will start in 60 seconds",
    )
    .await;

    let game = Game {
        players: Vec::new(),
        cards: HashMap::new(),
        has_clicked: HashMap::new(),
        status: HashMap::new(),
        is_playing: false,
        //is playing status
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

        start_game(&ctx, channel_id, msg_id).await;
    });
}

async fn button(ctx: &Context, channel_id: ChannelId, msg_id: MessageId) {
    channel_id
        .send_message(
            &ctx.http,
            CreateMessage::new()
                .content("Select what your move would be:")
                .button(CreateButton::new(format!("hit:{}", msg_id)).label("Hit"))
                .button(CreateButton::new(format!("stand:{}", msg_id)).label("Stand")),
        )
        .await
        .unwrap();
}

async fn start_game(ctx: &Context, channel_id: ChannelId, msg_id: MessageId) {
    let cards = gen_cards();

    {
        let mut data = ctx.data.write().await;
        let games = data.get_mut::<GamesKey>().unwrap();
        let game = games.get_mut(&msg_id).unwrap();
        game.is_playing = true;
        for player in game.players.clone() {
            game.add_card(player);
            game.add_card(player);
        }
    }

    // add a view card impl for game
    {
        let mut data = ctx.data.write().await;
        let games = data.get_mut::<GamesKey>().unwrap();
        let game = games.get(&msg_id).unwrap();

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

    {
        let mut data = ctx.data.write().await;
        let games = data.get_mut::<GamesKey>().unwrap();
        let game = games.get(&msg_id).unwrap();

        for &player in game.players.iter() {
            if game
                .cards
                .get(&player)
                .unwrap()
                .iter()
                .map(|card| card.value)
                .sum::<i8>()
                == 21 as i8
            {
                channel_id
                    .say(&ctx.http, format!("<@{}> won", &player))
                    .await
                    .unwrap();
                return;
            }
        }
    }

    let mut finished = false;

    while !finished {
        button(ctx, channel_id, msg_id).await;

        let mut contains_false = true;

        while contains_false {
            let data = ctx.data.read().await;
            let games = data.get::<GamesKey>().unwrap();
            let game = games.get(&msg_id).unwrap();

            if game.has_clicked.values().any(|x| !x) {
                continue;
            } else {
                contains_false = false;
            }
        }

        {
            let mut data = ctx.data.write().await;
            let games = data.get_mut::<GamesKey>().unwrap();
            let game = games.get_mut(&msg_id).unwrap();

            for v in game.has_clicked.values_mut() {
                *v = false;
            }
        }

        {
            let data = ctx.data.write().await;
            let games = data.get::<GamesKey>().unwrap();
            let game = games.get(&msg_id).unwrap();

            let winners: Vec<UserId> = game
                .cards
                .iter()
                .filter(|(_, hand)| hand.iter().map(|c| c.value).sum::<i8>() == 21)
                .map(|(player, _)| *player)
                .collect();

            if winners.len() == 1 {
                channel_id
                    .say(&ctx.http, format!("<@{}> has won", winners[0]))
                    .await
                    .unwrap();
            } else if !winners.is_empty() {
                let mentions = winners
                    .iter()
                    .map(|id| format!("<@{}>", id))
                    .collect::<Vec<_>>()
                    .join(" ");

                channel_id
                    .say(&ctx.http, format!("The winners are: {}", mentions))
                    .await
                    .unwrap();
            }
        }

        {
            let data = ctx.data.write().await;
            let games = data.get::<GamesKey>().unwrap();
            let game = games.get(&msg_id).unwrap();

            if game.status.values().any(|s| *s == Status::Hit) {
            } else {
                while system.iter().map(|c| c.value).sum::<i8>() < 17 {
                    let two = {
                        let mut rng = rand::rng();
                        cards[rng.random_range(0..cards.len())]
                    };
                    system.push(two);
                    channel_id
                        .say(&ctx.http, format!("System drew {}", two))
                        .await
                        .unwrap();
                }

                let system_sum: i8 = system.iter().map(|c| c.value).sum();
                if system_sum > 21 {
                    channel_id
                        .say(&ctx.http, "system busted, everyone won")
                        .await
                        .unwrap();
                    finished = true;
                    continue;
                } else {
                    let winners: Vec<UserId> = game
                        .cards
                        .iter()
                        .filter(|(_, hand)| {
                            let sum = hand.iter().map(|c| c.value).sum::<i8>();
                            sum < 21 && sum > system_sum
                        })
                        .map(|(player, _)| *player)
                        .collect();

                    if winners.len() == 1 {
                        channel_id
                            .say(&ctx.http, format!("<@{}> won", winners[0]))
                            .await
                            .unwrap();
                        finished = true;
                        continue;
                    } else if !winners.is_empty() {
                        let mentions = winners
                            .iter()
                            .map(|id| format!("<@{}>", id))
                            .collect::<Vec<_>>()
                            .join(" ");

                        channel_id
                            .say(&ctx.http, format!("The winners are: {}", mentions))
                            .await
                            .unwrap();
                        finished = true;
                        continue;
                    } else {
                        channel_id
                            .say(&ctx.http, " none of you won, lmao losers")
                            .await
                            .unwrap();
                        finished = true;
                        continue;
                    }
                }
            }
        }
        {
            let mut data = ctx.data.write().await;
            let games = data.get_mut::<GamesKey>().unwrap();
            let game = games.get_mut(&msg_id).unwrap();
            for player in game.status.values_mut() {
                *player = Status::None;
            }
        }
    }
}
