use serde::{Deserialize, Serialize};
use serenity::model::id::GuildId;
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Server {
    pub id: GuildId,
    pub prefix: char,
}
pub async fn save_servers(servers: &Vec<Server>) {
    let json = serde_json::to_string_pretty(servers).unwrap();

    tokio::fs::write("servers.tmp", json).await.unwrap();
    tokio::fs::rename("servers.tmp", "servers.json")
        .await
        .unwrap();
}
