use emmylua_parser::{
    BinaryOperator, LuaAssignStat, LuaAstNode, LuaAstToken, LuaExpr, LuaForRangeStat, LuaFuncStat,
    LuaIndexExpr, LuaLocalFuncStat, LuaLocalStat, LuaTableField, LuaVarExpr, PathTrait,
};

use crate::{
    compilation::analyzer::unresolve::{
        merge_decl_expr_type, merge_member_type, UnResolveDecl, UnResolveIterVar, UnResolveMember,
    },
    db_index::{LuaDeclId, LuaMemberId, LuaMemberOwner, LuaOperatorMetaMethod, LuaType},
    InferFailReason,
};

use super::{set_owner_and_add_member, LuaAnalyzer};

pub fn analyze_local_stat(analyzer: &mut LuaAnalyzer, local_stat: LuaLocalStat) -> Option<()> {
    let name_list: Vec<_> = local_stat.get_local_name_list().collect();
    let expr_list: Vec<_> = local_stat.get_value_exprs().collect();
    let name_count = name_list.len();
    let expr_count = expr_list.len();
    if expr_count == 0 {
        for local_name in name_list {
            let position = local_name.get_position();
            let decl_id = LuaDeclId::new(analyzer.file_id, position);
            let decl = analyzer.db.get_decl_index_mut().get_decl_mut(&decl_id)?;
            if decl.get_type().is_none() {
                decl.set_decl_type(LuaType::Nil);
            }
        }

        return Some(());
    }

    for i in 0..name_count {
        let name = name_list.get(i)?;
        let position = name.get_position();
        let expr = expr_list.get(i);
        if expr.is_none() {
            break;
        }
        let expr = expr.unwrap();
        match analyzer.infer_expr(expr) {
            Ok(mut expr_type) => {
                if let LuaType::MuliReturn(multi) = expr_type {
                    expr_type = multi.get_type(0)?.clone();
                }

                let decl_id = LuaDeclId::new(analyzer.file_id, position);
                merge_decl_expr_type(analyzer.db, &mut analyzer.infer_cache, decl_id, expr_type);
            }
            Err(InferFailReason::None) => {
                let decl_id = LuaDeclId::new(analyzer.file_id, position);
                let decl = analyzer.db.get_decl_index_mut().get_decl_mut(&decl_id)?;
                if decl.get_type().is_none() {
                    decl.set_decl_type(LuaType::Unknown);
                }
            }
            Err(reason) => {
                let decl_id = LuaDeclId::new(analyzer.file_id, position);
                let unresolve = UnResolveDecl {
                    file_id: analyzer.file_id,
                    decl_id,
                    expr: expr.clone(),
                    ret_idx: 0,
                    reason,
                };

                analyzer.add_unresolved(unresolve.into());
            }
        }
    }

    // The complexity brought by multiple return values is too high
    if name_count > expr_count {
        let last_expr = expr_list.last();
        if let Some(last_expr) = last_expr {
            match analyzer.infer_expr(last_expr) {
                Ok(last_expr_type) => {
                    if let LuaType::MuliReturn(multi) = last_expr_type {
                        for i in expr_count..name_count {
                            let name = name_list.get(i)?;
                            let position = name.get_position();
                            let decl_id = LuaDeclId::new(analyzer.file_id, position);
                            let decl = analyzer.db.get_decl_index_mut().get_decl_mut(&decl_id)?;
                            let ret_type = multi.get_type(i - expr_count + 1);
                            if let Some(ty) = ret_type {
                                merge_decl_expr_type(
                                    analyzer.db,
                                    &mut analyzer.infer_cache,
                                    decl_id,
                                    ty.clone(),
                                );
                            } else {
                                decl.set_decl_type(LuaType::Unknown);
                            }
                        }
                        return Some(());
                    }
                }
                Err(reason) => {
                    for i in expr_count..name_count {
                        let name = name_list.get(i)?;
                        let position = name.get_position();
                        let decl_id = LuaDeclId::new(analyzer.file_id, position);
                        let unresolve = UnResolveDecl {
                            file_id: analyzer.file_id,
                            decl_id,
                            expr: last_expr.clone(),
                            ret_idx: i - expr_count + 1,
                            reason: reason.clone(),
                        };

                        analyzer.add_unresolved(unresolve.into());
                    }
                }
            }
        }

        for i in expr_count..name_count {
            let name = name_list.get(i)?;
            let position = name.get_position();
            let decl_id = LuaDeclId::new(analyzer.file_id, position);
            let decl = analyzer.db.get_decl_index_mut().get_decl_mut(&decl_id)?;
            if decl.get_type().is_none() {
                if expr_count == 0 {
                    decl.set_decl_type(LuaType::Unknown);
                } else {
                    decl.set_decl_type(LuaType::Nil);
                }
            }
        }
    }

    Some(())
}

