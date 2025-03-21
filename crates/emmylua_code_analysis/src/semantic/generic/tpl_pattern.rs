use std::ops::Deref;

use emmylua_parser::LuaSyntaxNode;
use smol_str::SmolStr;

use crate::{
    db_index::{DbIndex, LuaGenericType, LuaType},
    semantic::{member::infer_member_map, LuaInferCache},
    LuaFunctionType, LuaMemberKey, LuaMemberOwner, LuaMultiReturn, LuaTupleType, LuaUnionType,
};

use super::type_substitutor::TypeSubstitutor;

pub fn tpl_pattern_match_args(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    func_param_types: &[LuaType],
    call_arg_types: &[LuaType],
    root: &LuaSyntaxNode,
    substitutor: &mut TypeSubstitutor,
) -> Option<()> {
    for (i, func_param_type) in func_param_types.iter().enumerate() {
        let call_arg_type = if i < call_arg_types.len() {
            &call_arg_types[i]
        } else {
            continue;
        };

        match (func_param_type, call_arg_type) {
            (LuaType::Variadic(multi_tpl), _) => {
                variadic_tpl_pattern_match(multi_tpl, &call_arg_types[i..], substitutor);
                break;
            }
            (_, LuaType::MuliReturn(multi_return)) => {
                multi_param_tpl_pattern_match_multi_return(
                    db,
                    cache,
                    &func_param_types[i..],
                    multi_return,
                    root,
                    substitutor,
                );
                break;
            }
            _ => {
                tpl_pattern_match(db, cache, root, func_param_type, call_arg_type, substitutor);
            }
        }
    }

    Some(())
}

fn multi_param_tpl_pattern_match_multi_return(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    func_param_types: &[LuaType],
    multi_return: &LuaMultiReturn,
    root: &LuaSyntaxNode,
    substitutor: &mut TypeSubstitutor,
) -> Option<()> {
    match &multi_return {
        LuaMultiReturn::Base(base) => {
            let mut call_arg_types = Vec::new();
            for param in func_param_types {
                if param.is_variadic() {
                    call_arg_types.push(LuaType::MuliReturn(multi_return.clone().into()));
                    break;
                } else {
                    call_arg_types.push(base.clone());
                }
            }

            tpl_pattern_match_args(
                db,
                cache,
                func_param_types,
                &call_arg_types,
                root,
                substitutor,
            )
        }
        LuaMultiReturn::Multi(_) => {
            let mut call_arg_types = Vec::new();
            for (i, param) in func_param_types.iter().enumerate() {
                let return_type = multi_return.get_type(i);
                if return_type.is_none() {
                    break;
                }

                if param.is_variadic() {
                    call_arg_types.push(LuaType::MuliReturn(
                        multi_return.get_new_multi_from(i).into(),
                    ));
                    break;
                } else {
                    call_arg_types.push(return_type.unwrap().clone());
                }
            }

            tpl_pattern_match_args(
                db,
                cache,
                func_param_types,
                &call_arg_types,
                root,
                substitutor,
            )
        }
    }
}

fn tpl_pattern_match(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    root: &LuaSyntaxNode,
    pattern: &LuaType,
    target: &LuaType,
    substitutor: &mut TypeSubstitutor,
) -> Option<()> {
    match pattern {
        LuaType::TplRef(tpl) => {
            if tpl.get_tpl_id().is_func() {
                substitutor.insert_type(tpl.get_tpl_id(), target.clone());
            }
        }
        LuaType::StrTplRef(str_tpl) => match target {
            LuaType::StringConst(s) => {
                let prefix = str_tpl.get_prefix();
                let suffix = str_tpl.get_suffix();
                let type_name = SmolStr::new(format!("{}{}{}", prefix, s, suffix));
                substitutor.insert_type(str_tpl.get_tpl_id(), type_name.into());
            }
            _ => {}
        },
        LuaType::Array(base) => {
            array_tpl_pattern_match(db, cache, root, base, target, substitutor);
        }
        LuaType::TableGeneric(table_generic_params) => {
            table_generic_tpl_pattern_match(
                db,
                cache,
                root,
                table_generic_params,
                target,
                substitutor,
            );
        }
        LuaType::Generic(generic) => {
            generic_tpl_pattern_match(db, cache, root, generic, target, substitutor);
        }
        LuaType::Union(union) => {
            union_tpl_pattern_match(db, cache, root, union, target, substitutor);
        }
        LuaType::DocFunction(doc_func) => {
            func_tpl_pattern_match(db, cache, root, doc_func, target, substitutor);
        }
        LuaType::Tuple(tuple) => {
            tuple_tpl_pattern_match(db, cache, root, tuple, target, substitutor);
        }
        _ => {}
    }

    Some(())
}

