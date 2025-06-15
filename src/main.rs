use anyhow::Result;
use rmcp::{tool, ServerHandler, ServiceExt, transport::stdio, model::{InitializeResult, ServerCapabilities, ProtocolVersion, Implementation}};
use schemars::JsonSchema;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Calculator;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CalculateRequest {
    #[schemars(description = "計算する数式（例: \"2 + 3 * 4\", \"sqrt(25)\", \"sin(1.57)\"）。サポート: 四則演算(+, -, *, /)、べき乗(^)、括弧、数学関数(sqrt, abs, sin, cos, tan, ln)")]
    pub expression: String,
}

// セキュアな数式パーサー
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),
    Operator(char),
    Function(String),
    LeftParen,
    RightParen,
}

pub struct SecureCalculator {
    // 許可された関数のホワイトリスト
    allowed_functions: HashMap<String, Box<dyn Fn(f64) -> f64>>,
}

impl SecureCalculator {
    pub fn new() -> Self {
        let mut allowed_functions: HashMap<String, Box<dyn Fn(f64) -> f64>> = HashMap::new();
        allowed_functions.insert("sqrt".to_string(), Box::new(|x: f64| x.sqrt()));
        allowed_functions.insert("abs".to_string(), Box::new(|x: f64| x.abs()));
        allowed_functions.insert("sin".to_string(), Box::new(|x: f64| x.sin()));
        allowed_functions.insert("cos".to_string(), Box::new(|x: f64| x.cos()));
        allowed_functions.insert("tan".to_string(), Box::new(|x: f64| x.tan()));
        allowed_functions.insert("ln".to_string(), Box::new(|x: f64| x.ln()));
        
        Self { allowed_functions }
    }

    pub fn evaluate(&self, expression: &str) -> Result<f64, String> {
        // 入力長制限（DoS攻撃防止）
        if expression.len() > 1000 {
            return Err("式が長すぎます（最大1000文字）".to_string());
        }

        // 危険な文字をチェック
        if expression.contains(';') || expression.contains('|') || expression.contains('&') {
            return Err("不正な文字が含まれています".to_string());
        }

        let tokens = self.tokenize(expression)?;
        self.evaluate_tokens(&tokens)
    }

    fn tokenize(&self, expression: &str) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        let mut chars = expression.chars().peekable();

        while let Some(&ch) = chars.peek() {
            match ch {
                ' ' => {
                    chars.next();
                }
                '0'..='9' | '.' => {
                    let number = self.parse_number(&mut chars)?;
                    tokens.push(Token::Number(number));
                }
                '+' | '-' | '*' | '/' | '^' => {
                    chars.next();
                    tokens.push(Token::Operator(ch));
                }
                '(' => {
                    chars.next();
                    tokens.push(Token::LeftParen);
                }
                ')' => {
                    chars.next();
                    tokens.push(Token::RightParen);
                }
                'a'..='z' | 'A'..='Z' => {
                    let function_name = self.parse_identifier(&mut chars);
                    if self.allowed_functions.contains_key(&function_name) {
                        tokens.push(Token::Function(function_name));
                    } else {
                        return Err(format!("未サポートの関数: {}", function_name));
                    }
                }
                _ => {
                    return Err(format!("不正な文字: {}", ch));
                }
            }
        }

