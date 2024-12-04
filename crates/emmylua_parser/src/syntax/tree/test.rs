#[cfg(test)]
mod test {
    use crate::{LuaAstNode, LuaLanguageLevel, LuaParser, ParserConfig};
    // use std::time::Instant;
    use std::thread;

    #[test]
    fn test_multithreaded_syntax_tree_traversal() {
        let code = r#"
            local a = 1
            local b = 2
            print(a + b)
        "#;
        let tree = LuaParser::parse(code, ParserConfig::default());
        let tree_arc = std::sync::Arc::new(tree);

        let mut handles = vec![];

        for i in 0..4 {
            let tree_ref = tree_arc.clone();
            let handle = thread::spawn(move || {
                let node = tree_ref.get_chunk_node();
                println!("{:?} {}", node.dump(), i);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_lua51() {
        let code = r#"
if a ~= b then
end
        "#;
        let parse_config = ParserConfig::new(LuaLanguageLevel::Lua51, None);
        let tree = LuaParser::parse(code, parse_config);
        assert_eq!(tree.get_errors().len(), 0);
    }

    #[test]
    fn test_tree_struct() {
        let code = r#"
function f()
    -- hh
    local t
end
        "#;
        let tree = LuaParser::parse(code, ParserConfig::default());
        let chunk = tree.get_chunk_node();
        println!("{:?}", chunk.dump());
    }
}