fn array_tpl_pattern_match(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    root: &LuaSyntaxNode,
    base: &LuaType,
    target: &LuaType,
    substitutor: &mut TypeSubstitutor,
) -> Option<()> {
    match target {
        LuaType::Array(target_base) => {
            tpl_pattern_match(db, cache, root, base, target_base, substitutor);
        }
        LuaType::Tuple(target_tuple) => {
            let target_base = target_tuple.cast_down_array_base();
            tpl_pattern_match(db, cache, root, base, &target_base, substitutor);
        }
        LuaType::Object(target_object) => {
            let target_base = target_object.cast_down_array_base()?;
            tpl_pattern_match(db, cache, root, base, &target_base, substitutor);
        }
        _ => {}
    }

    Some(())
}

fn table_generic_tpl_pattern_match(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    root: &LuaSyntaxNode,
    table_generic_params: &Vec<LuaType>,
    target: &LuaType,
    substitutor: &mut TypeSubstitutor,
) -> Option<()> {
    if table_generic_params.len() != 2 {
        return None;
    }

    match target {
        LuaType::TableGeneric(target_table_generic_params) => {
            let min_len = table_generic_params
                .len()
                .min(target_table_generic_params.len());
            for i in 0..min_len {
                tpl_pattern_match(
                    db,
                    cache,
                    root,
                    &table_generic_params[i],
                    &target_table_generic_params[i],
                    substitutor,
                );
            }
        }
        LuaType::Array(target_array_base) => {
            tpl_pattern_match(
                db,
                cache,
                root,
                &table_generic_params[0],
                &LuaType::Integer,
                substitutor,
            );
            tpl_pattern_match(
                db,
                cache,
                root,
                &table_generic_params[1],
                target_array_base,
                substitutor,
            );
        }
        LuaType::Tuple(target_tuple) => {
            let len = target_tuple.get_types().len();
            let mut keys = Vec::new();
            for i in 0..len {
                keys.push(LuaType::IntegerConst((i as i64) + 1));
            }

            let key_type = LuaType::Union(LuaUnionType::new(keys).into());
            let target_base = target_tuple.cast_down_array_base();
            tpl_pattern_match(
                db,
                cache,
                root,
                &table_generic_params[0],
                &key_type,
                substitutor,
            );
            tpl_pattern_match(
                db,
                cache,
                root,
                &table_generic_params[1],
                &target_base,
                substitutor,
            );
        }
        LuaType::TableConst(inst) => {
            let owner = LuaMemberOwner::Element(inst.clone());
            table_generic_tpl_pattern_member_owner_match(
                db,
                cache,
                root,
                table_generic_params,
                owner,
                substitutor,
            );
        }
        LuaType::Ref(type_id) => {
            let owner = LuaMemberOwner::Type(type_id.clone());
            table_generic_tpl_pattern_member_owner_match(
                db,
                cache,
                root,
                table_generic_params,
                owner,
                substitutor,
            );
        }
        LuaType::Def(type_id) => {
            let owner = LuaMemberOwner::Type(type_id.clone());
            table_generic_tpl_pattern_member_owner_match(
                db,
                cache,
                root,
                table_generic_params,
                owner,
                substitutor,
            );
        }
        LuaType::Object(obj) => {
            let mut keys = vec![];
            let mut values = vec![];
            for (k, v) in obj.get_fields() {
                match k {
                    LuaMemberKey::Integer(i) => keys.push(LuaType::IntegerConst(i.clone())),
                    LuaMemberKey::Name(s) => keys.push(LuaType::StringConst(s.clone().into())),
                    _ => {}
                };
                values.push(v.clone());
            }

            let key_type = LuaType::Union(LuaUnionType::new(keys).into());
            let value_type = LuaType::Union(LuaUnionType::new(values).into());
            tpl_pattern_match(
                db,
                cache,
                root,
                &table_generic_params[0],
                &key_type,
                substitutor,
            );
            tpl_pattern_match(
                db,
                cache,
                root,
                &table_generic_params[1],
                &value_type,
                substitutor,
            );
        }

        LuaType::Global | LuaType::Any | LuaType::Table | LuaType::Userdata => {
            // too many
            tpl_pattern_match(
                db,
                cache,
                root,
                &table_generic_params[0],
                &LuaType::Any,
                substitutor,
            );
            tpl_pattern_match(
                db,
                cache,
                root,
                &table_generic_params[1],
                &LuaType::Any,
                substitutor,
            );
        }
        _ => {}
    }

    Some(())
}

