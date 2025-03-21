#[cfg(test)]
mod tests {

    use lsp_types::{CompletionItemKind, CompletionTriggerKind};

    use crate::handlers::completion::test::{CompletionVirtualWorkspace, VirtualCompletionItem};

    #[test]
    fn test_1() {
        let mut ws = CompletionVirtualWorkspace::new();

        assert!(ws.check_completion(
            r#"
            local zabcde
            za<??>
        "#,
            vec![VirtualCompletionItem {
                label: "zabcde".to_string(),
                kind: CompletionItemKind::VARIABLE,
            }],
        ));
    }

    #[test]
    fn test_2() {
        let mut ws = CompletionVirtualWorkspace::new();
        assert!(ws.check_completion(
            r#"
            ---@overload fun(event: "AAA", callback: fun(trg: string, data: number)): number
            ---@overload fun(event: "BBB", callback: fun(trg: string, data: string)): string
            local function test(event, callback)
            end

            test("AAA", function(trg, data)
            <??>
            end)
        "#,
            vec![
                VirtualCompletionItem {
                    label: "data".to_string(),
                    kind: CompletionItemKind::VARIABLE,
                },
                VirtualCompletionItem {
                    label: "trg".to_string(),
                    kind: CompletionItemKind::VARIABLE,
                },
                VirtualCompletionItem {
                    label: "test".to_string(),
                    kind: CompletionItemKind::FUNCTION,
                },
            ],
        ));

        // 主动触发补全
        assert!(ws.check_completion(
            r#"
            ---@overload fun(event: "AAA", callback: fun(trg: string, data: number)): number
            ---@overload fun(event: "BBB", callback: fun(trg: string, data: string)): string
            local function test(event, callback)
            end
            test(<??>)
        "#,
            vec![
                VirtualCompletionItem {
                    label: "\"AAA\"".to_string(),
                    kind: CompletionItemKind::ENUM_MEMBER,
                },
                VirtualCompletionItem {
                    label: "\"BBB\"".to_string(),
                    kind: CompletionItemKind::ENUM_MEMBER,
                },
                VirtualCompletionItem {
                    label: "test".to_string(),
                    kind: CompletionItemKind::FUNCTION,
                },
            ],
        ));

        // 被动触发补全
        assert!(ws.check_completion_with_kind(
            r#"
            ---@overload fun(event: "AAA", callback: fun(trg: string, data: number)): number
            ---@overload fun(event: "BBB", callback: fun(trg: string, data: string)): string
            local function test(event, callback)
            end
            test(<??>)
        "#,
            vec![
                VirtualCompletionItem {
                    label: "\"AAA\"".to_string(),
                    kind: CompletionItemKind::ENUM_MEMBER,
                },
                VirtualCompletionItem {
                    label: "\"BBB\"".to_string(),
                    kind: CompletionItemKind::ENUM_MEMBER,
                },
            ],
            CompletionTriggerKind::TRIGGER_CHARACTER,
        ));
    }

    #[test]
    fn test_3() {
        let mut ws = CompletionVirtualWorkspace::new();
        // 被动触发补全
        assert!(ws.check_completion_with_kind(
            r#"
            ---@class Test
            ---@field event fun(a: "A", b: number)
            ---@field event fun(a: "B", b: string)
            local Test = {}
            Test.event(<??>)
        "#,
            vec![
                VirtualCompletionItem {
                    label: "\"A\"".to_string(),
                    kind: CompletionItemKind::ENUM_MEMBER,
                },
                VirtualCompletionItem {
                    label: "\"B\"".to_string(),
                    kind: CompletionItemKind::ENUM_MEMBER,
                },
            ],
            CompletionTriggerKind::TRIGGER_CHARACTER,
        ));

        // 主动触发补全
        assert!(ws.check_completion(
            r#"
                    ---@class Test
                    ---@field event fun(a: "A", b: number)
                    ---@field event fun(a: "B", b: string)
                    local Test = {}
                    Test.event(<??>)
                "#,
            vec![
                VirtualCompletionItem {
                    label: "\"A\"".to_string(),
                    kind: CompletionItemKind::ENUM_MEMBER,
                },
                VirtualCompletionItem {
                    label: "\"B\"".to_string(),
                    kind: CompletionItemKind::ENUM_MEMBER,
                },
                VirtualCompletionItem {
                    label: "Test".to_string(),
                    kind: CompletionItemKind::CLASS,
                },
            ],
        ));

        assert!(ws.check_completion(
            r#"
                    ---@class Test
                    ---@field event fun(a: "A", b: number)
                    ---@field event fun(a: "B", b: string)
                    local Test = {}
                    Test.<??>
                "#,
            vec![VirtualCompletionItem {
                label: "event".to_string(),
                kind: CompletionItemKind::FUNCTION,
            },],
        ));
    }
}
