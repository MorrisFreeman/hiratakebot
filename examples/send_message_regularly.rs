use std::env;
use std::time::Duration;

use serenity::async_trait;
use serenity::prelude::*;
use serenity::model::gateway::Ready;

use tokio::time::sleep;

struct Handler {
    channel_id: serenity::model::id::ChannelId,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        // コンテキストをクローン（別スレッドで使用するため）
        let ctx = ctx.clone();
        let channel_id = self.channel_id;

        // 新しいタスクを開始
        tokio::spawn(async move {
            loop {
                // 定期的にメッセージを送信
                if let Err(e) = channel_id.say(&ctx.http, "定期メッセージです！").await {
                    println!("Error sending scheduled message: {:?}", e);
                }

                // 24時間待機（例として）
                sleep(Duration::from_secs(60)).await;
            }
        });
    }
}

#[tokio::main]
async fn main() {
    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let channel_id = env::var("CHANNEL_ID")
        .expect("Expected a channel id in the environment")
        .parse::<u64>()
        .expect("Expected a channel id in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot.
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler {
            channel_id: serenity::model::id::ChannelId::new(channel_id),
        })
        .await
        .expect("Err creating client");

    // Start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
