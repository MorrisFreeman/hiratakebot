use serde::{Deserialize, Serialize};
use serde_json::json;
use jsonwebtoken::{encode, EncodingKey, Header};

#[derive(Debug, Serialize, Deserialize)]
struct Credentials {
    client_email: String,
    private_key: String,
    token_uri: String,
}

#[derive(Debug, Serialize)]
struct Claims {
    iss: String,
    scope: String,
    aud: String,
    exp: i64,
    iat: i64,
}

// jwtの作成
fn create_jwt(creds: &Credentials) -> Result<String, Box<dyn std::error::Error>> {
  let now = chrono::Utc::now();
  let iat = now.timestamp();
  let exp = (now + chrono::Duration::hours(1)).timestamp();

  let claims = Claims {
      iss: creds.client_email.clone(),
      scope: "https://www.googleapis.com/auth/spreadsheets".to_string(),
      aud: creds.token_uri.clone(),
      exp,
      iat,
  };

  let header = Header::new(jsonwebtoken::Algorithm::RS256);
  let key = EncodingKey::from_rsa_pem(creds.private_key.as_bytes())?;

  Ok(encode(&header, &claims, &key)?)
}

// アクセストークンの取得
async fn get_access_token(google_credentials_json: &str) -> Result<String, Box<dyn std::error::Error>> {
  // 認証情報の読み込み
  let creds: Credentials = serde_json::from_str(google_credentials_json)?;

  // アクセストークンの取得
  let client = reqwest::Client::new();
  let token_response = client
      .post(&creds.token_uri)
      .json(&json!({
          "grant_type": "urn:ietf:params:oauth:grant-type:jwt-bearer",
          "assertion": create_jwt(&creds)?,
      }))
      .send()
      .await?
      .json::<serde_json::Value>()
      .await?;

  let access_token = token_response["access_token"].as_str().unwrap();
  Ok(access_token.to_string())
}

// 最終行の取得
pub async fn get_last_row(google_credentials_json: &str, spreadsheet_id: &str, range: &str) -> Result<i64, Box<dyn std::error::Error>> {
  let access_token = get_access_token(google_credentials_json).await?;
  let client = reqwest::Client::new();
  let url = format!(
      "https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}",
      spreadsheet_id, range
  );

  let response = client
      .get(&url)
      .bearer_auth(access_token)
      .send()
      .await?;

  let result = response.json::<serde_json::Value>().await?;
  let num_rows = result["values"].as_array().map_or(0, |v| v.len());
  Ok(num_rows as i64)
}

// テキストの書き込み
pub async fn write_text(google_credentials_json: &str, spreadsheet_id: &str, range: &str, values: Vec<Vec<serde_json::Value>>) -> Result<reqwest::Response, Box<dyn std::error::Error>> {
  let value_input_option = "USER_ENTERED";
  let access_token = get_access_token(google_credentials_json).await?;
  let client = reqwest::Client::new();
  let url = format!(
      "https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}",
      spreadsheet_id, range
  );

  let response = client
      .put(&url)
      .bearer_auth(access_token)
      .json(&json!({
          "values": values
      }))
      .query(&[("valueInputOption", value_input_option)])
      .send()
      .await?;

  Ok(response)
}
