use anyhow::{anyhow, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::env;

const PERPLEXITY_API_URL: &str = "https://api.perplexity.ai/chat/completions";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Model {
    Sonar,
    SonarPro,
    SonarReasoning,
    SonarReasoningPro,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContextLength {
    Low,
    Medium,
    High,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebSearchOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_length: Option<ContextLength>,
}

#[derive(Debug, Serialize)]
pub struct ChatCompletionRequest {
    pub model: Model,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(rename = "webSearchOptions", skip_serializing_if = "Option::is_none")]
    pub web_search_options: Option<WebSearchOptions>,
}

#[derive(Debug, Deserialize)]
pub struct ChoiceMessage {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: ChoiceMessage,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Clone)]
pub struct PerplexityClient {
    api_key: String,
    client: reqwest::Client,
}

impl PerplexityClient {
    pub fn new() -> Result<Self> {
        dotenv::dotenv().ok();
        let api_key = env::var("PPLX_API_KEY")
            .map_err(|_| anyhow!("PPLX_API_KEY environment variable not set"))?;

        let client = reqwest::Client::new();
        Ok(Self { api_key, client })
    }

    fn create_headers(&self) -> Result<HeaderMap, reqwest::header::InvalidHeaderValue> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.api_key))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        Ok(headers)
    }

    pub async fn chat_completions(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse> {
        let headers = self.create_headers().unwrap();
        let response = self
            .client
            .post(PERPLEXITY_API_URL)
            .headers(headers)
            .json(request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await?;
            eprintln!("API Error: Status {} - Response: {}", status, text);
            return Err(anyhow!("API Error: Status {} - Response: {}", status, text));
        }

        Ok(response.json::<ChatCompletionResponse>().await?)
    }
}
