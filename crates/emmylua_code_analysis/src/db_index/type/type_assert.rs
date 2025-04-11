use std::sync::Arc;

use crate::{infer_expr, DbIndex, InferFailReason, LuaInferCache};
use emmylua_parser::{LuaAstNode, LuaExpr, LuaSyntaxId, LuaSyntaxNode};
use rowan::TextSize;

use super::{type_ops::TypeOps, LuaType};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum TypeAssertion {
    Exist,
    NotExist,
    Narrow(LuaType),
    Add(LuaType),
    Remove(LuaType),
    Reassign((LuaSyntaxId, i32))
}

#[allow(unused)]
impl TypeAssertion {
    pub fn get_negation(&self) -> Option<TypeAssertion> {
        match self {
            TypeAssertion::Exist => Some(TypeAssertion::NotExist),
            TypeAssertion::NotExist => Some(TypeAssertion::Exist),
            TypeAssertion::Narrow(t) => Some(TypeAssertion::Remove(t.clone())),
            TypeAssertion::Remove(t) => Some(TypeAssertion::Narrow(t.clone())),
            _ => None,
        }
    }

    pub fn tighten_type(
        &self,
        db: &DbIndex,
        config: &mut LuaInferCache,
        root: &LuaSyntaxNode,
        source: LuaType,
    ) -> Result<LuaType, InferFailReason> {
        match self {
            TypeAssertion::Exist => Ok(TypeOps::Remove.apply(&source, &LuaType::Nil)),
            TypeAssertion::NotExist => Ok(TypeOps::NarrowFalseOrNil.apply_source(&source)),
            TypeAssertion::Narrow(t) => Ok(TypeOps::Narrow.apply(&source, t)),
            TypeAssertion::Add(lua_type) => Ok(TypeOps::Union.apply(&source, lua_type)),
            TypeAssertion::Remove(lua_type) => Ok(TypeOps::Remove.apply(&source, lua_type)),
            TypeAssertion::Reassign((syntax_id, idx)) => {
                let expr = LuaExpr::cast(
                    syntax_id
                        .to_node_from_root(root)
                        .ok_or(InferFailReason::None)?,
                )
                .ok_or(InferFailReason::None)?;
                let expr_type = infer_expr(db, config, expr)?;
                let expr_type = match &expr_type {
                    LuaType::MuliReturn(multi) => {
                        multi.get_type(*idx as usize).unwrap_or(&LuaType::Nil)
                    }
                    t => t,
                };
                Ok(TypeOps::Narrow.apply(&source, &expr_type))
            }
            _ => Ok(source),
        }
    }

    pub fn is_reassign(&self) -> bool {
        matches!(self, TypeAssertion::Reassign(_))
    }
}