fn table_generic_tpl_pattern_member_owner_match(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    root: &LuaSyntaxNode,
    table_generic_params: &Vec<LuaType>,
    owner: LuaMemberOwner,
    substitutor: &mut TypeSubstitutor,
) -> Option<()> {
    if table_generic_params.len() != 2 {
        return None;
    }

    let owner_type = match &owner {
        LuaMemberOwner::Element(inst) => LuaType::TableConst(inst.clone()),
        LuaMemberOwner::Type(type_id) => LuaType::Ref(type_id.clone()),
        _ => {
            return None;
        }
    };

    let members = infer_member_map(db, &owner_type)?;
    let mut keys = Vec::new();
    let mut values = Vec::new();
    for (k, v) in members {
        match k {
            LuaMemberKey::Integer(i) => keys.push(LuaType::IntegerConst(i.clone())),
            LuaMemberKey::Name(s) => keys.push(LuaType::StringConst(s.clone().into())),
            _ => {}
        };

        let resolve_type = match v.len() {
            0 => LuaType::Any,
            1 => v[0].typ.clone(),
            _ => {
                let mut types = Vec::new();
                for m in v {
                    types.push(m.typ.clone());
                }
                LuaType::Union(LuaUnionType::new(types).into())
            }
        };

        values.push(resolve_type);
    }

    let key_type = LuaType::Union(LuaUnionType::new(keys).into());
    let value_type = LuaType::Union(LuaUnionType::new(values).into());
    tpl_pattern_match(
        db,
        cache,
        root,
        &table_generic_params[0],
        &key_type,
        substitutor,
    );
    tpl_pattern_match(
        db,
        cache,
        root,
        &table_generic_params[1],
        &value_type,
        substitutor,
    );
    Some(())
}

fn generic_tpl_pattern_match(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    root: &LuaSyntaxNode,
    generic: &LuaGenericType,
    target: &LuaType,
    substitutor: &mut TypeSubstitutor,
) -> Option<()> {
    match target {
        LuaType::Generic(target_generic) => {
            let base = generic.get_base_type();
            let target_base = target_generic.get_base_type();
            if target_base != base {
                return None;
            }

            let params = generic.get_params();
            let target_params = target_generic.get_params();
            let min_len = params.len().min(target_params.len());
            for i in 0..min_len {
                tpl_pattern_match(db, cache, root, &params[i], &target_params[i], substitutor);
            }
        }
        _ => {}
    }

    Some(())
}

fn union_tpl_pattern_match(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    root: &LuaSyntaxNode,
    union: &LuaUnionType,
    target: &LuaType,
    substitutor: &mut TypeSubstitutor,
) -> Option<()> {
    for u in union.get_types() {
        tpl_pattern_match(db, cache, root, u, target, substitutor);
    }

    Some(())
}

fn func_tpl_pattern_match(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    root: &LuaSyntaxNode,
    tpl_func: &LuaFunctionType,
    target: &LuaType,
    substitutor: &mut TypeSubstitutor,
) -> Option<()> {
    match target {
        LuaType::DocFunction(target_doc_func) => {
            func_tpl_pattern_match_doc_func(
                db,
                cache,
                root,
                tpl_func,
                target_doc_func,
                substitutor,
            );
        }
        LuaType::Signature(signature_id) => {
            let signature = db.get_signature_index().get(&signature_id)?;
            let typed_params = signature.get_type_params();
            let rets = signature.get_return_types();
            let fake_doc_func = LuaFunctionType::new(
                signature.is_async,
                signature.is_colon_define,
                typed_params,
                rets,
            );
            func_tpl_pattern_match_doc_func(db, cache, root, tpl_func, &fake_doc_func, substitutor);
        }
        _ => {}
    }

    Some(())
}

