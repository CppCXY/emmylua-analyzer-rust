#[cfg(test)]
mod tests {

    use crate::{DiagnosticCode, VirtualWorkspace};

    #[test]
    fn test_290() {
        let mut ws = VirtualWorkspace::new();
        ws.enable_full_diagnostic();

        assert!(ws.check_code_for(
            DiagnosticCode::IncompleteSignatureDoc,
            r#"
                ---@param a string
                local function foo(_, a)
                    _ = a
                end
            "#
        ));
    }

    #[test]
    fn test_return() {
        let mut ws = VirtualWorkspace::new();
        ws.enable_full_diagnostic();

        assert!(!ws.check_code_for(
            DiagnosticCode::IncompleteSignatureDoc,
            r#"
            ---@param p number
            local function FLPR3(p, e)
                return 0
            end
            "#
        ));

        assert!(!ws.check_code_for(
            DiagnosticCode::IncompleteSignatureDoc,
            r#"
            ---@param p number
            local function FLPR3(p)
                return 0
            end
            "#
        ));

        assert!(ws.check_code_for(
            DiagnosticCode::IncompleteSignatureDoc,
            r#"
            local function FLPR3(p)
                return 0
            end
            "#
        ));
        assert!(ws.check_code_for(
            DiagnosticCode::IncompleteSignatureDoc,
            r#"
            ---
            local function FLPR3(p)
                return 0
            end
            "#
        ));

        assert!(!ws.check_code_for(
            DiagnosticCode::IncompleteSignatureDoc,
            r#"

                ---@class Test
                local Test = {}

                ---@param test Test
                function Test:add(test, c)
                end
            "#
        ));
    }

    #[test]
    fn test_global() {
        let mut ws = VirtualWorkspace::new();
        ws.enable_full_diagnostic();

        assert!(!ws.check_code_for(
            DiagnosticCode::MissingGlobalDoc,
            r#"
                function FLPR1()
                end
            "#
        ));

        assert!(!ws.check_code_for(
            DiagnosticCode::MissingGlobalDoc,
            r#"
                ---
                function FLPR1(a)
                end
            "#
        ));
        assert!(!ws.check_code_for(
            DiagnosticCode::MissingGlobalDoc,
            r#"
                ---
                function FLPR1()
                    return 1
                end
            "#
        ));

        assert!(ws.check_code_for(
            DiagnosticCode::MissingGlobalDoc,
            r#"
                ---
                function FLPR2()
                end
            "#
        ));
    }
}
