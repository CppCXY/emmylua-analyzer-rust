use emmylua_parser::{
    LuaAssignStat, LuaAstNode, LuaAstToken, LuaExpr, LuaForRangeStat, LuaFuncStat,
    LuaLocalFuncStat, LuaLocalStat, LuaTableField, LuaVarExpr,
};
use serde::de;

use crate::{
    compilation::analyzer::unresolve::{
        merge_decl_expr_type, merge_member_type, UnResolveDecl, UnResolveIterVar, UnResolveMember,
    },
    db_index::{LuaDeclId, LuaMemberId, LuaMemberOwner, LuaType},
};

use super::LuaAnalyzer;

pub fn analyze_local_stat(analyzer: &mut LuaAnalyzer, local_stat: LuaLocalStat) -> Option<()> {
    let name_list: Vec<_> = local_stat.get_local_name_list().collect();
    let expr_list: Vec<_> = local_stat.get_value_exprs().collect();
    let name_count = name_list.len();
    let expr_count = expr_list.len();
    for i in 0..name_count {
        let name = name_list.get(i)?;
        let position = name.get_position();
        let expr = expr_list.get(i);
        if expr.is_none() {
            break;
        }
        let expr = expr.unwrap();
        let expr_type = analyzer.infer_expr(expr);
        match expr_type {
            Some(expr_type) => {
                let decl_id = LuaDeclId::new(analyzer.file_id, position);
                merge_decl_expr_type(analyzer.db, decl_id, expr_type);
            }
            None => {
                let decl_id = LuaDeclId::new(analyzer.file_id, position);
                let unresolve = UnResolveDecl {
                    decl_id,
                    expr: expr.clone(),
                    ret_idx: 0,
                };

                analyzer.add_unresolved(unresolve.into());
            }
        }
    }

    // The complexity brought by multiple return values is too high
    if name_count > expr_count {
        let last_expr = expr_list.last();
        if let Some(last_expr) = last_expr {
            let last_expr_type = analyzer.infer_expr(last_expr);
            if let Some(last_expr_type) = last_expr_type {
                if let LuaType::MuliReturn(multi) = last_expr_type {
                    for i in expr_count..name_count {
                        let name = name_list.get(i)?;
                        let position = name.get_position();
                        let decl_id = LuaDeclId::new(analyzer.file_id, position);
                        let decl = analyzer.db.get_decl_index_mut().get_decl_mut(&decl_id)?;
                        let ret_type = multi.get_type(i - expr_count + 1);
                        if let Some(ty) = ret_type {
                            merge_decl_expr_type(analyzer.db, decl_id, ty.clone());
                        } else {
                            decl.set_decl_type(LuaType::Unknown);
                        }
                    }
                    return Some(());
                }
            } else {
                for i in expr_count..name_count {
                    let name = name_list.get(i)?;
                    let position = name.get_position();
                    let decl_id = LuaDeclId::new(analyzer.file_id, position);
                    let unresolve = UnResolveDecl {
                        decl_id,
                        expr: last_expr.clone(),
                        ret_idx: i - expr_count + 1,
                    };

                    analyzer.add_unresolved(unresolve.into());
                }
                return Some(());
            }
        }

        for i in expr_count..name_count {
            let name = name_list.get(i)?;
            let position = name.get_position();
            let decl_id = LuaDeclId::new(analyzer.file_id, position);
            let decl = analyzer.db.get_decl_index_mut().get_decl_mut(&decl_id)?;
            decl.set_decl_type(LuaType::Unknown);
        }
    }

    Some(())
}

#[derive(Debug)]
enum TypeOwner {
    Decl(LuaDeclId),
    Member(LuaMemberId),
}

#[allow(unused)]
impl TypeOwner {
    pub fn is_decl(&self) -> bool {
        matches!(self, TypeOwner::Decl(_))
    }

    pub fn is_member(&self) -> bool {
        matches!(self, TypeOwner::Member(_))
    }
}

