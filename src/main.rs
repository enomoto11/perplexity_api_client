use anyhow::{anyhow, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::env;

/// Perplexity APIのエンドポイント
const PERPLEXITY_API_BASE_URL: &str = "https://api.perplexity.ai";

/// Perplexity APIのチャットエンドポイント
const PERPLEXITY_CHAT_COMPLETIONS_URL: &str = "/chat/completions";

/// ロールを表すEnum
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")] // JSONでの表現を小文字にする
pub enum Role {
    System,
    User,
    Assistant,
}

/// チャットメッセージの構造体
#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

/// チャットリクエストの構造体
#[derive(Debug, Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    // 他にも必要なパラメータがあればここに追加
}

/// チャットレスポンスのメッセージ構造体
#[derive(Debug, Deserialize)]
pub struct ChoiceMessage {
    pub role: Role,
    pub content: String,
}

/// チャットレスポンスの選択肢構造体
#[derive(Debug, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: ChoiceMessage,
    pub finish_reason: String,
}

/// 使用量情報の構造体
#[derive(Debug, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// チャットレスポンスの構造体
#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub model: String,
    pub created: u64,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

/// Perplexity APIクライアント
pub struct PerplexityClient {
    api_key: String,
    client: reqwest::Client,
}

impl PerplexityClient {
    /// 新しいPerplexityClientを作成します。
    /// APIキーは環境変数 `PPLX_API_KEY` から読み込みます。
    pub fn new() -> Result<Self> {
        dotenv::dotenv().ok(); // .envファイルをロード
        let api_key = env::var("PPLX_API_KEY")
            .map_err(|_| anyhow!("PPLX_API_KEY environment variable not set"))?;

        let client = reqwest::Client::new();
        Ok(Self { api_key, client })
    }

    /// ヘッダーマップを作成します。
    fn create_headers(&self) -> Result<HeaderMap, reqwest::header::InvalidHeaderValue> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.api_key))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        Ok(headers)
    }

    /// チャット補完をリクエストします。
    pub async fn chat_completions(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse> {
        let url = format!(
            "{}{}",
            PERPLEXITY_API_BASE_URL, PERPLEXITY_CHAT_COMPLETIONS_URL
        );
        let headers = self.create_headers().unwrap(); // エラーハンドリングは適宜強化
        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(request)
            .send()
            .await?;

        // ステータスコードがエラーの場合はエラーを返す
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            eprintln!("API Error: Status {} - Response: {}", status, text);
            return Err(anyhow!("API Error: Status {} - Response: {}", status, text));
        }

        Ok(response.json::<ChatCompletionResponse>().await?)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // .envファイルに PPLX_API_KEY=YOUR_API_KEY を記述しておく
    let client = PerplexityClient::new()?;

    let request = ChatCompletionRequest {
        model: "sonar".to_string(), // または "llama-3-sonar-large-32k-online"
        messages: vec![
            ChatMessage {
                role: Role::System,
                content: "You are an AI assistant that answers questions accurately and concisely."
                    .to_string(),
            },
            ChatMessage {
                role: Role::User,
                content: "Rustプログラミング言語について教えてください。".to_string(),
            },
        ],
        max_tokens: Some(500),
        temperature: Some(0.7),
        top_p: Some(1.0),
    };

    println!("Sending request to Perplexity API...");
    match client.chat_completions(&request).await {
        Ok(response) => {
            println!("\n--- API Response ---");
            for choice in response.choices {
                println!("Role: {:?}", choice.message.role);
                println!("Content:\n{}", choice.message.content);
            }
            println!("\n--- Usage ---");
            println!("Prompt Tokens: {}", response.usage.prompt_tokens);
            println!("Completion Tokens: {}", response.usage.completion_tokens);
            println!("Total Tokens: {}", response.usage.total_tokens);
        }
        Err(e) => {
            eprintln!("API Error: {}", e);
        }
    }

    Ok(())
}
