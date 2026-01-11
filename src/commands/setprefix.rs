use std::str::SplitWhitespace;

use crate::utils::servers::{Server, load_servers, save_servers};
use serenity::{all::Message, model::id::GuildId};

pub async fn run(args: SplitWhitespace<'_>, msg: Message) {
    {
        let new = args.clone().next().unwrap();
        let new_prefix = new.chars().next().unwrap();
        if let Some(guild_id) = msg.guild_id {
            setprefix(guild_id, new_prefix).await;
        } else {
            todo!() //prefix for dms
        }
    }
}

async fn setprefix(serverid: GuildId, newprefix: char) {
    let mut servers: Vec<Server> = load_servers().await;
    match servers.iter_mut().find(|s| s.id == serverid) {
        Some(s) => s.prefix = newprefix,
        None => println!("This should not have happened"),
    }
    save_servers(&servers).await;
}
