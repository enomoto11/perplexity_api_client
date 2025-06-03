use anyhow::Result;
use api::perplexity::{
    ChatCompletionRequest, ChatMessage, ContextLength, Model, PerplexityClient, Role,
    WebSearchOptions,
};
use futures::stream::{FuturesUnordered, StreamExt};
use tokio::time::{sleep, Duration, Instant};

#[tokio::main]
async fn main() -> Result<()> {
    let contents = [
        "Rustの言語思想を教えて",
        "Goの言語思想を教えて",
        "Pythonの言語思想を教えて",
        "Javaの言語思想を教えて",
        "C++の言語思想を教えて",
        "C#の言語思想を教えて",
        "Rubyの言語思想を教えて",
        "PHPの言語思想を教えて",
        "JavaScriptの言語思想を教えて",
        "Rustの言語思想を教えて",
        "Goの言語思想を教えて",
        "Pythonの言語思想を教えて",
        "Javaの言語思想を教えて",
        "C++の言語思想を教えて",
        "C#の言語思想を教えて",
        "Rubyの言語思想を教えて",
        "PHPの言語思想を教えて",
        "JavaScriptの言語思想を教えて",
        "Rustの言語思想を教えて",
        "Goの言語思想を教えて",
        "Pythonの言語思想を教えて",
        "Javaの言語思想を教えて",
        "C++の言語思想を教えて",
        "C#の言語思想を教えて",
        "Rubyの言語思想を教えて",
        "PHPの言語思想を教えて",
        "JavaScriptの言語思想を教えて",
        "Rustの言語思想を教えて",
        "Goの言語思想を教えて",
        "Pythonの言語思想を教えて",
        "Javaの言語思想を教えて",
        "C++の言語思想を教えて",
        "C#の言語思想を教えて",
        "Rubyの言語思想を教えて",
        "PHPの言語思想を教えて",
        "JavaScriptの言語思想を教えて",
        "Rustの言語思想を教えて",
        "Goの言語思想を教えて",
        "Pythonの言語思想を教えて",
        "Javaの言語思想を教えて",
        "C++の言語思想を教えて",
        "C#の言語思想を教えて",
        "Rubyの言語思想を教えて",
        "PHPの言語思想を教えて",
        "JavaScriptの言語思想を教えて",
        "Rustの言語思想を教えて",
        "Goの言語思想を教えて",
        "Pythonの言語思想を教えて",
        "Javaの言語思想を教えて",
        "C++の言語思想を教えて",
        "C#の言語思想を教えて",
        "Rubyの言語思想を教えて",
        "PHPの言語思想を教えて",
        "JavaScriptの言語思想を教えて",
        "Rustの言語思想を教えて",
        "Goの言語思想を教えて",
        "Pythonの言語思想を教えて",
        "Javaの言語思想を教えて",
        "C++の言語思想を教えて",
        "C#の言語思想を教えて",
        "Rubyの言語思想を教えて",
        "PHPの言語思想を教えて",
        "JavaScriptの言語思想を教えて",
        "Rustの言語思想を教えて",
        "Goの言語思想を教えて",
        "Pythonの言語思想を教えて",
        "Javaの言語思想を教えて",
        "C++の言語思想を教えて",
        "C#の言語思想を教えて",
        "Rubyの言語思想を教えて",
        "PHPの言語思想を教えて",
        "JavaScriptの言語思想を教えて",
    ];

    println!("処理を開始します {}件", contents.len());

    let start_at = Instant::now();

    let client = PerplexityClient::new()?;

    let mut results = Vec::new();
    let mut in_flight = FuturesUnordered::new();
    let mut iter = contents.iter().map(|s| s.to_string());
    let mut req_count = 0;
    let mut window_start = Instant::now();

    let max_concurrency = 6;
    let max_requests_per_min = 45;

    // 最初に最大3つまで起動
    for _ in 0..max_concurrency {
        if let Some(content) = iter.next() {
            let client = client.clone();
            let req_content = content.clone();
            let req = ChatCompletionRequest {
                model: Model::Sonar,
                messages: vec![
                    ChatMessage {
                        role: Role::System,
                        content: "You are an AI assistant that answers questions accurately and concisely.".to_string(),
                    },
                    ChatMessage {
                        role: Role::User,
                        content: req_content.clone(),
                    },
                ],
                max_tokens: Some(500),
                temperature: Some(0.0),
                top_p: Some(0.5),
                web_search_options: Some(WebSearchOptions {
                    context_length: Some(ContextLength::Low),
                }),
            };
            in_flight.push(tokio::spawn(async move {
                (req_content, client.chat_completions(&req).await)
            }));
            req_count += 1;
        }
    }

    while let Some(res) = in_flight.next().await {
        if let Ok((content, Ok(response))) = res {
            let texts: Vec<_> = response
                .choices
                .into_iter()
                .map(|c| c.message.content)
                .collect();
            results.push((content.to_string(), texts));
        } else if let Ok((content, Err(e))) = res {
            eprintln!("API Error for {}: {}", content, e);
        } else if let Err(e) = res {
            eprintln!("Join Error: {}", e);
        }

        // 45回ごとに1分待機
        if req_count > 0 && req_count % max_requests_per_min == 0 {
            let elapsed = window_start.elapsed();
            if elapsed < Duration::from_secs(60) {
                let wait = Duration::from_secs(60) - elapsed;
                println!(
                    "Rate limit reached: waiting for {:.1}秒...",
                    wait.as_secs_f32()
                );
                sleep(wait).await;
            }
            window_start = Instant::now();
        }

        // 次のリクエストを追加
        if let Some(content) = iter.next() {
            let client = client.clone();
            let req_content = content.clone();
            let req = ChatCompletionRequest {
                model: Model::Sonar,
                messages: vec![
                    ChatMessage {
                        role: Role::System,
                        content: "You are an AI assistant that answers questions accurately and concisely.".to_string(),
                    },
                    ChatMessage {
                        role: Role::User,
                        content: req_content.clone(),
                    },
                ],
                max_tokens: Some(500),
                temperature: Some(0.0),
                top_p: Some(0.5),
                web_search_options: Some(WebSearchOptions {
                    context_length: Some(ContextLength::Low),
                }),
            };
            in_flight.push(tokio::spawn(async move {
                (req_content, client.chat_completions(&req).await)
            }));
            req_count += 1;
        }
    }

    for (content, texts) in &results {
        for text in texts {
            println!("{}", text);
        }
        println!("--------------------------------");
    }

    let elapsed = start_at.elapsed();
    println!(
        "🎊🎊🎊🎊🎊🎊Elapsed time: {:.2} seconds🎊🎊🎊🎊🎊🎊🎊🎊🎊🎊🎊🎊🎊🎊",
        elapsed.as_secs_f32()
    );

    Ok(())
}
