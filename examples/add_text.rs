use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // アクセストークンの取得
    let access_token = get_access_token().await?;

    let input_text = "coop　1999";
    let parts: Vec<&str> = input_text.split(|c: char| !c.is_alphanumeric()).collect();
    let text_part = parts.get(0).unwrap_or(&"");
    let number_part = parts.get(1).unwrap_or(&"");
    let parsed_number: i32 = number_part.parse().unwrap_or(0);
    let result = vec![text_part.to_string(), parsed_number.to_string()];
    println!("{:?}", result);

    // スプレッドシートの更新
    let spreadsheet_id = std::env::var("EXPENSES_SPREADSHEET_ID").unwrap();
    let next_row = get_last_row(&spreadsheet_id, "Sheet1!A:A", &access_token).await? + 1;
    let range = format!("Sheet1!A{}", next_row);
    let response =write_text(&spreadsheet_id, &range, &access_token, vec![result]).await?;
    let result = response.json::<serde_json::Value>().await?;
    println!("Updated {} cells.", result["updatedCells"].as_i64().unwrap_or(0));

    Ok(())
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
async fn get_access_token() -> Result<String, Box<dyn std::error::Error>> {
    // 認証情報の読み込み
    let creds: Credentials = serde_json::from_str(
        &fs::read_to_string("/Users/hirata/private/tmp/spreadsheet-python/credentials.json")?
    )?;

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
async fn get_last_row(spreadsheet_id: &str, range: &str, access_token: &str) -> Result<i64, Box<dyn std::error::Error>> {
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
async fn write_text(spreadsheet_id: &str, range: &str, access_token: &str, values: Vec<Vec<String>>) -> Result<reqwest::Response, Box<dyn std::error::Error>> {
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
        .query(&[("valueInputOption", "RAW")])
        .send()
        .await?;

    Ok(response)
}
