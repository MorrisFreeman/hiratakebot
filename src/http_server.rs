use axum::{
  routing::{post, get},
  Router,
  extract::Json,
  http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::net::SocketAddr;
use poise::serenity_prelude::{Http, ChannelId};
use tracing::error;

#[derive(Deserialize)]
struct MessageRequest {
  content: String,
}

#[derive(Serialize)]
struct MessageResponse {
  success: bool,
  message: String,
}

async fn send_message(
  Json(payload): Json<MessageRequest>,
  http_context: Arc<Http>,
  channel_id: ChannelId,
) -> (StatusCode, Json<MessageResponse>) {
  println!("メッセージ送信: {:?}", payload.content);
  match channel_id.say(&http_context, &payload.content).await {
      Ok(_) => (
          StatusCode::OK,
          Json(MessageResponse {
              success: true,
              message: "メッセージを送信しました".to_string(),
          }),
      ),
      Err(e) => {
          error!("メッセージ送信エラー: {:?}", e);
          (
              StatusCode::INTERNAL_SERVER_ERROR,
              Json(MessageResponse {
                  success: false,
                  message: format!("エラー: {:?}", e),
              }),
          )
      }
  }
}

// ルートパスのハンドラを追加
async fn root() -> &'static str {
    "Discord Bot API Server is running!"
}

pub async fn start_server(http_context: Arc<Http>, channel_id: ChannelId) {
  let app = Router::new()
      .route("/", get(root))
      .route("/send-message", post(move |payload| {
          send_message(payload, http_context.clone(), channel_id)
      }));

  let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
  println!("HTTPサーバーを起動: {:?}", listener);
  axum::serve(listener, app).await.unwrap();
}
