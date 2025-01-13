use std::env;

use serenity::async_trait;
use serenity::model::channel::Reaction;
use serenity::model::event::ChannelPinsUpdateEvent;
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // ピン留めメッセージが更新されたときに呼ばれる
    async fn channel_pins_update(&self, ctx: Context, pin: ChannelPinsUpdateEvent) {
        if let Ok(pinned_messages) = pin.channel_id.pins(&ctx.http).await {
            if let Some(latest_pin) = pinned_messages.first() {
                println!("最新のピン留めメッセージ: {}", latest_pin.content);
                println!("投稿者: {}", latest_pin.author.name);
            }
        }
    }

    // リアクションが追加されたときに呼ばれる
    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        println!("Reaction added: {:?}", reaction);
    }
}

#[tokio::main]
async fn main() {
    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS;

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
