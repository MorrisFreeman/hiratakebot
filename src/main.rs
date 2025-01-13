use anyhow::Context as _;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use tokio::time::{sleep, Duration};
use tracing::{error, info};
use poise::serenity_prelude as serenity;

struct Data {} // User data, which is stored and accessible in all command invocations
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Displays your or another user's account creation date
#[poise::command(slash_command, prefix_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    println!("age command called");
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

struct Bot {
    channel_id: serenity::model::id::ChannelId,
}

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: serenity::Context, msg: Message) {
        if msg.content == "!hello" {
            if let Err(e) = msg.channel_id.say(&ctx.http, "world!!!!!").await {
                error!("Error sending message: {:?}", e);
            }
        }
    }

    async fn ready(&self, ctx: serenity::Context, ready: Ready) {
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
    let token = secrets
        .get("DISCORD_TOKEN")
        .context("'DISCORD_TOKEN' was not found")?;

    let channel_id = secrets
        .get("CHANNEL_ID")
        .context("'CHANNEL_ID' was not found")?
        .parse::<u64>()
        .context("'CHANNEL_ID' is not a valid u64")?;
    let channel_id = serenity::model::id::ChannelId::from(channel_id);
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![age()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .event_handler(Bot { channel_id })
        .await
        .unwrap();

    Ok(client.into())
    // // Get the discord token set in `Secrets.toml`
    // let token = secrets
    //     .get("DISCORD_TOKEN")
    //     .context("'DISCORD_TOKEN' was not found")?;

    // let channel_id = secrets
    //     .get("CHANNEL_ID")
    //     .context("'CHANNEL_ID' was not found")?
    //     .parse::<u64>()
    //     .context("'CHANNEL_ID' is not a valid u64")?;

    // let channel_id = serenity::model::id::ChannelId::from(channel_id);

    // // Set gateway intents, which decides what events the bot will be notified about
    // let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    // let client = Client::builder(&token, intents)
    //     .event_handler(Bot { channel_id })
    //     .await
    //     .expect("Err creating client");

    // Ok(client.into())
}
