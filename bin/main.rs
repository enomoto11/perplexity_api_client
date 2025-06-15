use anyhow::Result;
use api::perplexity::{
    ChatCompletionRequest, ChatMessage, ContextLength, Model, PerplexityClient, Role,
    WebSearchOptions,
};
use futures::stream::{self, StreamExt};
use governor::clock::DefaultClock;
use governor::{Quota, RateLimiter};
use plotters::prelude::*;
use std::fs::OpenOptions;
use std::io::Write;
use std::num::NonZeroU32;
use std::sync::Arc;
use tokio::time::Instant;

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
    ];

    let mut log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("log.txt")?;

    // log.txt　の中身をクリア
    log_file.set_len(0)?;

    writeln!(log_file, "処理を開始します {}件", contents.len())?;

    let client = PerplexityClient::new()?;

    // 1分間に45リクエストのレートリミット
    let quota =
        Quota::per_minute(NonZeroU32::new(40).unwrap()).allow_burst(NonZeroU32::new(1).unwrap());
    let limiter = Arc::new(RateLimiter::direct_with_clock(
        quota,
        &DefaultClock::default(),
    ));
    let concurrency = 6;

    let results_stream = stream::iter(contents.iter().cloned().map(String::from))
        .map(|content| {
            let client = client.clone();
            let limiter = Arc::clone(&limiter);
            async move {
                limiter.until_ready().await;
                let req = ChatCompletionRequest {
                    model: Model::Sonar,
                    messages: vec![
                        ChatMessage {
                            role: Role::System,
                            content: "You are an AI assistant that answers questions accurately and concisely.".to_string(),
                        },
                        ChatMessage {
                            role: Role::User,
                            content: content.clone(),
                        },
                    ],
                    max_tokens: Some(500),
                    temperature: Some(0.0),
                    top_p: Some(0.5),
                    web_search_options: Some(WebSearchOptions {
                        context_length: Some(ContextLength::Low),
                    }),
                };
                let res = client.chat_completions(&req).await;
                (content, res)
            }
        })
        .buffer_unordered(concurrency);

    let mut results = Vec::new();
    let start_at = Instant::now();
    futures::pin_mut!(results_stream);
    while let Some((content, res)) = results_stream.next().await {
        let elapsed = start_at.elapsed().as_secs_f32();
        if let Ok(response) = res {
            let texts: Vec<_> = response
                .choices
                .into_iter()
                .map(|c| c.message.content)
                .collect();
            writeln!(
                log_file,
                "[{:.2} sec] SUCCESS: {} => {:?}",
                elapsed, content, texts
            )?;
            results.push((content.to_string(), texts));
        } else if let Err(e) = res {
            writeln!(
                log_file,
                "[{:.2} sec] API Error for {}: {}",
                elapsed, content, e
            )?;
        }
    }

    for (_, texts) in &results {
        for text in texts {
            writeln!(log_file, "{}", text)?;
        }
        writeln!(log_file, "--------------------------------")?;
    }

    let elapsed = start_at.elapsed();
    writeln!(
        log_file,
        "🎊🎊🎊🎊🎊🎊Elapsed time: {:.2} seconds🎊🎊🎊🎊🎊🎊🎊🎊🎊🎊🎊🎊🎊🎊",
        elapsed.as_secs_f32()
    )?;

    // リクエスト流量グラフを描画
    draw_request_flow_chart("log.txt", "request_flow_6.png")?;

    Ok(())
}

fn draw_request_flow_chart(log_path: &str, output_path: &str) -> anyhow::Result<()> {
    // log.txtから経過秒数を抽出
    let log = std::fs::read_to_string(log_path)?;
    let mut points = Vec::new();
    let mut count = 0;
    for line in log.lines() {
        if let Some(sec_str) = line.strip_prefix("[") {
            if let Some(rest) = sec_str.split_once(" sec]") {
                if line.contains("SUCCESS") {
                    count += 1;
                    let sec: f32 = rest.0.parse().unwrap_or(0.0);
                    points.push((sec, count));
                }
            }
        }
    }
    if points.is_empty() {
        return Ok(());
    }
    let root = BitMapBackend::new(output_path, (800, 480)).into_drawing_area();
    root.fill(&WHITE)?;
    let x_max = points.last().unwrap().0.ceil();
    let y_max = points.last().unwrap().1 as f32;
    let mut chart = ChartBuilder::on(&root)
        .caption("Request Flow", ("sans-serif", 30))
        .margin(40)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0f32..x_max, 0f32..y_max)?;
    chart
        .configure_mesh()
        .x_desc("経過秒数")
        .y_desc("累積リクエスト数")
        .draw()?;
    chart.draw_series(LineSeries::new(
        points.iter().map(|(x, y)| (*x, *y as f32)),
        &RED,
    ))?;
    Ok(())
}
