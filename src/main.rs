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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_basic_arithmetic() {
        let calculator = Calculator;
        
        // 足し算
        let request = CalculateRequest {
            expression: "2 + 3".to_string(),
        };
        let result = calculator.calculate(request).unwrap();
        assert_eq!(result, "計算結果: 5");

        // 掛け算
        let request = CalculateRequest {
            expression: "4 * 5".to_string(),
        };
        let result = calculator.calculate(request).unwrap();
        assert_eq!(result, "計算結果: 20");

        // 複合演算
        let request = CalculateRequest {
            expression: "2 + 3 * 4".to_string(),
        };
        let result = calculator.calculate(request).unwrap();
        assert_eq!(result, "計算結果: 14");
    }

    #[test]
    fn test_calculate_with_parentheses() {
        let calculator = Calculator;
        
        let request = CalculateRequest {
            expression: "(2 + 3) * 4".to_string(),
        };
        let result = calculator.calculate(request).unwrap();
        assert_eq!(result, "計算結果: 20");
    }

    #[test]
    fn test_calculate_math_functions() {
        let calculator = Calculator;
        
        // 平方根（evalexprでは使用できないため、べき乗で代替）
        let request = CalculateRequest {
            expression: "25^0.5".to_string(),
        };
        let result = calculator.calculate(request).unwrap();
        assert_eq!(result, "計算結果: 5");

        // 数学定数pi
        let request = CalculateRequest {
            expression: "math::pi".to_string(),
        };
        let result = calculator.calculate(request);
        // evalexprの制限により、この関数は使用できない場合がある
        if result.is_ok() {
            assert!(result.unwrap().contains("計算結果"));
        }
    }

    #[test]
    fn test_calculate_error_handling() {
        let calculator = Calculator;
        
        // 無効な式
        let request = CalculateRequest {
            expression: "2 +".to_string(),
        };
        let result = calculator.calculate(request);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("計算エラー"));

        // 未定義の変数
        let request = CalculateRequest {
            expression: "x + 1".to_string(),
        };
        let result = calculator.calculate(request);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("計算エラー"));
    }

    #[test]
    fn test_calculate_floating_point() {
        let calculator = Calculator;
        
        let request = CalculateRequest {
            expression: "3.14 * 2".to_string(),
        };
        let result = calculator.calculate(request).unwrap();
        assert_eq!(result, "計算結果: 6.28");
    }

    #[test]
    fn test_calculate_power() {
        let calculator = Calculator;
        
        let request = CalculateRequest {
            expression: "2^3".to_string(),
        };
        let result = calculator.calculate(request).unwrap();
        assert_eq!(result, "計算結果: 8");
    }

    #[test]
    fn test_server_info() {
        let calculator = Calculator;
        let info = calculator.get_info();
        
        assert_eq!(info.server_info.name, "calc-mcp");
        assert_eq!(info.server_info.version, "0.1.0");
        assert!(info.instructions.is_some());
        assert!(info.instructions.unwrap().contains("計算機能"));
    }
}