#[derive(Debug, Clone)]
enum TypeOwner {
    Decl(LuaDeclId),
    Member(LuaMemberId),
}

fn get_var_owner(analyzer: &mut LuaAnalyzer, var: LuaVarExpr) -> TypeOwner {
    let file_id = analyzer.file_id;
    match var {
        LuaVarExpr::NameExpr(var_name) => {
            let position = var_name.get_position();
            let decl_id = LuaDeclId::new(file_id, position);
            TypeOwner::Decl(decl_id)
        }
        LuaVarExpr::IndexExpr(index_expr) => {
            let maybe_decl_id = LuaDeclId::new(file_id, index_expr.get_position());
            if analyzer
                .db
                .get_decl_index()
                .get_decl(&maybe_decl_id)
                .is_some()
            {
                return TypeOwner::Decl(maybe_decl_id);
            }

            let member_id = LuaMemberId::new(index_expr.get_syntax_id(), file_id);
            TypeOwner::Member(member_id)
        }
    }
}

fn set_index_expr_owner(analyzer: &mut LuaAnalyzer, var_expr: LuaVarExpr) -> Option<()> {
    let file_id = analyzer.file_id;
    let index_expr = LuaIndexExpr::cast(var_expr.syntax().clone())?;
    let prefix_expr = index_expr.get_prefix_expr()?;

    match analyzer.infer_expr(&prefix_expr.clone().into()) {
        Ok(prefix_type) => {
            index_expr.get_index_key()?;
            let member_id = LuaMemberId::new(index_expr.get_syntax_id(), file_id);
            let member_owner = match prefix_type {
                LuaType::TableConst(in_file_range) => LuaMemberOwner::Element(in_file_range),
                LuaType::Def(def_id) => LuaMemberOwner::Type(def_id),
                LuaType::Instance(instance) => {
                    LuaMemberOwner::Element(instance.get_range().clone())
                }
                LuaType::Ref(ref_id) => {
                    let member_owner = LuaMemberOwner::Type(ref_id);
                    analyzer.db.get_member_index_mut().set_member_owner(
                        member_owner,
                        member_id.file_id,
                        member_id,
                    );
                    return Some(());
                }
                // is ref need extend field?
                _ => {
                    return None;
                }
            };

            set_owner_and_add_member(analyzer.db, member_owner, member_id);
        }
        Err(InferFailReason::None) => {}
        Err(reason) => {
            // record unresolve
            let unresolve_member = UnResolveMember {
                file_id: analyzer.file_id,
                member_id: LuaMemberId::new(var_expr.get_syntax_id(), file_id),
                expr: None,
                prefix: Some(prefix_expr.into()),
                ret_idx: 0,
                reason,
            };
            analyzer.add_unresolved(unresolve_member.into());
        }
    }

    Some(())
}