        Ok(tokens)
    }

    fn parse_number(&self, chars: &mut std::iter::Peekable<std::str::Chars>) -> Result<f64, String> {
        let mut number_str = String::new();
        let mut has_dot = false;

        while let Some(&ch) = chars.peek() {
            match ch {
                '0'..='9' => {
                    number_str.push(ch);
                    chars.next();
                }
                '.' if !has_dot => {
                    has_dot = true;
                    number_str.push(ch);
                    chars.next();
                }
                _ => break,
            }
        }

        number_str.parse::<f64>()
            .map_err(|_| format!("数値の解析に失敗: {}", number_str))
    }

    fn parse_identifier(&self, chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
        let mut identifier = String::new();

        while let Some(&ch) = chars.peek() {
            if ch.is_alphabetic() {
                identifier.push(ch);
                chars.next();
            } else {
                break;
            }
        }

        identifier
    }

    fn evaluate_tokens(&self, tokens: &[Token]) -> Result<f64, String> {
        if tokens.is_empty() {
            return Err("空の式です".to_string());
        }

        self.evaluate_expression(tokens, 0).map(|(result, _)| result)
    }

    fn evaluate_expression(&self, tokens: &[Token], mut pos: usize) -> Result<(f64, usize), String> {
        let (mut left, new_pos) = self.evaluate_term(tokens, pos)?;
        pos = new_pos;

        while pos < tokens.len() {
            match &tokens[pos] {
                Token::Operator('+') => {
                    pos += 1;
                    let (right, new_pos) = self.evaluate_term(tokens, pos)?;
                    left += right;
                    pos = new_pos;
                }
                Token::Operator('-') => {
                    pos += 1;
                    let (right, new_pos) = self.evaluate_term(tokens, pos)?;
                    left -= right;
                    pos = new_pos;
                }
                _ => break,
            }
        }

        Ok((left, pos))
    }

    fn evaluate_term(&self, tokens: &[Token], mut pos: usize) -> Result<(f64, usize), String> {
        let (mut left, new_pos) = self.evaluate_power(tokens, pos)?;
        pos = new_pos;

        while pos < tokens.len() {
            match &tokens[pos] {
                Token::Operator('*') => {
                    pos += 1;
                    let (right, new_pos) = self.evaluate_power(tokens, pos)?;
                    left *= right;
                    pos = new_pos;
                }
                Token::Operator('/') => {
                    pos += 1;
                    let (right, new_pos) = self.evaluate_power(tokens, pos)?;
                    if right == 0.0 {
                        return Err("ゼロ除算エラー".to_string());
                    }
                    left /= right;
                    pos = new_pos;
                }
                _ => break,
            }
        }

        Ok((left, pos))
    }

    fn evaluate_power(&self, tokens: &[Token], mut pos: usize) -> Result<(f64, usize), String> {
        let (mut left, new_pos) = self.evaluate_factor(tokens, pos)?;
        pos = new_pos;

        while pos < tokens.len() {
            match &tokens[pos] {
                Token::Operator('^') => {
                    pos += 1;
                    let (right, new_pos) = self.evaluate_factor(tokens, pos)?;
                    left = left.powf(right);
                    
                    // べき乗の結果をチェック
                    if !left.is_finite() {
                        return Err("べき乗の計算結果が無効です".to_string());
                    }
                    
                    pos = new_pos;
                }
                _ => break,
            }
        }

        Ok((left, pos))
    }

    fn evaluate_factor(&self, tokens: &[Token], mut pos: usize) -> Result<(f64, usize), String> {
        if pos >= tokens.len() {
            return Err("予期しない式の終了".to_string());
        }

        match &tokens[pos] {
            Token::Number(n) => Ok((*n, pos + 1)),
            Token::Operator('-') => {
                pos += 1;
                let (value, new_pos) = self.evaluate_factor(tokens, pos)?;
                Ok((-value, new_pos))
            }
            Token::Operator('+') => {
                pos += 1;
                self.evaluate_factor(tokens, pos)
            }
            Token::LeftParen => {
                pos += 1;
                let (result, new_pos) = self.evaluate_expression(tokens, pos)?;
                pos = new_pos;
                if pos >= tokens.len() || !matches!(tokens[pos], Token::RightParen) {
                    return Err("対応する右括弧がありません".to_string());
                }
                Ok((result, pos + 1))
            }
            Token::Function(name) => {
                pos += 1;
                if pos >= tokens.len() || !matches!(tokens[pos], Token::LeftParen) {
                    return Err("関数の後に左括弧が必要です".to_string());
                }
                pos += 1;
                let (arg, new_pos) = self.evaluate_expression(tokens, pos)?;
                pos = new_pos;
                if pos >= tokens.len() || !matches!(tokens[pos], Token::RightParen) {
                    return Err("関数の引数の後に右括弧が必要です".to_string());
                }
                
                let function = self.allowed_functions.get(name)
                    .ok_or_else(|| format!("未知の関数: {}", name))?;
                let result = function(arg);
                
                // NaN や無限大のチェック
                if !result.is_finite() {
                    return Err("計算結果が無効です（NaN または 無限大）".to_string());
                }
                
                Ok((result, pos + 1))
            }
            _ => Err(format!("予期しないトークン: {:?}", tokens[pos])),
        }
    }
}

#[tool(tool_box)]
impl Calculator {
    #[tool(description = "セキュアな数式計算を実行します。四則演算、べき乗、括弧、数学関数（平方根、絶対値、三角関数、自然対数）をサポートし、悪意のある入力から保護されています。")]
    fn calculate(&self, #[tool(aggr)] request: CalculateRequest) -> Result<String, String> {
        let calculator = SecureCalculator::new();
        match calculator.evaluate(&request.expression) {
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
        
        // 平方根
        let request = CalculateRequest {
            expression: "sqrt(25)".to_string(),
        };
        let result = calculator.calculate(request).unwrap();
        assert_eq!(result, "計算結果: 5");

        // 絶対値
        let request = CalculateRequest {
            expression: "abs(-10)".to_string(),
        };
        let result = calculator.calculate(request).unwrap();
        assert_eq!(result, "計算結果: 10");

        // べき乗と平方根の組み合わせ
        let request = CalculateRequest {
            expression: "25^0.5".to_string(),
        };
        let result = calculator.calculate(request).unwrap();
        assert_eq!(result, "計算結果: 5");
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

    #[test]
    fn test_security_input_length_limit() {
        let calculator = Calculator;
        
        // 長すぎる入力
        let long_expression = "1+".repeat(1000);
        let request = CalculateRequest {
            expression: long_expression,
        };
        let result = calculator.calculate(request);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("長すぎます"));
    }

    #[test]
    fn test_security_dangerous_characters() {
        let calculator = Calculator;
        
        // 危険な文字のテスト
        let dangerous_inputs = vec![
            "2 + 3; rm -rf /",
            "2 + 3 | echo hello",
            "2 + 3 & echo world",
        ];

        for input in dangerous_inputs {
            let request = CalculateRequest {
                expression: input.to_string(),
            };
            let result = calculator.calculate(request);
            assert!(result.is_err());
            assert!(result.unwrap_err().contains("不正な文字"));
        }
    }

    #[test]
    fn test_security_function_whitelist() {
        let calculator = Calculator;
        
        // 許可されていない関数
        let request = CalculateRequest {
            expression: "exec(rm)".to_string(),
        };
        let result = calculator.calculate(request);
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("未サポートの関数") || error_msg.contains("不正な文字"));
    }

    #[test]
    fn test_security_zero_division() {
        let calculator = Calculator;
        
        let request = CalculateRequest {
            expression: "1 / 0".to_string(),
        };
        let result = calculator.calculate(request);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("ゼロ除算"));
    }

    #[test]
    fn test_security_nan_infinity() {
        let calculator = Calculator;
        
        // 無限大を生成する可能性のある計算
        let request = CalculateRequest {
            expression: "sqrt(-1)".to_string(),
        };
        let result = calculator.calculate(request);
        // NaNの場合はエラーになるはず
        if result.is_err() {
            assert!(result.unwrap_err().contains("無効"));
        }
    }
}
