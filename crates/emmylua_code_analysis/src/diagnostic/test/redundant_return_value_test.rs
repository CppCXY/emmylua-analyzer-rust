#[cfg(test)]
mod tests {
    use crate::{DiagnosticCode, VirtualWorkspace};

    #[test]
    fn test_dots() {
        let mut ws = VirtualWorkspace::new();

        assert!(ws.check_code_for(
            DiagnosticCode::RedundantReturnValue,
            r#"
            ---@return number, any...
            local function test()
                return 1, 2, 3
            end
        "#
        ));
    }

    #[test]
    fn test_redundant_return_value() {
        let mut ws = VirtualWorkspace::new();

        assert!(!ws.check_code_for(
            DiagnosticCode::RedundantReturnValue,
            r#"
            ---@return number
            local function test()
                return 1, 2
            end
        "#
        ));
    }

    #[test]
    fn test_not_return_anno() {
        let mut ws = VirtualWorkspace::new();

        assert!(ws.check_code_for(
            DiagnosticCode::MissingReturnValue,
            r#"
            local function baz()
                if true then
                    return 
                end
                return 1
            end
        "#
        ));

        assert!(ws.check_code_for(
            DiagnosticCode::RedundantReturnValue,
            r#"
            function bar(a)
                return tonumber(a)
            end
        "#
        ));
    }

    #[test]
    fn test_return_expr_list() {
        let mut ws = VirtualWorkspace::new();

        assert!(ws.check_code_for(
            DiagnosticCode::RedundantReturnValue,
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
            DiagnosticCode::RedundantReturnValue,
            r#"
            ---@return integer, integer, integer
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
