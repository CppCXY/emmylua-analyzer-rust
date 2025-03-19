#[cfg(test)]
mod test {
    use crate::{DiagnosticCode, VirtualWorkspace};

    #[test]
    fn test() {
        let mut ws = VirtualWorkspace::new();

        assert!(!ws.check_code_for(
            DiagnosticCode::RedundantParameter,
            r#"
            ---@class Test
            local Test = {}

            ---@param a string
            function Test.name(a)
            end

            Test:name("")
        "#
        ));

        assert!(!ws.check_code_for(
            DiagnosticCode::RedundantParameter,
            r#"
            ---@class Test2
            local Test = {}

            ---@param a string
            function Test.name(a)
            end

            Test.name("", "")
        "#
        ));

        assert!(!ws.check_code_for(
            DiagnosticCode::RedundantParameter,
            r#"
            ---@class A
            ---@field event fun()

            ---@type A
            local a = {
                event = function(aaa)
                end,
            }
        "#
        ));
    }

    #[test]
    fn test_dots() {
        let mut ws = VirtualWorkspace::new();

        assert!(ws.check_code_for(
            DiagnosticCode::RedundantParameter,
            r#"
            ---@class Test
            local Test = {}

            ---@param a string
            ---@param ... any
            function Test.dots(a, ...)
                print(a, ...)
            end

            Test.dots(1, 2, 3)
            Test:dots(1, 2, 3)
        "#
        ));
    }
}