// assign stat is toooooooooo complex
pub fn analyze_assign_stat(analyzer: &mut LuaAnalyzer, assign_stat: LuaAssignStat) -> Option<()> {
    let (var_list, expr_list) = assign_stat.get_var_and_expr_list();
    let expr_count = expr_list.len();
    let var_count = var_list.len();
    for i in 0..expr_count {
        let var = var_list.get(i)?;
        let expr = expr_list.get(i);
        if expr.is_none() {
            break;
        }
        let expr = expr.unwrap();
        let type_owner = get_var_owner(analyzer, var.clone());
        set_index_expr_owner(analyzer, var.clone());

        match special_assign_pattern(analyzer, type_owner.clone(), var.clone(), expr.clone()) {
            Some(_) => {
                continue;
            }
            None => {}
        }

        let expr_type = match analyzer.infer_expr(expr) {
            Ok(expr_type) => match expr_type {
                LuaType::MuliReturn(multi) => multi.get_type(0)?.clone(),
                _ => expr_type,
            },
            Err(InferFailReason::None) => LuaType::Unknown,
            Err(reason) => {
                match type_owner {
                    TypeOwner::Decl(decl_id) => {
                        let decl = analyzer.db.get_decl_index().get_decl(&decl_id)?;
                        let decl_type = decl.get_type();
                        if decl_type.is_none() {
                            let unresolve_decl = UnResolveDecl {
                                file_id: analyzer.file_id,
                                decl_id,
                                expr: expr.clone(),
                                ret_idx: 0,
                                reason,
                            };

                            analyzer.add_unresolved(unresolve_decl.into());
                        }
                    }
                    TypeOwner::Member(member_id) => {
                        let unresolve_member = UnResolveMember {
                            file_id: analyzer.file_id,
                            member_id,
                            expr: Some(expr.clone()),
                            prefix: None,
                            ret_idx: 0,
                            reason,
                        };
                        analyzer.add_unresolved(unresolve_member.into());
                    }
                }
                continue;
            }
        };

        merge_type_owner_and_expr_type(analyzer, type_owner, &expr_type, 0);
    }

    // The complexity brought by multiple return values is too high
    if var_count > expr_count {
        let last_expr = expr_list.last();
        if let Some(last_expr) = last_expr.clone() {
            match analyzer.infer_expr(last_expr) {
                Ok(last_expr_type) => {
                    if last_expr_type.is_multi_return() {
                        for i in expr_count..var_count {
                            let var = var_list.get(i)?;
                            let type_owner = get_var_owner(analyzer, var.clone());
                            set_index_expr_owner(analyzer, var.clone());
                            merge_type_owner_and_expr_type(
                                analyzer,
                                type_owner,
                                &last_expr_type,
                                i - expr_count + 1,
                            );
                        }
                    }
                }
                Err(_) => {
                    for i in expr_count..var_count {
                        let var = var_list.get(i)?;
                        let type_owner = get_var_owner(analyzer, var.clone());
                        set_index_expr_owner(analyzer, var.clone());
                        merge_type_owner_and_expr(
                            analyzer,
                            type_owner,
                            last_expr.clone(),
                            i - expr_count + 1,
                        );
                    }
                }
            }
        }

        // Expressions like a, b are not valid
    }

    Some(())
}

fn merge_type_owner_and_expr_type(
    analyzer: &mut LuaAnalyzer,
    type_owner: TypeOwner,
    expr_type: &LuaType,
    idx: usize,
) -> Option<()> {
    let mut expr_type = expr_type.clone();
    if let LuaType::MuliReturn(multi) = expr_type {
        expr_type = multi.get_type(idx).unwrap_or(&LuaType::Nil).clone();
    }

    match type_owner {
        TypeOwner::Decl(decl_id) => {
            let decl = analyzer.db.get_decl_index_mut().get_decl_mut(&decl_id)?;
            let decl_type = decl.get_type();
            if decl_type.is_none() {
                decl.set_decl_type(expr_type);
            } else {
                merge_decl_expr_type(analyzer.db, &mut analyzer.infer_cache, decl_id, expr_type);
            }
        }
        TypeOwner::Member(member_id) => {
            let member = analyzer
                .db
                .get_member_index_mut()
                .get_member_mut(&member_id)?;
            if member.get_option_decl_type().is_none() {
                member.set_decl_type(expr_type);
            } else {
                merge_member_type(analyzer.db, &mut analyzer.infer_cache, member_id, expr_type);
            }
        }
    }

    Some(())
}

