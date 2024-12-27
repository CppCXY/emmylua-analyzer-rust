use code_analysis::LuaDocument;
use emmylua_parser::LuaSyntaxToken;
use lsp_types::{SemanticToken, SemanticTokenModifier, SemanticTokenType};
use std::{collections::HashMap, vec::Vec};

pub const SEMANTIC_TOKEN_TYPES: &[SemanticTokenType] = &[
    SemanticTokenType::NAMESPACE,
    SemanticTokenType::TYPE,
    SemanticTokenType::CLASS,
    SemanticTokenType::ENUM,
    SemanticTokenType::PARAMETER,
    SemanticTokenType::VARIABLE,
    SemanticTokenType::FUNCTION,
    SemanticTokenType::KEYWORD,
    SemanticTokenType::OPERATOR,
    SemanticTokenType::STRING,
    SemanticTokenType::NUMBER,
    SemanticTokenType::REGEXP,
];

pub const SEMANTIC_TOKEN_MODIFIERS: &[SemanticTokenModifier] = &[
    SemanticTokenModifier::MODIFICATION,
    SemanticTokenModifier::DECLARATION,
    SemanticTokenModifier::DEFINITION,
    SemanticTokenModifier::READONLY,
    SemanticTokenModifier::STATIC,
    SemanticTokenModifier::ABSTRACT,
    SemanticTokenModifier::DEPRECATED,
];

#[derive(Debug)]
struct SemanticTokenData {
    line: u32,
    col: u32,
    length: u32,
    typ: u32,
    modifiers: u32,
}

#[allow(unused)]
#[derive(Debug)]
pub struct SemanticBuilder<'a> {
    document: &'a LuaDocument<'a>,
    multi_line_support: bool,
    type_to_id: HashMap<SemanticTokenType, u32>,
    modifier_to_id: HashMap<SemanticTokenModifier, u32>,
    data: Vec<SemanticTokenData>,
}

#[allow(unused)]
impl<'a> SemanticBuilder<'a> {
    pub fn new(
        document: &'a LuaDocument,
        multi_line_support: bool,
        types: Vec<SemanticTokenType>,
        modifier: Vec<SemanticTokenModifier>,
    ) -> Self {
        let mut type_to_id = HashMap::new();
        for (i, ty) in types.into_iter().enumerate() {
            type_to_id.insert(ty, i as u32);
        }
        let mut modifier_to_id = HashMap::new();
        for (i, modifier) in modifier.into_iter().enumerate() {
            modifier_to_id.insert(modifier, i as u32);
        }

        Self {
            document,
            multi_line_support,
            type_to_id,
            modifier_to_id,
            data: Vec::new(),
        }
    }

    fn push_data(&mut self, token: LuaSyntaxToken, typ: u32, modifiers: u32) -> Option<()> {
        let range = token.text_range();
        let lsp_range = self.document.to_lsp_range(range)?;
        let start_line = lsp_range.start.line;
        let start_col = lsp_range.start.character;
        let end_line = lsp_range.end.line;

        if (!self.multi_line_support && start_line != end_line) {
            self.data.push(SemanticTokenData {
                line: start_line,
                col: start_col,
                length: 9999,
                typ,
                modifiers,
            });

            for i in start_line + 1..end_line - 1 {
                self.data.push(SemanticTokenData {
                    line: i,
                    col: 0,
                    length: 9999,
                    typ,
                    modifiers,
                });
            }

            self.data.push(SemanticTokenData {
                line: end_line,
                col: 0,
                length: lsp_range.end.character,
                typ,
                modifiers,
            });
        } else {
            self.data.push(SemanticTokenData {
                line: start_line as u32,
                col: start_col as u32,
                length: token.text().chars().count() as u32,
                typ,
                modifiers,
            });
        }

        Some(())
    }

    pub fn push(&mut self, token: LuaSyntaxToken, ty: SemanticTokenType) -> Option<()> {
        self.push_data(token, *self.type_to_id.get(&ty)?, 0);
        Some(())
    }

    pub fn push_with_modifier(
        &mut self,
        token: LuaSyntaxToken,
        ty: SemanticTokenType,
        modifier: SemanticTokenModifier,
    ) -> Option<()> {
        let typ = *self.type_to_id.get(&ty)?;
        let modifier = 1 << *self.modifier_to_id.get(&modifier)?;
        self.push_data(token, typ, modifier);
        Some(())
    }

    pub fn push_with_modifiers(
        &mut self,
        token: LuaSyntaxToken,
        ty: SemanticTokenType,
        modifiers: Vec<SemanticTokenModifier>,
    ) -> Option<()> {
        let typ = *self.type_to_id.get(&ty)?;
        let mut modifier = 0;
        for m in modifiers {
            modifier |= 1 << *self.modifier_to_id.get(&m)?;
        }
        self.push_data(token, typ, modifier);

        Some(())
    }

    pub fn build(self) -> Vec<SemanticToken> {
        let mut data = self.data;
        data.sort_by(|a, b| {
            let line1 = a.line;
            let line2 = b.line;
            if line1 == line2 {
                let character1 = a.col;
                let character2 = b.col;
                return character1.cmp(&character2);
            }
            line1.cmp(&line2)
        });

        let mut result = Vec::with_capacity(data.len());
        let mut prev_line = 0;
        let mut prev_col = 0;

        for token_data in data {
            let line_diff = token_data.line - prev_line;
            if line_diff != 0 {
                prev_col = 0;
            }
            let col_diff = token_data.col - prev_col;

            result.push(SemanticToken {
                delta_line: line_diff,
                delta_start: col_diff,
                length: token_data.length,
                token_type: token_data.typ,
                token_modifiers_bitset: token_data.modifiers,
            });

            prev_line = token_data.line;
            prev_col = token_data.col;
        }

        result
    }
}
