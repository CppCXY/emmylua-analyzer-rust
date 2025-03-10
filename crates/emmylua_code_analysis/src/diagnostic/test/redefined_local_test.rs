#[cfg(test)]
mod tests {
    use crate::{DiagnosticCode, VirtualWorkspace};

    #[test]
    fn test() {
        let mut ws = VirtualWorkspace::new();
        assert!(!ws.check_code_for(
            DiagnosticCode::RedefinedLocal,
            r#"
                local x = 1
                local x = 2
        "#
        ));

        assert!(!ws.check_code_for(
            DiagnosticCode::RedefinedLocal,
            r#"
            local function aaa()
            end

            local function aaa()
            end
        "#
        ));

        assert!(ws.check_code_for(
            DiagnosticCode::RedefinedLocal,
            r#"
            local function aaa(a, b)
            end
            local a = 2
            local b = 2
        "#
        ));

        assert!(ws.check_code_for(
            DiagnosticCode::RedefinedLocal,
            r#"
            ---@class Test
            local Test = {}

            function Test:test(c)
            end

            local c = 1
        "#
        ));
    }

    #[test]
    fn test_do_end() {
        let mut ws = VirtualWorkspace::new();
        assert!(ws.check_code_for(
            DiagnosticCode::RedefinedLocal,
            r#"
                do
                    local c = 1
                end

                do
                    local c = 1
                end
                local c = 1
        "#
        ));
    }

    #[test]
    fn test_for() {
        let mut ws = VirtualWorkspace::new();
        assert!(ws.check_code_for(
            DiagnosticCode::RedefinedLocal,
            r#"
            local function aaa()
                for a = 1, 1 do
                    local fora = 1
                end
                local fora = 1
            end
        "#
        ));
    }
}
