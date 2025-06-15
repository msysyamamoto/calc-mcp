use anyhow::Result;
use evalexpr::eval;
use rmcp::{tool, ServerHandler, ServiceExt, transport::stdio, model::{InitializeResult, ServerCapabilities, ProtocolVersion, Implementation}};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Calculator;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CalculateRequest {
    #[schemars(description = "計算する数式（例: \"2 + 3 * 4\", \"sqrt(25)\", \"sin(pi/2)\"）")]
    pub expression: String,
}

#[tool(tool_box)]
impl Calculator {
    #[tool(description = "数式を評価して計算結果を返します。基本的な算術演算（+, -, *, /, ^）、括弧、数学関数（sin, cos, tan, sqrt, log, etc.）をサポートしています。")]
    fn calculate(&self, #[tool(aggr)] request: CalculateRequest) -> Result<String, String> {
        match eval(&request.expression) {
            Ok(result) => Ok(format!("計算結果: {}", result)),
            Err(e) => Err(format!("計算エラー: {}", e)),
        }
    }
}

#[tool(tool_box)]
impl ServerHandler for Calculator {
    fn get_info(&self) -> InitializeResult {
        InitializeResult {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "calc-mcp".into(),
                version: "0.1.0".into(),
            },
            instructions: Some("計算機能を提供するMCPサーバです。数式を受け取って計算結果を返します。".into()),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let service = Calculator.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
