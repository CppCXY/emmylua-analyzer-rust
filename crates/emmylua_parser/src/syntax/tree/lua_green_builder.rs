use rowan::{GreenNode, NodeCache};

use crate::{
    kind::{LuaSyntaxKind, LuaTokenKind},
    text::SourceRange,
};

#[derive(Debug, Clone)]
enum LuaGreenElement {
    None,
    Node {
        kind: LuaSyntaxKind,
        children: Vec<usize>,
    },
    Token {
        kind: LuaTokenKind,
        range: SourceRange,
    },
}
/// A builder for a green tree.
#[derive(Default, Debug)]
pub struct LuaGreenNodeBuilder<'cache> {
    parents: Vec<(LuaSyntaxKind, usize)>,
    children: Vec<usize>, /*index for elements*/
    elements: Vec<LuaGreenElement>,
    builder: rowan::GreenNodeBuilder<'cache>,
}

impl LuaGreenNodeBuilder<'_> {
    /// Creates new builder.
    pub fn new() -> LuaGreenNodeBuilder<'static> {
        LuaGreenNodeBuilder::default()
    }

    pub fn with_cache(cache: &mut NodeCache) -> LuaGreenNodeBuilder<'_> {
        LuaGreenNodeBuilder {
            parents: Vec::new(),
            children: Vec::new(),
            elements: Vec::new(),
            builder: rowan::GreenNodeBuilder::with_cache(cache),
        }
    }

    #[inline]
    pub fn token(&mut self, kind: LuaTokenKind, range: SourceRange) {
        let len = self.elements.len();
        self.elements.push(LuaGreenElement::Token { kind, range });
        self.children.push(len);
    }

    #[inline]
    pub fn start_node(&mut self, kind: LuaSyntaxKind) {
        let len = self.children.len();
        self.parents.push((kind, len));
    }

    #[inline]
    pub fn finish_node(&mut self) {
        if self.parents.is_empty() {
            return;
        }

        let (parent_kind, first_start) = self.parents.pop().unwrap();
        let mut child_start = first_start;
        let mut child_end = self.children.len() - 1;
        let child_count = self.children.len();
        let green = if parent_kind != LuaSyntaxKind::Block && parent_kind != LuaSyntaxKind::Chunk {
            while child_start < child_count {
                if self.is_trivia(self.children[child_start]) {
                    child_start += 1;
                } else {
                    break;
                }
            }
            while child_end >= child_start {
                if self.is_trivia(self.children[child_end]) {
                    child_end -= 1;
                } else {
                    break;
                }
            }

            let children = self
                .children
                .drain(child_start..=child_end)
                .collect::<Vec<_>>();
            LuaGreenElement::Node {
                kind: parent_kind,
                children,
            }
        } else {
            let children = self.children.drain(first_start..).collect::<Vec<_>>();
            LuaGreenElement::Node {
                kind: parent_kind,
                children,
            }
        };

        let pos = self.elements.len();
        self.elements.push(green);

        if child_end + 1 < child_count {
            self.children.insert(child_start, pos);
        } else {
            self.children.push(pos);
        }
    }

    fn is_trivia(&self, pos: usize) -> bool {
        if let Some(element) = self.elements.get(pos) {
            match element {
                LuaGreenElement::Token {
                    kind: LuaTokenKind::TkWhitespace | LuaTokenKind::TkEndOfLine,
                    ..
                } => true,
                LuaGreenElement::Node {
                    kind: LuaSyntaxKind::Comment,
                    ..
                } => true,
                _ => false,
            }
        } else {
            false
        }
    }

    fn build_rowan_green(&mut self, parent: usize, text: &str) {
        let element = std::mem::replace(&mut self.elements[parent], LuaGreenElement::None);
        match element {
            LuaGreenElement::Node { kind, children } => {
                self.builder.start_node(kind.into());

                for child in children {
                    self.build_rowan_green(child, text);
                }

                self.builder.finish_node();
            }
            LuaGreenElement::Token { kind, range } => {
                let start = range.start_offset;
                let end = range.end_offset();
                let text = &text[start..end];
                self.builder.token(kind.into(), text)
            }
            _ => {}
        }
    }

    #[inline]
    pub fn finish(mut self, text: &str) -> GreenNode {
        let root_pos = self.children.pop().unwrap();
        self.build_rowan_green(root_pos, text);

        self.builder.finish()
    }
}