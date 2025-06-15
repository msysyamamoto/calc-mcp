use anyhow::Result;
use serde_json::{json, Value};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_mcp_server_initialization() -> Result<()> {
    let mut child = Command::new("cargo")
        .args(&["run", "--bin", "calc-mcp"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);
    let mut writer = stdin;

    // 初期化リクエストを送信
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {"tools": {}},
            "clientInfo": {"name": "test-client", "version": "1.0.0"}
        }
    });

    writer.write_all(format!("{}\n", init_request).as_bytes()).await?;
    writer.flush().await?;

    // レスポンスを読み取り
    let mut response_line = String::new();
    let result = timeout(Duration::from_secs(5), reader.read_line(&mut response_line)).await;
    
    match result {
        Ok(_) => {
            let response: Value = serde_json::from_str(&response_line)?;
            
            // レスポンスの検証
            assert_eq!(response["jsonrpc"], "2.0");
            assert_eq!(response["id"], 1);
            assert!(response["result"].is_object());
            
            let result = &response["result"];
            assert_eq!(result["protocolVersion"], "2024-11-05");
            assert_eq!(result["serverInfo"]["name"], "calc-mcp");
            assert_eq!(result["serverInfo"]["version"], "0.1.0");
        }
        Err(_) => {
            panic!("タイムアウト: サーバーからのレスポンスが得られませんでした");
        }
    }

    // プロセスを終了
    child.kill().await?;
    Ok(())
}

#[tokio::test]
async fn test_mcp_server_tools_list() -> Result<()> {
    let mut child = Command::new("cargo")
        .args(&["run", "--bin", "calc-mcp"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);
    let mut writer = stdin;

    // 初期化シーケンス
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {"tools": {}},
            "clientInfo": {"name": "test-client", "version": "1.0.0"}
        }
    });

    writer.write_all(format!("{}\n", init_request).as_bytes()).await?;
    writer.flush().await?;

    // 初期化レスポンスを読み取り（スキップ）
    let mut line = String::new();
    timeout(Duration::from_secs(5), reader.read_line(&mut line)).await??;

    // 初期化完了通知
    let initialized_notification = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    
    writer.write_all(format!("{}\n", initialized_notification).as_bytes()).await?;
    writer.flush().await?;

    // ツールリストリクエスト
    let tools_request = json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list"
    });

    writer.write_all(format!("{}\n", tools_request).as_bytes()).await?;
    writer.flush().await?;

    // ツールリストレスポンスを読み取り
    let mut response_line = String::new();
    let result = timeout(Duration::from_secs(5), reader.read_line(&mut response_line)).await;
    
    match result {
        Ok(_) => {
            let response: Value = serde_json::from_str(&response_line)?;
            
            // レスポンスの検証
            assert_eq!(response["jsonrpc"], "2.0");
            assert_eq!(response["id"], 2);
            assert!(response["result"]["tools"].is_array());
            
            let tools = response["result"]["tools"].as_array().unwrap();
            assert_eq!(tools.len(), 1);
            
            let calculate_tool = &tools[0];
            assert_eq!(calculate_tool["name"], "calculate");
            assert!(calculate_tool["description"].as_str().unwrap().contains("数式を評価"));
        }
        Err(_) => {
            panic!("タイムアウト: ツールリストレスポンスが得られませんでした");
        }
    }

    // プロセスを終了
    child.kill().await?;
    Ok(())
}

#[tokio::test]
async fn test_mcp_server_calculate_tool() -> Result<()> {
    let mut child = Command::new("cargo")
        .args(&["run", "--bin", "calc-mcp"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);
    let mut writer = stdin;

    // 初期化シーケンス
    let init_request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {"tools": {}},
            "clientInfo": {"name": "test-client", "version": "1.0.0"}
        }
    });

    writer.write_all(format!("{}\n", init_request).as_bytes()).await?;
    writer.flush().await?;

    // 初期化レスポンスを読み取り（スキップ）
    let mut line = String::new();
    timeout(Duration::from_secs(5), reader.read_line(&mut line)).await??;

    // 初期化完了通知
    let initialized_notification = json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    
    writer.write_all(format!("{}\n", initialized_notification).as_bytes()).await?;
    writer.flush().await?;

    // 計算ツール呼び出し
    let calculate_request = json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "calculate",
            "arguments": {
                "expression": "2 + 3 * 4"
            }
        }
    });

    writer.write_all(format!("{}\n", calculate_request).as_bytes()).await?;
    writer.flush().await?;

    // 計算結果レスポンスを読み取り
    let mut response_line = String::new();
    let result = timeout(Duration::from_secs(5), reader.read_line(&mut response_line)).await;
    
    match result {
        Ok(_) => {
            let response: Value = serde_json::from_str(&response_line)?;
            
            // レスポンスの検証
            assert_eq!(response["jsonrpc"], "2.0");
            assert_eq!(response["id"], 3);
            assert!(response["result"].is_object());
            
            let result = &response["result"];
            assert!(result["content"].is_array());
            
            let content = result["content"].as_array().unwrap();
            assert!(!content.is_empty());
            
            let text_content = &content[0];
            assert_eq!(text_content["type"], "text");
            assert!(text_content["text"].as_str().unwrap().contains("計算結果: 14"));
        }
        Err(_) => {
            panic!("タイムアウト: 計算結果レスポンスが得られませんでした");
        }
    }

    // プロセスを終了
    child.kill().await?;
    Ok(())
}