fn get_var_type_owner(
    analyzer: &mut LuaAnalyzer,
    var: LuaVarExpr,
    expr: LuaExpr,
) -> Option<TypeOwner> {
    let file_id = analyzer.file_id;
    match var {
        LuaVarExpr::NameExpr(var_name) => {
            let position = var_name.get_position();
            let decl_id = LuaDeclId::new(file_id, position);
            let mut decl = analyzer.db.get_decl_index().get_decl(&decl_id);
            if decl.is_none() {
                let decl_tree = analyzer.db.get_decl_index().get_decl_tree(&file_id)?;
                let name = var_name.get_name_text()?;
                decl = decl_tree.find_local_decl(&name, position);
            }

            if decl.is_some() {
                return Some(TypeOwner::Decl(decl.unwrap().get_id()));
            }
        }
        LuaVarExpr::IndexExpr(var_index) => {
            let prefix_expr = var_index.get_prefix_expr()?;
            let prefix_type = analyzer.infer_expr(&prefix_expr.clone().into());
            match prefix_type {
                Some(prefix_type) => {
                    var_index.get_index_key()?;

                    let member_owner = match prefix_type {
                        LuaType::TableConst(in_file_range) => LuaMemberOwner::Table(in_file_range),
                        LuaType::Def(def_id) => LuaMemberOwner::Type(def_id),
                        // is ref need extend field?
                        _ => {
                            return None;
                        }
                    };
                    let member_id = LuaMemberId::new(var_index.get_syntax_id(), file_id);
                    analyzer
                        .db
                        .get_member_index_mut()
                        .add_member_owner(member_owner, member_id);
                    return Some(TypeOwner::Member(member_id));
                }
                None => {
                    // record unresolve
                    let unresolve_member = UnResolveMember {
                        member_id: LuaMemberId::new(var_index.get_syntax_id(), file_id),
                        expr: expr.clone(),
                        prefix: Some(prefix_expr.into()),
                        ret_idx: 0,
                    };
                    analyzer.add_unresolved(unresolve_member.into());
                }
            }
        }
    }

    None
}

