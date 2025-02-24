use serde::{Deserialize, Serialize};
use serde_json::json;
use jsonwebtoken::{encode, EncodingKey, Header};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct User(pub HashMap<u64, String>);

#[derive(Debug, Serialize, Deserialize)]
pub struct Book {
  id: String,
  pub users: User,
  credentials: Credentials,
  access_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Credentials {
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

impl Book {
  pub fn new(id: String, users:HashMap<u64, String>, credentials: String) -> Self {
    let credentials: Credentials = serde_json::from_str(credentials.as_str()).unwrap();
    let users: User = User(users);
    Self { id, users, credentials, access_token: None }
  }

  fn create_jwt(&self) -> Result<String, Box<dyn std::error::Error>> {
    let now = chrono::Utc::now();
    let iat = now.timestamp();
    let exp = (now + chrono::Duration::hours(1)).timestamp();

    let claims = Claims {
        iss: self.credentials.client_email.clone(),
        scope: "https://www.googleapis.com/auth/spreadsheets".to_string(),
        aud: self.credentials.token_uri.clone(),
        exp,
        iat,
    };

    let header = Header::new(jsonwebtoken::Algorithm::RS256);
    let key = EncodingKey::from_rsa_pem(self.credentials.private_key.as_bytes())?;

    Ok(encode(&header, &claims, &key)?)
  }

  // アクセストークンの取得
  pub async fn get_access_token(&self) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let token_response = client
        .post(&self.credentials.token_uri)
        .json(&json!({
            "grant_type": "urn:ietf:params:oauth:grant-type:jwt-bearer",
            "assertion": self.create_jwt()?,
        }))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let access_token = token_response["access_token"].as_str().unwrap();
    Ok(access_token.to_string())
  }

  pub async fn get_last_row(&self, range: &str) -> Result<i64, Box<dyn std::error::Error>> {
    let access_token = self.get_access_token().await?;
    let client = reqwest::Client::new();
    let url = format!(
        "https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}",
        self.id, range
    );

    let response = client
        .get(&url)
        .bearer_auth(access_token)
        .send()
        .await?;

    let result = response.json::<serde_json::Value>().await?;
    let num_rows = result["values"].as_array().map_or(0, |v| v.len()) as i64;

    Ok(num_rows)
  }

  // テキストの書き込み
  pub async fn write_text(&self, range: &str, values: Vec<Vec<serde_json::Value>>) -> Result<reqwest::Response, Box<dyn std::error::Error>> {
    let value_input_option = "USER_ENTERED";
    let access_token = self.get_access_token().await?;
    let client = reqwest::Client::new();
    let url = format!(
        "https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}",
        self.id, range
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
}
