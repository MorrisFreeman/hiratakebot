use serenity::builder::{CreateAttachment, CreateMessage};
use serenity::http::Http;
use serenity::model::id::ChannelId;
use std::env;

async fn run() -> Result<(), serenity::Error> {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let channel_id = env::var("CHANNEL_ID")
        .expect("Expected a channel id in the environment")
        .parse::<u64>()
        .expect("Expected a channel id in the environment");
    let http = Http::new(&token);

    let channel_id = ChannelId::new(channel_id);
    let paths = [
        CreateAttachment::path(
            env!("CARGO_MANIFEST_DIR").to_owned() + "/examples/files/test_image.jpg",
        )
        .await?,
        CreateAttachment::path(
            env!("CARGO_MANIFEST_DIR").to_owned() + "/examples/files/test_movie.MOV",
        )
        .await?,
    ];

    let builder = CreateMessage::new().content("some files");
    channel_id.send_files(&http, paths, builder).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run().await?;
    Ok(())
}
