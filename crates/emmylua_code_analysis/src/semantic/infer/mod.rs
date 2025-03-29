mod infer_binary;
mod infer_call;
mod infer_call_func;
mod infer_fail_reason;
mod infer_index;
mod infer_name;
mod infer_table;
mod infer_unary;
mod test;

use std::ops::Deref;

use emmylua_parser::{
    LuaAst, LuaAstNode, LuaClosureExpr, LuaExpr, LuaLiteralExpr, LuaLiteralToken, LuaTableExpr,
};
use infer_binary::infer_binary_expr;
use infer_call::infer_call_expr;
pub use infer_call_func::infer_call_expr_func;
pub use infer_fail_reason::InferFailReason;
use infer_index::infer_index_expr;
use infer_name::{infer_name_expr, infer_param};
use infer_table::infer_table_expr;
pub use infer_table::{infer_table_field_value_should_be, infer_table_should_be};
use infer_unary::infer_unary_expr;
use rowan::TextRange;
use smol_str::SmolStr;

use crate::{
    db_index::{DbIndex, LuaOperator, LuaOperatorMetaMethod, LuaSignatureId, LuaType},
    InFiled, LuaMultiReturn,
};

use super::{member::infer_members, CacheEntry, CacheKey, LuaInferCache};

pub type InferResult = Result<LuaType, InferFailReason>;
pub use infer_call_func::InferCallFuncResult;

pub fn infer_expr(db: &DbIndex, cache: &mut LuaInferCache, expr: LuaExpr) -> InferResult {
    let syntax_id = expr.get_syntax_id();
    let key = CacheKey::Expr(syntax_id);
    match cache.get(&key) {
        Some(cache) => match cache {
            CacheEntry::ExprCache(ty) => return Ok(ty.clone()),
            _ => return Err(InferFailReason::RecursiveInfer),
        },
        None => {}
    }

    // for @as
    let file_id = cache.get_file_id();
    let in_filed_syntax_id = InFiled::new(file_id, syntax_id);
    if let Some(force_type) = db.get_type_index().get_as_force_type(&in_filed_syntax_id) {
        cache.add_cache(&key, CacheEntry::ExprCache(force_type.clone()));
        return Ok(force_type.clone());
    }

    cache.ready_cache(&key);
    let result_type = match expr {
        LuaExpr::CallExpr(call_expr) => infer_call_expr(db, cache, call_expr),
        LuaExpr::TableExpr(table_expr) => infer_table_expr(db, cache, table_expr),
        LuaExpr::LiteralExpr(literal_expr) => infer_literal_expr(db, cache, literal_expr),
        LuaExpr::BinaryExpr(binary_expr) => infer_binary_expr(db, cache, binary_expr),
        LuaExpr::UnaryExpr(unary_expr) => infer_unary_expr(db, cache, unary_expr),
        LuaExpr::ClosureExpr(closure_expr) => infer_closure_expr(db, cache, closure_expr),
        LuaExpr::ParenExpr(paren_expr) => infer_expr(
            db,
            cache,
            paren_expr.get_expr().ok_or(InferFailReason::None)?,
        ),
        LuaExpr::NameExpr(name_expr) => infer_name_expr(db, cache, name_expr),
        LuaExpr::IndexExpr(index_expr) => infer_index_expr(db, cache, index_expr),
    };

    match &result_type {
        Ok(result_type) => cache.add_cache(&key, CacheEntry::ExprCache(result_type.clone())),
        Err(InferFailReason::None) | Err(InferFailReason::RecursiveInfer) => {
            cache.add_cache(&key, CacheEntry::ExprCache(LuaType::Unknown));
            return Ok(LuaType::Unknown);
        }
        Err(InferFailReason::FieldDotFound) => {
            if cache.get_config().analysis_phase.is_force() {
                cache.add_cache(&key, CacheEntry::ExprCache(LuaType::Nil));
                return Ok(LuaType::Nil);
            } else {
                cache.ready_cache(&key);
            }
        }
        _ => {
            cache.remove(&key);
        }
    }

    result_type
}

