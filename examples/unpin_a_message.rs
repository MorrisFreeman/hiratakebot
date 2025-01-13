use serenity::http::Http;
use serenity::model::id::ChannelId;
use std::env;

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Discordトークンの設定
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // 送信先のチャンネルID
    let channel_id = env::var("CHANNEL_ID")
        .expect("Expected a channel id in the environment")
        .parse::<u64>()
        .expect("Expected a channel id in the environment");
    let message_id = env::var("MESSAGE_ID")
        .expect("Expected a message id in the environment")
        .parse::<u64>()
        .expect("Expected a message id in the environment");

    // HTTPクライアントの作成
    let http = Http::new(&token);

    ChannelId::new(channel_id).unpin(http, message_id).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run().await?;
    Ok(())
}
