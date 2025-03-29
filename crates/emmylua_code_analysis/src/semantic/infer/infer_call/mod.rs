mod infer_require;
mod infer_setmetatable;

use std::{ops::Deref, sync::Arc};

use emmylua_parser::{LuaAstNode, LuaCallExpr, LuaExpr, LuaSyntaxKind};
use infer_require::infer_require_call;
use infer_setmetatable::infer_setmetatable_call;
use rowan::TextRange;

use crate::{
    db_index::{
        DbIndex, LuaFunctionType, LuaGenericType, LuaInstanceType, LuaMultiReturn,
        LuaOperatorMetaMethod, LuaSignatureId, LuaType, LuaTypeDeclId,
    },
    semantic::{
        generic::{instantiate_func_generic, instantiate_type_generic, TypeSubstitutor},
        overload_resolve::resolve_signature,
        InferGuard,
    },
    InFiled, LuaInferCache,
};

use super::{infer_expr, InferFailReason, InferResult};

pub fn infer_call_expr(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    call_expr: LuaCallExpr,
) -> InferResult {
    let prefix_expr = call_expr.get_prefix_expr().ok_or(InferFailReason::None)?;
    if call_expr.is_require() {
        return infer_require_call(db, cache, call_expr);
    } else if call_expr.is_setmetatable() {
        return infer_setmetatable_call(db, cache, call_expr);
    }

    check_can_infer(db, cache, &call_expr)?;

    let prefix_type = infer_expr(db, cache, prefix_expr)?;

    infer_call_result(db, cache, prefix_type, call_expr, &mut InferGuard::new())
}

fn check_can_infer(
    db: &DbIndex,
    cache: &LuaInferCache,
    call_expr: &LuaCallExpr,
) -> Result<(), InferFailReason> {
    let call_args = call_expr
        .get_args_list()
        .ok_or(InferFailReason::None)?
        .get_args();
    for arg in call_args {
        if let LuaExpr::ClosureExpr(closure) = arg {
            let sig_id = LuaSignatureId::from_closure(cache.get_file_id(), &closure);
            let signature = db
                .get_signature_index()
                .get(&sig_id)
                .ok_or(InferFailReason::None)?;
            if !signature.is_resolve_return() {
                return Err(InferFailReason::UnResolveSignatureReturn(sig_id));
            }
        }
    }

    Ok(())
}

fn infer_call_result(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    prefix_type: LuaType,
    call_expr: LuaCallExpr,
    infer_guard: &mut InferGuard,
) -> InferResult {
    let mut funcs = Vec::new();
    collect_function_type(
        db,
        cache,
        call_expr.clone(),
        prefix_type.clone(),
        infer_guard,
        &mut funcs,
    )?;

    let resolve_func = match funcs.len() {
        0 => return Err(InferFailReason::None),
        1 => funcs[0].clone(),
        _ => resolve_signature(db, cache, funcs, call_expr.clone(), false, None)?,
    };

    let rets = resolve_func.get_ret();
    let return_type = match rets.len() {
        0 => LuaType::Nil,
        1 => rets[0].clone(),
        _ => LuaType::MuliReturn(LuaMultiReturn::Multi(rets.to_vec()).into()),
    };
    unwrapp_return_type(db, cache, return_type, call_expr)
}

