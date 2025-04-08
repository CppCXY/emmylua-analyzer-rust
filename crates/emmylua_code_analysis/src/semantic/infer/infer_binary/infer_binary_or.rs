use emmylua_parser::LuaExpr;

use crate::{check_type_compact, semantic::infer::InferResult, DbIndex, LuaType, TypeOps};

pub fn special_or_rule(
    db: &DbIndex,
    left_type: &LuaType,
    right_type: &LuaType,
    _: LuaExpr,
    right_expr: LuaExpr,
) -> Option<LuaType> {
    match right_expr {
        // workaround for x or error('')
        LuaExpr::CallExpr(call_expr) => {
            if call_expr.is_error() {
                return Some(TypeOps::Remove.apply(&left_type, &LuaType::Nil));
            }
        }
        LuaExpr::TableExpr(table_expr) => {
            if table_expr.is_empty() && check_type_compact(db, &left_type, &LuaType::Table).is_ok()
            {
                return Some(TypeOps::Remove.apply(&left_type, &LuaType::Nil));
            }
        }
        LuaExpr::LiteralExpr(_) => {
            if check_type_compact(db, &left_type, &right_type).is_ok() {
                return Some(TypeOps::Remove.apply(&left_type, &LuaType::Nil));
            }
        }

        _ => {}
    }

    None
}

pub fn infer_binary_expr_or(left: LuaType, right: LuaType) -> InferResult {
    if left.is_always_truthy() {
        return Ok(left);
    } else if left.is_always_falsy() {
        return Ok(right);
    }

    // if check_type_compact(db, source, compact_type)

    Ok(TypeOps::Union.apply(&TypeOps::Remove.apply(&left, &LuaType::Nil), &right))
}
