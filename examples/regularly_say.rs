use anyhow::Context as _;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use tokio::time::{sleep, Duration};
use tracing::{error, info};

struct Bot {
    channel_id: serenity::model::id::ChannelId,
}

#[async_trait]
impl EventHandler for Bot {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        // コンテキストをクローン（別スレッドで使用するため）
        let ctx = ctx.clone();
        let channel_id = self.channel_id;

        // 新しいタスクを開始
        tokio::spawn(async move {
            loop {
                // 定期的にメッセージを送信
                if let Err(e) = channel_id.say(&ctx.http, "定期メッセージです！").await {
                    error!("Error sending scheduled message: {:?}", e);
                }

                // 24時間待機（例として）
                sleep(Duration::from_secs(60)).await;
            }
        });
    }
}

#[shuttle_runtime::main]
async fn serenity(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> shuttle_serenity::ShuttleSerenity {
    // Get the discord token set in `Secrets.toml`
    let token = secrets
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    let channel_id = secrets
        .get("CHANNEL_ID")
        .context("'CHANNEL_ID' was not found")?
        .parse::<u64>()
        .context("'CHANNEL_ID' is not a valid u64")?;

    let channel_id = serenity::model::id::ChannelId::from(channel_id);

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let client = Client::builder(&token, intents)
        .event_handler(Bot { channel_id })
        .await
        .expect("Err creating client");

    Ok(client.into())
}
