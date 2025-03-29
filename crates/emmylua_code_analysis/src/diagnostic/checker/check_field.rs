use std::collections::HashSet;

use emmylua_parser::{LuaAst, LuaAstNode, LuaExpr, LuaIndexExpr, LuaIndexKey, LuaVarExpr};

use crate::{DiagnosticCode, InferFailReason, LuaMemberKey, LuaType, SemanticModel};

use super::{humanize_lint_type, Checker, DiagnosticContext};

pub struct CheckFieldChecker;

impl Checker for CheckFieldChecker {
    const CODES: &[DiagnosticCode] = &[DiagnosticCode::InjectField, DiagnosticCode::UndefinedField];

    fn check(context: &mut DiagnosticContext, semantic_model: &SemanticModel) {
        let root = semantic_model.get_root().clone();
        let mut checked_index_expr = HashSet::new();
        for node in root.descendants::<LuaAst>() {
            match node {
                LuaAst::LuaAssignStat(assign) => {
                    let (vars, _) = assign.get_var_and_expr_list();
                    for var in vars.iter() {
                        if let LuaVarExpr::IndexExpr(index_expr) = var {
                            checked_index_expr.insert(index_expr.syntax().clone());
                            check_index_expr(
                                context,
                                semantic_model,
                                index_expr,
                                DiagnosticCode::InjectField,
                            );
                        }
                    }
                }
                LuaAst::LuaIndexExpr(index_expr) => {
                    if checked_index_expr.contains(index_expr.syntax()) {
                        continue;
                    }
                    check_index_expr(
                        context,
                        semantic_model,
                        &index_expr,
                        DiagnosticCode::UndefinedField,
                    );
                }
                _ => {}
            }
        }
    }
}

fn check_index_expr(
    context: &mut DiagnosticContext,
    semantic_model: &SemanticModel,
    index_expr: &LuaIndexExpr,
    code: DiagnosticCode,
) -> Option<()> {
    let db = context.db;
    let prefix_typ = semantic_model
        .infer_expr(index_expr.get_prefix_expr()?)
        .unwrap_or(LuaType::Unknown);

    if !is_valid_prefix_type(&prefix_typ) {
        return Some(());
    }

    let index_key = index_expr.get_index_key()?;

    if is_valid_member(semantic_model, &prefix_typ, index_expr, &index_key).is_some() {
        return Some(());
    }

    let index_name = index_key.get_path_part();
    match code {
        DiagnosticCode::InjectField => {
            context.add_diagnostic(
                DiagnosticCode::InjectField,
                index_key.get_range()?,
                t!(
                    "Fields cannot be injected into the reference of `%{class}` for `%{field}`. ",
                    class = humanize_lint_type(&db, &prefix_typ),
                    field = index_name,
                )
                .to_string(),
                None,
            );
        }
        DiagnosticCode::UndefinedField => {
            context.add_diagnostic(
                DiagnosticCode::UndefinedField,
                index_key.get_range()?,
                t!("Undefined field `%{field}`. ", field = index_name,).to_string(),
                None,
            );
        }
        _ => {}
    }

    Some(())
}

fn is_valid_prefix_type(typ: &LuaType) -> bool {
    let mut current_typ = typ;
    loop {
        match current_typ {
            LuaType::Any
            | LuaType::Unknown
            | LuaType::Table
            | LuaType::TplRef(_)
            | LuaType::StrTplRef(_)
            | LuaType::TableConst(_) => return false,
            LuaType::Instance(instance_typ) => {
                current_typ = instance_typ.get_base();
            }
            _ => return true,
        }
    }
}

#[allow(dead_code)]
fn is_valid_index_key(index_key: &LuaIndexKey) -> bool {
    match index_key {
        LuaIndexKey::String(_) | LuaIndexKey::Name(_) | LuaIndexKey::Integer(_) => true,
        _ => false,
    }
}

fn is_valid_member(
    semantic_model: &SemanticModel,
    prefix_typ: &LuaType,
    index_expr: &LuaIndexExpr,
    index_key: &LuaIndexKey,
) -> Option<()> {
    // 检查 member_info
    let need_add_diagnostic =
        match semantic_model.get_semantic_info(index_expr.syntax().clone().into()) {
            Some(info) => info.semantic_decl.is_none() && info.typ.is_unknown(),
            None => true,
        };

    if !need_add_diagnostic {
        return Some(());
    }

    // 获取并验证 key_type
    let key_type = match index_key {
        LuaIndexKey::Expr(expr) => match semantic_model.infer_expr(expr.clone()) {
            Ok(
                LuaType::Any
                | LuaType::Unknown
                | LuaType::Table
                | LuaType::TplRef(_)
                | LuaType::StrTplRef(_),
            ) => {
                return Some(());
            }
            Ok(typ) => typ,
            // 解析失败时认为其是合法的, 因为他可能没有标注类型
            Err(InferFailReason::UnResolveDeclType(_)) => {
                return Some(());
            }
            Err(_) => {
                return None;
            }
        },
        _ => return None,
    };

    // 允许特定类型组合通过
    match (prefix_typ, &key_type) {
        (LuaType::Tuple(_), LuaType::Integer | LuaType::IntegerConst(_)) => return Some(()),
        _ => {}
    }
    // 解决`key`类型为联合名称时的报错(通常是`pairs`返回的`key`)
    let (key_name_set, key_type_set) = get_key_types(&key_type);
    if key_name_set.is_empty() && key_type_set.is_empty() {
        return None;
    }
    let members = semantic_model.infer_member_infos(prefix_typ)?;
    for info in members {
        match &info.key {
            LuaMemberKey::SyntaxId(syntax_id) => {
                let node = syntax_id.to_node_from_root(semantic_model.get_root().syntax())?;
                let expr = LuaExpr::cast(node)?;
                if let Ok(typ) = semantic_model.infer_expr(expr) {
                    if key_type_set.contains(&typ) {
                        return Some(());
                    }
                }
            }
            _ => {
                let key_name = info.key.to_path();
                if key_name_set.contains(&key_name) {
                    return Some(());
                }
            }
        }
    }

    None
}

fn get_key_types(typ: &LuaType) -> (HashSet<String>, HashSet<LuaType>) {
    let mut type_set = HashSet::new();
    let mut name_set = HashSet::new();
    let mut stack = vec![typ.clone()];

    while let Some(current_type) = stack.pop() {
        match &current_type {
            LuaType::StringConst(name) => {
                name_set.insert(name.as_ref().to_string());
            }
            LuaType::IntegerConst(i) => {
                name_set.insert(format!("[{}]", i));
            }
            LuaType::Union(union_typ) => {
                for t in union_typ.get_types() {
                    stack.push(t.clone());
                }
            }
            _ => {
                if current_type.is_table() {
                    type_set.insert(current_type);
                }
            }
        }
    }
    (name_set, type_set)
}