fn collect_function_type(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    call_expr: LuaCallExpr,
    prefix_type: LuaType,
    infer_guard: &mut InferGuard,
    funcs: &mut Vec<Arc<LuaFunctionType>>,
) -> Result<(), InferFailReason> {
    match prefix_type {
        LuaType::DocFunction(func) => {
            collect_func_by_doc_function(db, cache, func.clone(), call_expr.clone(), funcs)?
        }
        LuaType::Signature(signature_id) => {
            collect_func_by_signature(db, cache, signature_id.clone(), call_expr.clone(), funcs)?
        }
        LuaType::Def(type_def_id) => collect_func_by_custom_type(
            db,
            cache,
            type_def_id.clone(),
            call_expr.clone(),
            infer_guard,
            funcs,
        )?,
        LuaType::Ref(type_ref_id) => collect_func_by_custom_type(
            db,
            cache,
            type_ref_id.clone(),
            call_expr.clone(),
            infer_guard,
            funcs,
        )?,
        LuaType::Generic(generic) => collect_call_by_custom_generic_type(
            db,
            cache,
            &generic,
            call_expr.clone(),
            infer_guard,
            funcs,
        )?,
        LuaType::Union(union_types) => {
            for sub_type in union_types.get_types() {
                collect_function_type(
                    db,
                    cache,
                    call_expr.clone(),
                    sub_type.clone(),
                    infer_guard,
                    funcs,
                )?;
            }
        }
        _ => {}
    };

    Ok(())
}

fn collect_func_by_doc_function(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    func: Arc<LuaFunctionType>,
    call_expr: LuaCallExpr,
    funcs: &mut Vec<Arc<LuaFunctionType>>,
) -> Result<(), InferFailReason> {
    if func.contain_tpl() {
        let instantiate_func = instantiate_func_generic(db, cache, &func, call_expr)?;
        funcs.push(instantiate_func.into());
    } else {
        funcs.push(func.clone());
    };

    Ok(())
}

fn collect_func_by_signature(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    signature_id: LuaSignatureId,
    call_expr: LuaCallExpr,
    funcs: &mut Vec<Arc<LuaFunctionType>>,
) -> Result<(), InferFailReason> {
    let signature = db
        .get_signature_index()
        .get(&signature_id)
        .ok_or(InferFailReason::None)?;
    if !signature.is_resolve_return() {
        return Err(InferFailReason::UnResolveSignatureReturn(signature_id));
    }

    let mut overloads = signature.overloads.clone();

    let signature_fake_func = LuaFunctionType::new(
        signature.is_async,
        signature.is_colon_define,
        signature.get_type_params(),
        signature.get_return_types(),
    );

    overloads.push(signature_fake_func.into());

    if signature.is_generic() {
        let func = resolve_signature(db, cache, overloads, call_expr, true, None)?;
        funcs.push(func);
    } else {
        for func in overloads {
            funcs.push(func);
        }
    }

    Ok(())
}

fn collect_func_by_custom_type(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    type_id: LuaTypeDeclId,
    call_expr: LuaCallExpr,
    infer_guard: &mut InferGuard,
    funcs: &mut Vec<Arc<LuaFunctionType>>,
) -> Result<(), InferFailReason> {
    infer_guard.check(&type_id)?;
    let type_decl = db
        .get_type_index()
        .get_type_decl(&type_id)
        .ok_or(InferFailReason::None)?;
    if type_decl.is_alias() {
        let origin_type = type_decl
            .get_alias_origin(db, None)
            .ok_or(InferFailReason::None)?;
        return collect_function_type(
            db,
            cache,
            call_expr,
            origin_type.clone(),
            infer_guard,
            funcs,
        );
    } else if type_decl.is_enum() {
        return Err(InferFailReason::None);
    }

    let operator_index = db.get_operator_index();
    let operator_ids = operator_index
        .get_operators(&type_id.into(), LuaOperatorMetaMethod::Call)
        .ok_or(InferFailReason::None)?;

    for overload_id in operator_ids {
        let operator = operator_index
            .get_operator(overload_id)
            .ok_or(InferFailReason::None)?;
        let func = operator.get_operator_func().ok_or(InferFailReason::None)?;
        match func {
            LuaType::DocFunction(f) => {
                funcs.push(f.clone());
            }
            _ => {}
        }
    }

    Ok(())
}

