#[cfg(test)]
mod tests {
    use crate::{DiagnosticCode, VirtualWorkspace};

    #[test]
    fn test_missing_return_value() {
        let mut ws = VirtualWorkspace::new();

        assert!(!ws.check_code_for(
            DiagnosticCode::MissingReturnValue,
            r#"
            ---@return number
            local function test()
                return
            end
        "#
        ));

        assert!(ws.check_code_for(
            DiagnosticCode::MissingReturnValue,
            r#"
            ---@return number
            ---@return string
            local function test()
                return 1, "2"
            end
        "#
        ));
    }

    #[test]
    fn test_missing_return_value_variadic() {
        let mut ws = VirtualWorkspace::new();

        assert!(ws.check_code_for(
            DiagnosticCode::MissingReturnValue,
            r#"
            --- @return integer?
            --- @return integer?
            function bar()
                return
            end
        "#
        ));
    }

    #[test]
    fn test_return_expr_list() {
        let mut ws = VirtualWorkspace::new();

        assert!(ws.check_code_for(
            DiagnosticCode::MissingReturnValue,
            r#"
            ---@return integer, integer
            local function foo()
            end

            ---@return integer, integer
            local function bar()
                return foo()
            end
        "#
        ));
        assert!(!ws.check_code_for(
            DiagnosticCode::MissingReturnValue,
            r#"
            ---@return integer
            local function foo()
            end

            ---@return integer, integer
            local function bar()
                return foo()
            end
        "#
        ));
    }
}
