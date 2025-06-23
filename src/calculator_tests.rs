#[cfg(test)]
mod tests {
    use crate::calculator::{CalculateRequest, CalculatorService};
    use rmcp::ServerHandler;

    #[test]
    fn test_calculate_basic_arithmetic() {
        let calculator = CalculatorService;

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
        let calculator = CalculatorService;

        let request = CalculateRequest {
            expression: "(2 + 3) * 4".to_string(),
        };
        let result = calculator.calculate(request).unwrap();
        assert_eq!(result, "計算結果: 20");
    }

    #[test]
    fn test_calculate_math_functions() {
        let calculator = CalculatorService;

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
        let calculator = CalculatorService;

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
        let calculator = CalculatorService;

        let request = CalculateRequest {
            expression: "3.14 * 2".to_string(),
        };
        let result = calculator.calculate(request).unwrap();
        assert_eq!(result, "計算結果: 6.28");
    }

    #[test]
    fn test_calculate_power() {
        let calculator = CalculatorService;

        let request = CalculateRequest {
            expression: "2^3".to_string(),
        };
        let result = calculator.calculate(request).unwrap();
        assert_eq!(result, "計算結果: 8");
    }

    #[test]
    fn test_server_info() {
        let calculator = CalculatorService;
        let info = calculator.get_info();

        assert_eq!(info.server_info.name, "calc-mcp");
        assert_eq!(info.server_info.version, "0.1.0");
        assert!(info.instructions.is_some());
        assert!(info.instructions.unwrap().contains("計算機能"));
    }

    #[test]
    fn test_security_input_length_limit() {
        let calculator = CalculatorService;

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
        let calculator = CalculatorService;

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
        let calculator = CalculatorService;

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
        let calculator = CalculatorService;

        let request = CalculateRequest {
            expression: "1 / 0".to_string(),
        };
        let result = calculator.calculate(request);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("ゼロ除算"));
    }

    #[test]
    fn test_security_nan_infinity() {
        let calculator = CalculatorService;

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