# Perplexity API Rust クライアント

Perplexity API にアクセスするための Rust 製クライアントです。

## 特徴

- Perplexity のチャット API に簡単にアクセス
- 非同期対応（tokio）
- 標準入力からプロンプトを入力可能
- エラーハンドリングに anyhow を使用

## セットアップ

1. リポジトリをクローン

```sh
git clone https://github.com/enomoto11/perplexity_api_client.git
cd perplexity_api_client
```

2. 依存クレートのインストール

```sh
cargo build
```

3. `.env` ファイルを作成し、API キーを設定

```
PPLX_API_KEY=あなたのAPIキー
```

## 使い方

```sh
cargo run
```

実行すると「質問を入力してください:」と表示されるので、任意の質問を入力してください。

## 主要ファイル

- `src/main.rs` : メインのクライアント実装
- `Cargo.toml` : 依存クレート定義

## ライセンス

MIT