fn merge_type_owner_and_expr(
    analyzer: &mut LuaAnalyzer,
    type_owner: TypeOwner,
    expr: LuaExpr,
    idx: usize,
) -> Option<()> {
    match type_owner {
        TypeOwner::Decl(decl_id) => {
            let decl = analyzer.db.get_decl_index().get_decl(&decl_id)?;
            let decl_type = decl.get_type();
            if decl_type.is_none() {
                let unresolve_decl = UnResolveDecl {
                    file_id: analyzer.file_id,
                    decl_id,
                    expr: expr.clone(),
                    ret_idx: idx,
                    reason: InferFailReason::UnResolveExpr(expr),
                };

                analyzer.add_unresolved(unresolve_decl.into());
            }
        }
        TypeOwner::Member(member_id) => {
            let unresolve_member = UnResolveMember {
                file_id: analyzer.file_id,
                member_id,
                expr: Some(expr.clone()),
                prefix: None,
                ret_idx: idx,
                reason: InferFailReason::UnResolveExpr(expr),
            };
            analyzer.add_unresolved(unresolve_member.into());
        }
    }

    Some(())
}

pub fn analyze_for_range_stat(
    analyzer: &mut LuaAnalyzer,
    for_range_stat: LuaForRangeStat,
) -> Option<()> {
    let var_name_list = for_range_stat.get_var_name_list();
    let first_iter_expr = for_range_stat.get_expr_list().next()?;
    let first_iter_type = analyzer.infer_expr(&first_iter_expr);

    match first_iter_type {
        Ok(first_iter_type) => {
            let iter_doc_func = match first_iter_type {
                LuaType::DocFunction(doc_func) => Some(doc_func),
                LuaType::Ref(type_decl_id) => {
                    let type_decl = analyzer.db.get_type_index().get_type_decl(&type_decl_id)?;
                    if type_decl.is_alias() {
                        let alias_origin = type_decl.get_alias_origin(analyzer.db, None)?;
                        match alias_origin {
                            LuaType::DocFunction(doc_func) => Some(doc_func),
                            _ => None,
                        }
                    } else if type_decl.is_class() {
                        let operator_index = analyzer.db.get_operator_index();
                        let operator_ids = operator_index
                            .get_operators(&type_decl_id.into(), LuaOperatorMetaMethod::Call)?;
                        operator_ids
                            .iter()
                            .filter_map(|overload_id| {
                                let operator = operator_index.get_operator(overload_id)?;
                                let func = operator.get_operator_func()?;
                                match func {
                                    LuaType::DocFunction(f) => {
                                        return Some(f.clone());
                                    }
                                    _ => None,
                                }
                            })
                            .nth(0)
                    } else {
                        None
                    }
                }
                _ => None,
            };

            if let Some(doc_func) = iter_doc_func {
                let multi_return = doc_func.get_multi_return();
                let mut idx = 0;
                for var_name in var_name_list {
                    let position = var_name.get_position();
                    let decl_id = LuaDeclId::new(analyzer.file_id, position);
                    let decl = analyzer.db.get_decl_index_mut().get_decl_mut(&decl_id)?;
                    let decl_type = decl.get_type();
                    if decl_type.is_none() {
                        let ret_type = multi_return
                            .get_type(idx)
                            .unwrap_or(&LuaType::Unknown)
                            .clone();
                        decl.set_decl_type(ret_type);
                    }
                    idx += 1;
                }
                return Some(());
            } else {
                for var_name in var_name_list {
                    let position = var_name.get_position();
                    let decl_id = LuaDeclId::new(analyzer.file_id, position);
                    let decl = analyzer.db.get_decl_index_mut().get_decl_mut(&decl_id)?;
                    let decl_type = decl.get_type();
                    if decl_type.is_none() {
                        decl.set_decl_type(LuaType::Unknown);
                    }
                }
                return Some(());
            }
        }
        Err(InferFailReason::None) => {
            for var_name in var_name_list {
                let position = var_name.get_position();
                let decl_id = LuaDeclId::new(analyzer.file_id, position);
                let decl = analyzer.db.get_decl_index_mut().get_decl_mut(&decl_id)?;
                let decl_type = decl.get_type();
                if decl_type.is_none() {
                    decl.set_decl_type(LuaType::Unknown);
                }
            }
            return Some(());
        }
        Err(reason) => {
            let mut idx = 0;
            for var_name in var_name_list {
                let position = var_name.get_position();
                let decl_id = LuaDeclId::new(analyzer.file_id, position);
                let decl = analyzer.db.get_decl_index_mut().get_decl_mut(&decl_id)?;
                let decl_type = decl.get_type();
                if decl_type.is_none() {
                    let unresolved = UnResolveIterVar {
                        file_id: analyzer.file_id,
                        decl_id,
                        iter_expr: first_iter_expr.clone(),
                        ret_idx: idx,
                        reason: reason.clone(),
                    };
                    analyzer.add_unresolved(unresolved.into());
                }
                idx += 1;
            }
        }
    }

    Some(())
}

