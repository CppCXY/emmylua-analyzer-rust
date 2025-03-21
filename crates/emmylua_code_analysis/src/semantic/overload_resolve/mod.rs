use std::sync::Arc;

use emmylua_parser::LuaCallExpr;

use crate::db_index::{DbIndex, LuaFunctionType, LuaType};

use super::{
    generic::instantiate_func_generic, infer_expr, type_check::check_type_compact, LuaInferCache,
};

pub fn resolve_signature(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    overloads: Vec<Arc<LuaFunctionType>>,
    call_expr: LuaCallExpr,
    is_generic: bool,
    arg_count: Option<usize>,
) -> Option<Arc<LuaFunctionType>> {
    let args = call_expr.get_args_list()?;
    let mut expr_types = Vec::new();
    for arg in args.get_args() {
        expr_types.push(infer_expr(db, cache, arg)?);
    }
    if is_generic {
        return resolve_signature_by_generic(
            db, cache, overloads, call_expr, expr_types, arg_count,
        );
    } else {
        return resolve_signature_by_args(
            db,
            overloads,
            expr_types,
            call_expr.is_colon_call(),
            arg_count,
        );
    }
}

fn resolve_signature_by_generic(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    overloads: Vec<Arc<LuaFunctionType>>,
    call_expr: LuaCallExpr,
    expr_types: Vec<LuaType>,
    arg_count: Option<usize>,
) -> Option<Arc<LuaFunctionType>> {
    let mut max_match: usize = 0;
    let mut matched_func: Option<Arc<LuaFunctionType>> = None;
    let mut instantiate_funcs = Vec::new();
    for func in overloads {
        let instantiate_func = instantiate_func_generic(db, cache, &func, call_expr.clone())?;
        instantiate_funcs.push(Arc::new(instantiate_func));
    }

    for func in &instantiate_funcs {
        let params = func.get_params();
        let mut match_count = 0;
        if params.len() < arg_count.unwrap_or(0) {
            continue;
        }

        for (i, param) in params.iter().enumerate() {
            if i >= expr_types.len() {
                break;
            }

            let param_type = param.1.clone().unwrap_or(LuaType::Any);
            let expr_type = &expr_types[i];
            if param_type == LuaType::Any {
                match_count += 1;
            } else if check_type_compact(db, &param_type, expr_type).is_ok() {
                match_count += 1;
            }
        }
        if match_count > max_match {
            max_match = match_count;
            matched_func = Some(func.clone());
        }
    }

    if matched_func.is_none() && !instantiate_funcs.is_empty() {
        matched_func = Some(instantiate_funcs.last().cloned().unwrap());
    }

    matched_func
}

fn resolve_signature_by_args(
    db: &DbIndex,
    overloads: Vec<Arc<LuaFunctionType>>,
    expr_types: Vec<LuaType>,
    is_colon_call: bool,
    arg_count: Option<usize>,
) -> Option<Arc<LuaFunctionType>> {
    let mut max_match: i32 = -1;
    let mut matched_func: Option<Arc<LuaFunctionType>> = None;

    for func in &overloads {
        let params = func.get_params();
        if params.len() < arg_count.unwrap_or(0) {
            continue;
        }

        let jump_param = if is_colon_call && !func.is_colon_define() {
            1
        } else {
            0
        };

        let mut match_count = 0;

        for (i, param) in params.iter().enumerate() {
            if i == 0 && jump_param > 0 {
                continue;
            }
            if expr_types.len() <= i - jump_param {
                break;
            }

            let param_type = param.1.clone().unwrap_or(LuaType::Any);
            let expr_type = &expr_types[i - jump_param];
            if param_type == LuaType::Any || check_type_compact(db, &param_type, expr_type).is_ok()
            {
                match_count += 1;
            }
        }

        if match_count > max_match {
            max_match = match_count;
            matched_func = Some(func.clone());
        }
    }

    matched_func.or_else(|| overloads.last().cloned())
}
