use std::sync::Arc;

use emmylua_parser::{LuaAstNode, LuaCallExpr};

use crate::{
    CacheEntry, CacheKey, DbIndex, LuaFunctionType, LuaGenericType, LuaOperatorMetaMethod,
    LuaSignatureId, LuaType, LuaTypeDeclId, LuaUnionType,
};

use super::{
    super::{
        generic::{instantiate_func_generic, TypeSubstitutor},
        instantiate_type_generic, resolve_signature, InferGuard, LuaInferCache,
    },
    InferFailReason,
};

pub type InferCallFuncResult = Result<Arc<LuaFunctionType>, InferFailReason>;

pub fn infer_call_expr_func(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    call_expr: LuaCallExpr,
    call_expr_type: LuaType,
    infer_guard: &mut InferGuard,
    args_count: Option<usize>,
) -> InferCallFuncResult {
    let syntax_id = call_expr.get_syntax_id();
    let key = CacheKey::Call(syntax_id, args_count, call_expr_type.clone());
    match cache.get(&key) {
        Some(cache) => match cache {
            CacheEntry::CallCache(ty) => return Ok(ty.clone()),
            _ => return Err(InferFailReason::RecursiveInfer),
        },
        None => {}
    }

    cache.ready_cache(&key);
    let result = match call_expr_type {
        LuaType::DocFunction(func) => infer_doc_function(db, cache, &func, call_expr, args_count),
        LuaType::Signature(signature_id) => infer_signature_doc_function(
            db,
            cache,
            signature_id.clone(),
            call_expr.clone(),
            args_count,
        ),
        LuaType::Def(type_def_id) => infer_type_doc_function(
            db,
            cache,
            type_def_id.clone(),
            call_expr.clone(),
            infer_guard,
            args_count,
        ),
        LuaType::Ref(type_ref_id) => infer_type_doc_function(
            db,
            cache,
            type_ref_id.clone(),
            call_expr.clone(),
            infer_guard,
            args_count,
        ),
        LuaType::Generic(generic) => infer_generic_type_doc_function(
            db,
            cache,
            &generic,
            call_expr.clone(),
            infer_guard,
            args_count,
        ),
        LuaType::Union(union) => {
            // 此时我们将其视为泛型实例化联合体
            if union.get_types().len() > 1
                && union
                    .get_types()
                    .iter()
                    .all(|t| matches!(t, LuaType::DocFunction(_)))
            {
                infer_generic_doc_function_union(db, cache, &union, call_expr, args_count)
            } else {
                Err(InferFailReason::None)
            }
        }
        _ => return Err(InferFailReason::None),
    };

    match &result {
        Ok(func_ty) => {
            cache.add_cache(&key, CacheEntry::CallCache(func_ty.clone()));
        }
        Err(r) if r.is_need_resolve() => {
            cache.remove(&key);
        }
        _ => {}
    }

    result
}

fn infer_doc_function(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    func: &LuaFunctionType,
    call_expr: LuaCallExpr,
    _: Option<usize>,
) -> InferCallFuncResult {
    if func.contain_tpl() {
        let result = instantiate_func_generic(db, cache, func, call_expr)?;
        return Ok(Arc::new(result));
    }

    return Ok(func.clone().into());
}