// assign stat is too complex
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
        let type_owner = match get_var_type_owner(analyzer, var.clone(), expr.clone()) {
            Some(type_owner) => type_owner,
            None => {
                continue;
            }
        };

        let expr_type = match analyzer.infer_expr(expr) {
            Some(expr_type) => match expr_type {
                LuaType::MuliReturn(multi) => multi.get_type(0)?.clone(),
                _ => expr_type,
            },
            None => {
                match type_owner {
                    TypeOwner::Decl(decl_id) => {
                        let decl = analyzer.db.get_decl_index().get_decl(&decl_id)?;
                        let decl_type = decl.get_type();
                        if decl_type.is_none() {
                            let unresolve_decl = UnResolveDecl {
                                decl_id,
                                expr: expr.clone(),
                                ret_idx: 0,
                            };

                            analyzer.add_unresolved(unresolve_decl.into());
                        }
                    }
                    TypeOwner::Member(member_id) => {
                        let unresolve_member = UnResolveMember {
                            member_id,
                            expr: expr.clone(),
                            prefix: None,
                            ret_idx: 0,
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
            let last_expr_type = analyzer.infer_expr(last_expr);
            if let Some(last_expr_type) = last_expr_type {
                for i in expr_count..var_count {
                    let var = var_list.get(i)?;
                    let type_owner =
                        match get_var_type_owner(analyzer, var.clone(), last_expr.clone()) {
                            Some(type_owner) => type_owner,
                            None => {
                                continue;
                            }
                        };
                    merge_type_owner_and_expr_type(
                        analyzer,
                        type_owner,
                        &last_expr_type,
                        i - expr_count + 1,
                    );
                }
                return Some(());
            } else {
                for i in expr_count..var_count {
                    let var = var_list.get(i)?;
                    let type_owner =
                        match get_var_type_owner(analyzer, var.clone(), last_expr.clone()) {
                            Some(type_owner) => type_owner,
                            None => {
                                continue;
                            }
                        };
                    merge_type_owner_and_expr(
                        analyzer,
                        type_owner,
                        last_expr.clone(),
                        i - expr_count + 1,
                    );
                }
                return Some(());
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
        expr_type = multi.get_type(idx).unwrap_or(&LuaType::Unknown).clone();
    }

    match type_owner {
        TypeOwner::Decl(decl_id) => {
            let decl = analyzer.db.get_decl_index_mut().get_decl_mut(&decl_id)?;
            let decl_type = decl.get_type();
            if decl_type.is_none() {
                decl.set_decl_type(expr_type);
            } else {
                merge_decl_expr_type(analyzer.db, decl_id, expr_type);
            }
        }
        TypeOwner::Member(member_id) => {
            let member = analyzer
                .db
                .get_member_index_mut()
                .get_member_mut(&member_id)?;
            if member.decl_type.is_unknown() {
                member.decl_type = expr_type;
            } else {
                merge_member_type(analyzer.db, member_id, expr_type);
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
                    decl_id,
                    expr,
                    ret_idx: idx,
                };

                analyzer.add_unresolved(unresolve_decl.into());
            }
        }
        TypeOwner::Member(member_id) => {
            let unresolve_member = UnResolveMember {
                member_id,
                expr,
                prefix: None,
                ret_idx: idx,
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

    if let Some(first_iter_type) = first_iter_type {
        if let LuaType::DocFunction(doc_func) = first_iter_type {
            let rets = doc_func.get_ret();
            let mut idx = 0;
            for var_name in var_name_list {
                let position = var_name.get_position();
                let decl_id = LuaDeclId::new(analyzer.file_id, position);
                let decl = analyzer.db.get_decl_index_mut().get_decl_mut(&decl_id)?;
                let decl_type = decl.get_type();
                if decl_type.is_none() {
                    let ret_type = rets.get(idx).unwrap_or(&LuaType::Unknown).clone();
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

    let mut idx = 0;
    for var_name in var_name_list {
        let position = var_name.get_position();
        let decl_id = LuaDeclId::new(analyzer.file_id, position);
        let decl = analyzer.db.get_decl_index_mut().get_decl_mut(&decl_id)?;
        let decl_type = decl.get_type();
        if decl_type.is_none() {
            let unresolved = UnResolveIterVar {
                decl_id,
                iter_expr: first_iter_expr.clone(),
                ret_idx: idx,
            };
            analyzer.add_unresolved(unresolved.into());
        }
        idx += 1;
    }

    Some(())
}

pub fn analyze_func_stat(analyzer: &mut LuaAnalyzer, func_stat: LuaFuncStat) -> Option<()> {
    let closure = func_stat.get_closure()?;
    let func_name = func_stat.get_func_name()?;
    let signature_type = analyzer.infer_expr(&closure.clone().into())?;
    let type_owner = get_var_type_owner(analyzer, func_name, closure.into())?;
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

            member.decl_type = signature_type;
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
    let signature_type = analyzer.infer_expr(&closure.clone().into())?;
    let position = func_name.get_position();
    let decl_id = LuaDeclId::new(analyzer.file_id, position);
    let decl = analyzer.db.get_decl_index_mut().get_decl_mut(&decl_id)?;
    decl.set_decl_type(signature_type);

    Some(())
}

pub fn analyze_table_field(analyzer: &mut LuaAnalyzer, field: LuaTableField) -> Option<()> {
    if field.is_value_field() {
        return None;
    }

    let _ = field.get_field_key()?;
    let value_expr = field.get_value_expr()?;
    let member_id = LuaMemberId::new(field.get_syntax_id(), analyzer.file_id);
    let value_type = match analyzer.infer_expr(&value_expr.clone().into()) {
        Some(value_type) => value_type,
        None => {
            let unresolve = UnResolveMember {
                member_id,
                expr: value_expr.clone(),
                prefix: None,
                ret_idx: 0,
            };

            analyzer.add_unresolved(unresolve.into());
            return None;
        }
    };

    let member = analyzer
        .db
        .get_member_index_mut()
        .get_member_mut(&member_id)?;

    let decl_type = member.get_decl_type();
    if decl_type.is_unknown() {
        member.decl_type = value_type;
    }

    Some(())
}