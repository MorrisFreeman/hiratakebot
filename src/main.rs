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
use spreadsheet::Book;

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
    book: Book,
}

impl Bot {
    async fn write_expenses(&self, msg: Message) -> Result<(), anyhow::Error> {
        let input_text = msg.content;
        let re = regex::Regex::new(r"([^\d]+)\s*(\d+)").unwrap();
        let text_part: String;
        let parsed_number: i32;

        let captures = match re.captures(input_text.as_str()) {
            Some(captures) => captures,
            None => return Err(anyhow::anyhow!("Invalid input format")),
        };
        text_part = captures.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
        parsed_number = captures.get(2).map(|m| m.as_str()).unwrap_or("0").parse().unwrap_or(0);

        let today = Local::now().format("%Y/%m/%d").to_string();
        let user_name = match msg.author.id {
            id if self.book.users.0.contains_key(&id.into()) => self.book.users.0[&id.into()].clone(),
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
        let range = format!("日々の記録（{}.{}）!A16:C", year, month);
        let row = self.book.get_last_row(&range).await.unwrap() + 16;

        let range = format!("日々の記録（{}.{}）!A{}", year, month, row);
        if let Err(e) = self.book.write_text(&range, values).await {
            eprintln!("Error writing to spreadsheet: {:?}", e);
        };

        Ok(())
    }
}

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: serenity::Context, msg: Message) {
        // メッセージの送信者がボット自身なら無視
        if msg.author.bot {
            return;
        }

        if msg.content == "!hello" {
            if let Err(e) = msg.channel_id.say(&ctx.http, "world!!!!!").await {
                error!("Error sending message: {:?}", anyhow::Error::new(e));
            }
        }

        if msg.channel_id == self.expenses_channel_id {
            match self.write_expenses(msg.clone()).await {
                Ok(_) => {
                    if let Err(e) = msg.reply(&ctx.http, format!("記録しました")).await {
                        error!("Error sending reply: {:?}", anyhow::Error::new(e));
                    }
                },
                Err(e) => {
                    if let Err(e) = msg.reply(&ctx.http, format!("エラーが発生しました: {:?}", e)).await {
                        error!("Error sending reply: {:?}", anyhow::Error::new(e));
                    }
                }
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

    let credentials = secrets
        .get("GOOGLE_CREDENTIALS_JSON")
        .context("'GOOGLE_CREDENTIALS_JSON' was not found")?;

    let user_id_map = secrets
        .get("USER_ID_MAP")
        .context("'USER_ID_MAP' was not found")?;
    let users: HashMap<u64, String> = serde_json::from_str(&user_id_map).unwrap();
    let book = Book::new(expenses_spreadsheet_id, users, credentials);

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
        .event_handler(Bot { channel_id, expenses_channel_id, book })
        .await
        .unwrap();

    Ok(client.into())
}
