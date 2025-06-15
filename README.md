# calc-mcp

Rust製セキュア数式計算MCPサーバ

## 概要
`calc-mcp`は、四則演算・べき乗・括弧・各種数学関数（平方根、絶対値、三角関数、自然対数）を安全に計算できるMCP（Model Context Protocol）サーバです。外部から数式を受け取り、計算結果を返します。DoS攻撃や危険な入力から保護するためのセキュリティ機能も備えています。

## 特徴
- 四則演算（+, -, *, /）
- べき乗（^）
- 括弧による優先順位制御
- 数学関数: `sqrt`, `abs`, `sin`, `cos`, `tan`, `ln`
- 入力長制限（最大1000文字）
- 危険な文字（`;`, `|`, `&`）の拒否
- 関数ホワイトリストによる安全性
- MCPプロトコル対応

## インストール

```sh
# Rustが必要です
cargo build --release
```

## 使い方

```sh
cargo run --release
```

MCPクライアントからJSON-RPCで数式を送信してください。

### リクエスト例
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "calculate",
    "arguments": {
      "expression": "2 + 3 * 4"
    }
  }
}
```

### レスポンス例
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "content": [
      { "type": "text", "text": "計算結果: 14" }
    ]
  }
}
```

## サポートする数式
- 四則演算: `2 + 3 * 4`
- 括弧: `(2 + 3) * 4`
- べき乗: `2^3`, `25^0.5`
- 関数: `sqrt(25)`, `abs(-10)`, `sin(1.57)`, `cos(0)`, `tan(0.5)`, `ln(2.718)`

## セキュリティ
- 入力長が1000文字を超える場合はエラー
- 危険な文字（`;`, `|`, `&`）を含む場合はエラー
- 許可されていない関数名はエラー
- ゼロ除算や無効な計算（NaN, 無限大）はエラー

## テスト

ユニットテスト・統合テストが用意されています。

```sh
cargo test
```

## 依存ライブラリ
- [tokio](https://crates.io/crates/tokio)
- [serde](https://crates.io/crates/serde)
- [serde_json](https://crates.io/crates/serde_json)
- [anyhow](https://crates.io/crates/anyhow)
- [schemars](https://crates.io/crates/schemars)
- [rmcp](https://github.com/modelcontextprotocol/rust-sdk)

## ライセンス

このリポジトリはMITライセンスの下で公開されています。
