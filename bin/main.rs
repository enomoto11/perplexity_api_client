use anyhow::Result;
use api::perplexity::{
    ChatCompletionRequest, ChatMessage, ContextLength, Model, PerplexityClient, Role,
    WebSearchOptions,
};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<()> {
    let client = PerplexityClient::new()?;

    print!("質問を入力してください: ");
    io::stdout().flush()?;
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input)?;
    let user_input = user_input.trim().to_string();

    let request = ChatCompletionRequest {
        model: Model::Sonar,
        messages: vec![
            ChatMessage {
                role: Role::System,
                content: "You are an AI assistant that answers questions accurately and concisely."
                    .to_string(),
            },
            ChatMessage {
                role: Role::User,
                content: user_input,
            },
        ],
        max_tokens: Some(500),
        temperature: Some(0.0),
        top_p: Some(0.5),
        web_search_options: Some(WebSearchOptions {
            context_length: Some(ContextLength::Low),
        }),
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
