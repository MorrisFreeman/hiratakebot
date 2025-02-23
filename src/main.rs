mod spreadsheet;

use anyhow::Context as _;
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;
use shuttle_runtime::SecretStore;
use tracing::error;
use poise::serenity_prelude as serenity;
use chrono::Local;
use regex;
use serde_json::Value;
use std::collections::HashMap;
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
    expenses_channel_id: serenity::model::id::ChannelId,
    expenses_spreadsheet_id: String,
    google_credentials_json: String,
    user_id_map: HashMap<u64, String>,
}

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: serenity::Context, msg: Message) {
        if msg.content == "!hello" {
            if let Err(e) = msg.channel_id.say(&ctx.http, "world!!!!!").await {
                error!("Error sending message: {:?}", e);
            }
        }

        if msg.channel_id == self.expenses_channel_id {
            let input_text = msg.content;
            let re = regex::Regex::new(r"([^\d]+)\s*(\d+)").unwrap();
            let text_part: String;
            let parsed_number: i32;
            if let Some(captures) = re.captures(input_text.as_str()) {
                text_part = captures.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
                let number_str = captures.get(2).map(|m| m.as_str()).unwrap_or("0");
                parsed_number = number_str.parse().unwrap_or(0);
            } else {
                text_part = "".to_string();
                parsed_number = 0;
            }
            let today = Local::now().format("%Y/%m/%d").to_string();
            let user_name = match msg.author.id {
                id if self.user_id_map.contains_key(&id.into()) => self.user_id_map[&id.into()].clone(),
                _ => "".to_string(),
            };

            let values: Vec<Vec<Value>> = vec![vec![
                Value::String(today),
                Value::String(text_part),
                Value::Number(parsed_number.into()),
                Value::Null,
                Value::String(user_name),

            ]];
            let year = Local::now().format("%Y").to_string();
            let month = Local::now().format("%m").to_string();
            let row = spreadsheet::get_last_row(&self.google_credentials_json, &self.expenses_spreadsheet_id, format!("日々の記録（{}.{}）!A16:C", year, month).as_str()).await.unwrap() + 16;
            let range = format!("日々の記録（{}.{}）!A{}", year, month, row);
            if let Err(e) = spreadsheet::write_text(&self.google_credentials_json, &self.expenses_spreadsheet_id, &range, values).await {
                eprintln!("Error writing to spreadsheet: {:?}", e);
            }
        }
    }

    // async fn ready(&self, ctx: serenity::Context, ready: Ready) {
    //     info!("{} is connected!", ready.user.name);

    //     // コンテキストをクローン（別スレッドで使用するため）
    //     let ctx = ctx.clone();
    //     let channel_id = self.channel_id;

    //     // 新しいタスクを開始
    //     tokio::spawn(async move {
    //         loop {
    //             // 定期的にメッセージを送信
    //             if let Err(e) = channel_id.say(&ctx.http, "定期メッセージです！").await {
    //                 error!("Error sending scheduled message: {:?}", e);
    //             }

    //             // 24時間待機（例として）
    //             sleep(Duration::from_secs(60)).await;
    //         }
    //     });
    // }
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

    let expenses_channel_id = secrets
        .get("EXPENSES_CHANNEL_ID")
        .context("'EXPENSES_CHANNEL_ID' was not found")?
        .parse::<u64>()
        .context("'EXPENSES_CHANNEL_ID' is not a valid u64")?;
    let expenses_channel_id = serenity::model::id::ChannelId::from(expenses_channel_id);

    let expenses_spreadsheet_id = secrets
        .get("EXPENSES_SPREADSHEET_ID")
        .context("'EXPENSES_SPREADSHEET_ID' was not found")?;

    let google_credentials_json = secrets
        .get("GOOGLE_CREDENTIALS_JSON")
        .context("'GOOGLE_CREDENTIALS_JSON' was not found")?;

    let user_id_map = secrets
        .get("USER_ID_MAP")
        .context("'USER_ID_MAP' was not found")?;
    let user_id_map: HashMap<u64, String> = serde_json::from_str(&user_id_map).unwrap();
    println!("user_id_map: {:?}", user_id_map);

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

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
        .event_handler(Bot { channel_id, expenses_channel_id, expenses_spreadsheet_id, google_credentials_json, user_id_map })
        .await
        .unwrap();

    Ok(client.into())
}
