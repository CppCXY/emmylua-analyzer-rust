use std::collections::HashMap;

use rowan::NodeCache;

use crate::{kind::LuaLanguageLevel, lexer::LexerConfig};

pub struct ParserConfig<'cache> {
    pub level: LuaLanguageLevel,
    lexer_config: LexerConfig,
    node_cache: Option<&'cache mut NodeCache>,
    special_like: HashMap<String, SpecialFunction>,
}

impl<'cache> ParserConfig<'cache> {
    pub fn new(
        level: LuaLanguageLevel,
        node_cache: Option<&'cache mut NodeCache>,
        special_like: HashMap<String, SpecialFunction>,
    ) -> Self {
        Self {
            level,
            lexer_config: LexerConfig {
                language_level: level,
            },
            node_cache,
            special_like,
        }
    }

    pub fn lexer_config(&self) -> LexerConfig {
        self.lexer_config
    }

    pub fn support_local_attrib(&self) -> bool {
        self.level == LuaLanguageLevel::Lua54
    }

    pub fn node_cache(&mut self) -> Option<&mut NodeCache> {
        self.node_cache.as_deref_mut()
    }

    pub fn get_special_function(&self, name: &str) -> SpecialFunction {
        match name {
            "require" => SpecialFunction::Require,
            "error" => SpecialFunction::Error,
            "assert" => SpecialFunction::Assert,
            "type" => SpecialFunction::Type,
            "setmetatable" => SpecialFunction::Setmatable,
            _ => *self
                .special_like
                .get(name)
                .unwrap_or(&SpecialFunction::None),
        }
    }
}

impl<'cache> Default for ParserConfig<'cache> {
    fn default() -> Self {
        Self {
            level: LuaLanguageLevel::Lua54,
            lexer_config: LexerConfig {
                language_level: LuaLanguageLevel::Lua54,
            },
            node_cache: None,
            special_like: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecialFunction {
    None,
    Require,
    Error,
    Assert,
    Type,
    Setmatable,
}
