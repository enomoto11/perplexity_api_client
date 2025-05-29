use anyhow::Result;
use api::perplexity::{
    ChatCompletionRequest, ChatMessage, ContextLength, Model, PerplexityClient, Role,
    WebSearchOptions,
};
use futures::future::join_all;

#[tokio::main]
async fn main() -> Result<()> {
    let client = PerplexityClient::new()?;

    let request = || ChatCompletionRequest {
        model: Model::Sonar,
        messages: vec![
            ChatMessage {
                role: Role::System,
                content: "You are an AI assistant that answers questions accurately and concisely."
                    .to_string(),
            },
            ChatMessage {
                role: Role::User,
                content: "Rustの言語思想を教えて".to_string(),
            },
        ],
        max_tokens: Some(500),
        temperature: Some(0.0),
        top_p: Some(0.5),
        web_search_options: Some(WebSearchOptions {
            context_length: Some(ContextLength::Low),
        }),
    };

    let mut handles = Vec::new();
    for _ in 0..3 {
        let client = client.clone();
        let req = request();
        handles.push(tokio::spawn(
            async move { client.chat_completions(&req).await },
        ));
    }

    let mut results = Vec::new();
    let responses = join_all(handles).await;
    for resp in responses {
        match resp {
            Ok(Ok(response)) => {
                let contents: Vec<_> = response
                    .choices
                    .into_iter()
                    .map(|c| c.message.content)
                    .collect();
                results.push(contents);
            }
            Ok(Err(e)) => {
                eprintln!("API Error: {}", e);
            }
            Err(e) => {
                eprintln!("Join Error: {}", e);
            }
        }
    }
    println!("results: {:?}", results);

    Ok(())
}