fn collect_call_by_custom_generic_type(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    generic: &LuaGenericType,
    call_expr: LuaCallExpr,
    infer_guard: &mut InferGuard,
    funcs: &mut Vec<Arc<LuaFunctionType>>,
) -> Result<(), InferFailReason> {
    let type_id = generic.get_base_type_id();
    infer_guard.check(&type_id)?;
    let generic_params = generic.get_params();
    let substitutor = TypeSubstitutor::from_type_array(generic_params.clone());
    let type_decl = db
        .get_type_index()
        .get_type_decl(&type_id)
        .ok_or(InferFailReason::None)?;
    if type_decl.is_alias() {
        let alias_type = type_decl
            .get_alias_origin(db, Some(&substitutor))
            .ok_or(InferFailReason::None)?;
        return collect_function_type(db, cache, call_expr, alias_type.clone(), infer_guard, funcs);
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

    let func = resolve_signature(db, cache, overloads, call_expr, true, None)?;
    funcs.push(func);
    Ok(())
}

fn unwrapp_return_type(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    return_type: LuaType,
    call_expr: LuaCallExpr,
) -> InferResult {
    match &return_type {
        LuaType::Table | LuaType::Any | LuaType::Unknown => {
            let id = InFiled {
                file_id: cache.get_file_id(),
                value: call_expr.get_range(),
            };

            return Ok(LuaType::Instance(
                LuaInstanceType::new(return_type, id).into(),
            ));
        }
        LuaType::TableConst(inst) => {
            if is_need_wrap_instance(cache, &call_expr, inst) {
                let id = InFiled {
                    file_id: cache.get_file_id(),
                    value: call_expr.get_range(),
                };

                return Ok(LuaType::Instance(
                    LuaInstanceType::new(return_type.clone(), id).into(),
                ));
            }

            return Ok(return_type);
        }
        LuaType::Instance(inst) => {
            if is_need_wrap_instance(cache, &call_expr, inst.get_range()) {
                let id = InFiled {
                    file_id: cache.get_file_id(),
                    value: call_expr.get_range(),
                };

                return Ok(LuaType::Instance(
                    LuaInstanceType::new(return_type.clone(), id).into(),
                ));
            }

            return Ok(return_type);
        }

        LuaType::MuliReturn(multi) => {
            if is_last_call_expr(&call_expr) {
                return Ok(return_type);
            }

            return match multi.get_type(0) {
                Some(ty) => Ok(ty.clone()),
                None => Ok(LuaType::Nil),
            };
        }
        LuaType::Variadic(inner) => {
            if is_last_call_expr(&call_expr) {
                return Ok(LuaType::MuliReturn(
                    LuaMultiReturn::Base(inner.deref().clone()).into(),
                ));
            }

            return Ok(inner.deref().clone());
        }
        LuaType::SelfInfer => {
            let prefix_expr = call_expr.get_prefix_expr();
            if let Some(prefix_expr) = prefix_expr {
                if let LuaExpr::IndexExpr(index) = prefix_expr {
                    let self_expr = index.get_prefix_expr();
                    if let Some(self_expr) = self_expr {
                        let self_type = infer_expr(db, cache, self_expr.into());
                        return self_type;
                    }
                }
            }
        }
        _ => {}
    }

    Ok(return_type)
}

fn is_need_wrap_instance(
    cache: &mut LuaInferCache,
    call_expr: &LuaCallExpr,
    inst: &InFiled<TextRange>,
) -> bool {
    if cache.get_file_id() != inst.file_id {
        return false;
    }

    return !call_expr.get_range().contains(inst.value.start());
}

fn is_last_call_expr(call_expr: &LuaCallExpr) -> bool {
    let mut opt_parent = call_expr.syntax().parent();
    while let Some(parent) = &opt_parent {
        match parent.kind().into() {
            LuaSyntaxKind::AssignStat
            | LuaSyntaxKind::LocalStat
            | LuaSyntaxKind::ReturnStat
            | LuaSyntaxKind::TableArrayExpr
            | LuaSyntaxKind::CallArgList => {
                let next_expr = call_expr.syntax().next_sibling();
                return next_expr.is_none();
            }
            LuaSyntaxKind::TableFieldValue => {
                opt_parent = parent.parent();
            }
            _ => return false,
        }
    }

    false
}
