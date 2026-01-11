use serde::{Deserialize, Serialize};
use serenity::model::id::GuildId;
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Server {
    pub id: GuildId,
    pub prefix: char,
}
pub async fn append_server(new_server: Server) {
    let mut servers: Vec<Server> = load_servers().await;

    if !servers.iter().any(|s| s.id == new_server.id) {
        servers.push(new_server);
        save_servers(&servers).await;
    }
}
pub async fn save_servers(servers: &Vec<Server>) {
    let json = serde_json::to_string_pretty(servers).unwrap();

    tokio::fs::write("servers.tmp", json).await.unwrap();
    tokio::fs::rename("servers.tmp", "servers.json")
        .await
        .unwrap();
}
pub async fn load_servers() -> Vec<Server> {
    let data = tokio::fs::read_to_string("servers.json")
        .await
        .unwrap_or_else(|_| "[]".to_string());

    serde_json::from_str(&data).unwrap_or_default()
}