fn infer_literal_expr(db: &DbIndex, config: &LuaInferCache, expr: LuaLiteralExpr) -> InferResult {
    match expr.get_literal().ok_or(InferFailReason::None)? {
        LuaLiteralToken::Nil(_) => Ok(LuaType::Nil),
        LuaLiteralToken::Bool(bool) => Ok(LuaType::BooleanConst(bool.is_true())),
        LuaLiteralToken::Number(num) => {
            if num.is_int() {
                Ok(LuaType::IntegerConst(num.get_int_value()))
            } else if num.is_float() {
                Ok(LuaType::FloatConst(num.get_float_value()))
            } else {
                Ok(LuaType::Number)
            }
        }
        LuaLiteralToken::String(str) => {
            Ok(LuaType::StringConst(SmolStr::new(str.get_value()).into()))
        }
        LuaLiteralToken::Dots(_) => {
            let file_id = config.get_file_id();
            let range = expr.get_range();

            let decl_id = db
                .get_reference_index()
                .get_local_reference(&file_id)
                .and_then(|file_ref| file_ref.get_decl_id(&range));

            let decl_type = match decl_id.and_then(|id| db.get_decl_index().get_decl(&id)) {
                Some(decl) if decl.is_global() => LuaType::Any,
                Some(decl) if decl.is_param() => {
                    let base = infer_param(db, decl).unwrap_or(LuaType::Unknown);
                    LuaType::MuliReturn(LuaMultiReturn::Base(base).into())
                }
                _ => LuaType::Any, // 默认返回 Any
            };

            Ok(decl_type)
        }
        // unreachable
        _ => Ok(LuaType::Any),
    }
}

fn infer_closure_expr(_: &DbIndex, config: &LuaInferCache, closure: LuaClosureExpr) -> InferResult {
    let signature_id = LuaSignatureId::from_closure(config.get_file_id(), &closure);
    Ok(LuaType::Signature(signature_id))
}

fn get_custom_type_operator(
    db: &DbIndex,
    operand_type: LuaType,
    op: LuaOperatorMetaMethod,
) -> Option<Vec<&LuaOperator>> {
    if operand_type.is_custom_type() {
        let type_id = match operand_type {
            LuaType::Ref(type_id) => type_id,
            LuaType::Def(type_id) => type_id,
            _ => return None,
        };
        let op_ids = db.get_operator_index().get_operators(&type_id.into(), op)?;
        let operators = op_ids
            .iter()
            .filter_map(|id| db.get_operator_index().get_operator(id))
            .collect();

        Some(operators)
    } else {
        None
    }
}

pub fn infer_multi_value_adjusted_expression_types(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    exprs: &[LuaExpr],
    var_count: Option<usize>,
) -> Option<Vec<(LuaType, TextRange)>> {
    let mut value_types = Vec::new();
    for (idx, expr) in exprs.iter().enumerate() {
        let expr_type = infer_expr(db, cache, expr.clone()).ok()?;
        match expr_type {
            LuaType::MuliReturn(multi) => {
                if let Some(var_count) = var_count {
                    if idx < var_count {
                        for i in idx..var_count {
                            if let Some(typ) = multi.get_type(i - idx) {
                                value_types.push((typ.clone(), expr.get_range()));
                            } else {
                                break;
                            }
                        }
                    }
                } else {
                    match multi.deref() {
                        LuaMultiReturn::Base(base) => {
                            value_types.push((base.clone(), expr.get_range()));
                        }
                        LuaMultiReturn::Multi(vecs) => {
                            for typ in vecs {
                                value_types.push((typ.clone(), expr.get_range()));
                            }
                        }
                    }
                }

                break;
            }
            _ => value_types.push((expr_type.clone(), expr.get_range())),
        }
    }

    Some(value_types)
}

/// 从右值推断左值已绑定的类型
pub fn infer_left_value_type_from_right_value(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    expr: LuaExpr,
) -> Option<LuaType> {
    let ast = expr.syntax().parent().map(LuaAst::cast).flatten()?;

    let typ = match ast {
        LuaAst::LuaAssignStat(assign) => {
            let (vars, exprs) = assign.get_var_and_expr_list();
            let mut typ = None;
            for (idx, assign_expr) in exprs.iter().enumerate() {
                if expr == *assign_expr {
                    let var = vars.get(idx);
                    if let Some(var) = var {
                        typ = Some(infer_expr(db, cache, var.clone().into()).ok()?);
                    }
                }
            }
            typ
        }
        LuaAst::LuaTableField(table_field) => {
            let field_key = table_field.get_field_key()?;
            let table_expr = table_field.get_parent::<LuaTableExpr>()?;
            let table_type = infer_table_should_be(db, cache, table_expr.clone()).ok()?;
            let member_infos = infer_members(db, &table_type)?;
            let mut typ = None;
            for member_info in member_infos.iter() {
                if member_info.key.to_path() == field_key.get_path_part() {
                    typ = Some(member_info.typ.clone());
                }
            }
            typ
        }
        _ => None,
    };

    typ
}
