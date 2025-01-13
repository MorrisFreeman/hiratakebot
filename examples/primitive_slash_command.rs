use std::env;

use serenity::all::CreateCommand;
use serenity::all::Ready;
use serenity::async_trait;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::application::Interaction;
use serenity::model::id::GuildId;
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let guild_id = GuildId::new(env::var("GUILD_ID").unwrap().parse::<u64>().unwrap());
        let commands = vec![commands::ping::register()];
        guild_id.set_commands(&ctx.http, commands).await.unwrap();
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            println!("Received command interaction: {command:#?}");

            let content = match command.data.name.as_str() {
                "ping" => Some(commands::ping::run(&command.data.options())),
                // "id" => Some(commands::id::run(&command.data.options())),
                // "attachmentinput" => Some(commands::attachmentinput::run(&command.data.options())),
                // "modal" => {
                //     commands::modal::run(&ctx, &command).await.unwrap();
                //     None
                // },
                _ => Some("not implemented :(".to_string()),
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    println!("Cannot respond to slash command: {why}");
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // Login with a bot token from the environment
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

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

mod commands {
    pub mod ping {
        use serenity::all::CreateCommand;
        use serenity::all::ResolvedOption;
        pub fn run(_options: &[ResolvedOption]) -> String {
            "Hey, I'm alive!".to_string()
        }

        pub fn register() -> CreateCommand {
            CreateCommand::new("ping").description("A ping command")
        }
    }
}