fn func_tpl_pattern_match_doc_func(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    root: &LuaSyntaxNode,
    tpl_func: &LuaFunctionType,
    target_func: &LuaFunctionType,
    substitutor: &mut TypeSubstitutor,
) -> Option<()> {
    let tpl_func_params = tpl_func.get_params();
    let target_func_params = target_func.get_params();
    let param_len = tpl_func_params.len();
    for i in 0..param_len {
        let tpl_param_tuple = tpl_func_params.get(i)?;
        let target_param_tuple = match target_func_params.get(i) {
            Some(t) => t,
            None => break,
        };

        let tpl_param_type = tpl_param_tuple.1.clone().unwrap_or(LuaType::Any);

        // T ... match all other params
        if tpl_param_tuple.0 == "..." {
            let target_rest_params = &target_func_params[i..];
            if let LuaType::Variadic(inner) = tpl_param_type {
                func_varargs_tpl_pattern_match(&inner, target_rest_params, substitutor);
            }

            break;
        }

        let target_param_type = target_param_tuple.1.clone().unwrap_or(LuaType::Any);
        tpl_pattern_match(
            db,
            cache,
            root,
            &tpl_param_type,
            &target_param_type,
            substitutor,
        );
    }

    let tpl_rets = tpl_func.get_ret();
    let target_rets = target_func.get_ret();
    let ret_len = tpl_rets.len();
    for i in 0..ret_len {
        let tpl_ret_type = &tpl_rets[i];

        if let LuaType::Variadic(inner) = tpl_ret_type {
            let target_rest_rets = &target_rets[i..];
            variadic_tpl_pattern_match(&inner, target_rest_rets, substitutor);
            break;
        }

        let target_ret_type = match target_rets.get(i) {
            Some(t) => t,
            None => return None,
        };
        tpl_pattern_match(db, cache, root, tpl_ret_type, target_ret_type, substitutor);
    }

    Some(())
}

fn func_varargs_tpl_pattern_match(
    tpl: &LuaType,
    target_rest_params: &[(String, Option<LuaType>)],
    substitutor: &mut TypeSubstitutor,
) -> Option<()> {
    if let LuaType::TplRef(tpl_ref) = tpl {
        let tpl_id = tpl_ref.get_tpl_id();
        substitutor.insert_params(
            tpl_id,
            target_rest_params
                .iter()
                .map(|(n, t)| (n.clone(), t.clone()))
                .collect(),
        );
    }

    Some(())
}

pub fn variadic_tpl_pattern_match(
    tpl: &LuaType,
    target_rest_types: &[LuaType],
    substitutor: &mut TypeSubstitutor,
) -> Option<()> {
    if let LuaType::TplRef(tpl_ref) = tpl {
        let tpl_id = tpl_ref.get_tpl_id();
        substitutor.insert_multi_types(tpl_id, target_rest_types.to_vec());
    }

    Some(())
}

fn tuple_tpl_pattern_match(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    root: &LuaSyntaxNode,
    tpl_tuple: &LuaTupleType,
    target: &LuaType,
    substitutor: &mut TypeSubstitutor,
) -> Option<()> {
    match target {
        LuaType::Tuple(target_tuple) => {
            let tpl_tuple_types = tpl_tuple.get_types();
            let target_tuple_types = target_tuple.get_types();
            let tpl_tuple_len = tpl_tuple_types.len();
            for i in 0..tpl_tuple_len {
                let tpl_type = &tpl_tuple_types[i];

                if let LuaType::Variadic(inner) = tpl_type {
                    let target_rest_types = &target_tuple_types[i..];
                    variadic_tpl_pattern_match(inner, target_rest_types, substitutor);
                    break;
                }

                let target_type = match target_tuple_types.get(i) {
                    Some(t) => t,
                    None => break,
                };

                tpl_pattern_match(db, cache, root, tpl_type, target_type, substitutor);
            }
        }
        LuaType::Array(target_array_base) => {
            let tupl_tuple_types = tpl_tuple.get_types();
            let last_type = tupl_tuple_types.last()?;
            if let LuaType::Variadic(inner) = last_type {
                if let LuaType::TplRef(tpl_ref) = inner.deref() {
                    let tpl_id = tpl_ref.get_tpl_id();
                    substitutor.insert_multi_base(tpl_id, target_array_base.deref().clone());
                }
            }
        }
        _ => {}
    }

    Some(())
}