fn infer_generic_doc_function_union(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    union: &LuaUnionType,
    call_expr: LuaCallExpr,
    args_count: Option<usize>,
) -> InferCallFuncResult {
    let overloads = union
        .get_types()
        .iter()
        .filter_map(|typ| match typ {
            LuaType::DocFunction(f) => Some(f.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();

    resolve_signature(db, cache, overloads, call_expr.clone(), false, args_count)
}

fn infer_signature_doc_function(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    signature_id: LuaSignatureId,
    call_expr: LuaCallExpr,
    args_count: Option<usize>,
) -> InferCallFuncResult {
    let signature = db
        .get_signature_index()
        .get(&signature_id)
        .ok_or(InferFailReason::None)?;
    let overloads = &signature.overloads;
    if overloads.is_empty() {
        let mut fake_doc_function = LuaFunctionType::new(
            signature.is_async,
            signature.is_colon_define,
            signature.get_type_params(),
            vec![],
        );
        if signature.is_generic() {
            fake_doc_function = instantiate_func_generic(db, cache, &fake_doc_function, call_expr)?;
        }

        Ok(fake_doc_function.into())
    } else {
        let mut new_overloads = signature.overloads.clone();
        let fake_doc_function = Arc::new(LuaFunctionType::new(
            signature.is_async,
            signature.is_colon_define,
            signature.get_type_params(),
            vec![],
        ));
        new_overloads.push(fake_doc_function);

        resolve_signature(
            db,
            cache,
            new_overloads,
            call_expr.clone(),
            signature.is_generic(),
            args_count,
        )
    }
}

fn infer_type_doc_function(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    type_id: LuaTypeDeclId,
    call_expr: LuaCallExpr,
    infer_guard: &mut InferGuard,
    args_count: Option<usize>,
) -> InferCallFuncResult {
    infer_guard.check(&type_id)?;
    let type_decl = db
        .get_type_index()
        .get_type_decl(&type_id)
        .ok_or(InferFailReason::None)?;
    if type_decl.is_alias() {
        let origin_type = type_decl
            .get_alias_origin(db, None)
            .ok_or(InferFailReason::None)?;
        return infer_call_expr_func(
            db,
            cache,
            call_expr,
            origin_type.clone(),
            infer_guard,
            args_count,
        );
    } else if type_decl.is_enum() {
        return Err(InferFailReason::None);
    }

    let operator_index = db.get_operator_index();
    let operator_ids = operator_index
        .get_operators(&type_id.into(), LuaOperatorMetaMethod::Call)
        .ok_or(InferFailReason::None)?;
    let mut overloads = Vec::new();
    for overload_id in operator_ids {
        let operator = operator_index
            .get_operator(overload_id)
            .ok_or(InferFailReason::None)?;
        let func = operator.get_operator_func().ok_or(InferFailReason::None)?;
        match func {
            LuaType::DocFunction(f) => {
                overloads.push(f.clone());
            }
            _ => {}
        }
    }

    resolve_signature(db, cache, overloads, call_expr.clone(), false, args_count)
}

fn infer_generic_type_doc_function(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    generic: &LuaGenericType,
    call_expr: LuaCallExpr,
    infer_guard: &mut InferGuard,
    args_count: Option<usize>,
) -> InferCallFuncResult {
    let type_id = generic.get_base_type_id();
    infer_guard.check(&type_id)?;
    let generic_params = generic.get_params();
    let substitutor = TypeSubstitutor::from_type_array(generic_params.clone());

    let type_decl = db
        .get_type_index()
        .get_type_decl(&type_id)
        .ok_or(InferFailReason::None)?;
    if type_decl.is_alias() {
        let origin_type = type_decl
            .get_alias_origin(db, Some(&substitutor))
            .ok_or(InferFailReason::None)?;
        return infer_call_expr_func(
            db,
            cache,
            call_expr,
            origin_type.clone(),
            infer_guard,
            args_count,
        );
    } else if type_decl.is_enum() {
        return Err(InferFailReason::None);
    }

    let operator_index = db.get_operator_index();
    let operator_ids = operator_index
        .get_operators(&type_id.into(), LuaOperatorMetaMethod::Call)
        .ok_or(InferFailReason::None)?;
    let mut overloads = Vec::new();
    for overload_id in operator_ids {
        let operator = operator_index
            .get_operator(overload_id)
            .ok_or(InferFailReason::None)?;
        let func = operator.get_operator_func().ok_or(InferFailReason::None)?;
        let new_f = instantiate_type_generic(db, func, &substitutor);
        match new_f {
            LuaType::DocFunction(f) => {
                overloads.push(f.clone());
            }
            _ => {}
        }
    }

    resolve_signature(db, cache, overloads, call_expr.clone(), false, args_count)
}