pub fn analyze_func_stat(analyzer: &mut LuaAnalyzer, func_stat: LuaFuncStat) -> Option<()> {
    let closure = func_stat.get_closure()?;
    let func_name = func_stat.get_func_name()?;
    let signature_type = analyzer.infer_expr(&closure.clone().into()).ok()?;
    let type_owner = get_var_owner(analyzer, func_name.clone());
    set_index_expr_owner(analyzer, func_name.clone());
    match type_owner {
        TypeOwner::Decl(decl_id) => {
            let decl = analyzer.db.get_decl_index_mut().get_decl_mut(&decl_id)?;
            decl.set_decl_type(signature_type);
        }
        TypeOwner::Member(member_id) => {
            let member = analyzer
                .db
                .get_member_index_mut()
                .get_member_mut(&member_id)?;

            member.set_decl_type(signature_type);
        }
    }

    Some(())
}

pub fn analyze_local_func_stat(
    analyzer: &mut LuaAnalyzer,
    local_func_stat: LuaLocalFuncStat,
) -> Option<()> {
    let closure = local_func_stat.get_closure()?;
    let func_name = local_func_stat.get_local_name()?;
    let signature_type = analyzer.infer_expr(&closure.clone().into()).ok()?;
    let position = func_name.get_position();
    let decl_id = LuaDeclId::new(analyzer.file_id, position);
    let decl = analyzer.db.get_decl_index_mut().get_decl_mut(&decl_id)?;
    decl.set_decl_type(signature_type);

    Some(())
}

pub fn analyze_table_field(analyzer: &mut LuaAnalyzer, field: LuaTableField) -> Option<()> {
    let _ = field.get_field_key()?;
    let value_expr = field.get_value_expr()?;
    let member_id = LuaMemberId::new(field.get_syntax_id(), analyzer.file_id);
    let value_type = match analyzer.infer_expr(&value_expr.clone().into()) {
        Ok(value_type) => value_type,
        Err(InferFailReason::None) => LuaType::Unknown,
        Err(reason) => {
            let unresolve = UnResolveMember {
                file_id: analyzer.file_id,
                member_id,
                expr: Some(value_expr.clone()),
                prefix: None,
                ret_idx: 0,
                reason,
            };

            analyzer.add_unresolved(unresolve.into());
            return None;
        }
    };

    let member = analyzer
        .db
        .get_member_index_mut()
        .get_member_mut(&member_id)?;

    match member.get_option_decl_type() {
        Some(_) => {
            merge_member_type(
                analyzer.db,
                &mut analyzer.infer_cache,
                member_id,
                value_type,
            );
        }
        None => {
            member.set_decl_type(value_type);
        }
    }

    Some(())
}

fn special_assign_pattern(
    analyzer: &mut LuaAnalyzer,
    type_owner: TypeOwner,
    var: LuaVarExpr,
    expr: LuaExpr,
) -> Option<()> {
    let access_path = var.get_access_path()?;
    let binary_expr = if let LuaExpr::BinaryExpr(binary_expr) = expr {
        binary_expr
    } else {
        return None;
    };

    if binary_expr.get_op_token()?.get_op() != BinaryOperator::OpOr {
        return None;
    }

    let (left, right) = binary_expr.get_exprs()?;
    let left_var = LuaVarExpr::cast(left.syntax().clone())?;
    let left_access_path = left_var.get_access_path()?;
    if access_path != left_access_path {
        return None;
    }

    match analyzer.infer_expr(&right) {
        Ok(right_expr_type) => {
            merge_type_owner_and_expr_type(analyzer, type_owner, &right_expr_type, 0);
        }
        Err(_) => return None,
    }

    Some(())
}
