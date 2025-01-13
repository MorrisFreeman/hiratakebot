use serenity::http::Http;
use serenity::model::id::ChannelId;
use std::env;

#[tokio::main]
async fn main() {
    // Discordトークンの設定
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // 送信先のチャンネルID
    let channel_id = env::var("CHANNEL_ID")
        .expect("Expected a channel id in the environment")
        .parse::<u64>()
        .expect("Expected a channel id in the environment");

    // HTTPクライアントの作成
    let http = Http::new(&token);

    // 入力中表示
    if let Err(why) = ChannelId::new(channel_id).broadcast_typing(&http).await {
        println!("エラー: {why:?}");
    }

    // メッセージを送信
    if let Err(why) = ChannelId::new(channel_id)
        .say(&http, "これはテストです！🤪")
        .await
    {
        println!("エラー: {why:?}");
    } else {
        println!("メッセージを送信しました");
    }
}
