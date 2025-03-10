use std::collections::{HashMap, HashSet};

use crate::{
    DiagnosticCode, LuaDeclId, LuaDeclarationTree, LuaScope, LuaScopeKind, ScopeOrDeclId,
    SemanticModel,
};

use super::DiagnosticContext;

pub const CODES: &[DiagnosticCode] = &[DiagnosticCode::RedefinedLocal];

pub fn check(context: &mut DiagnosticContext, semantic_model: &SemanticModel) -> Option<()> {
    let file_id = semantic_model.get_file_id();
    let decl_tree = semantic_model
        .get_db()
        .get_decl_index()
        .get_decl_tree(&file_id)?;

    let root_scope = decl_tree.get_root_scope()?;
    let mut diagnostics = HashSet::new();
    let mut root_locals = HashMap::new();

    check_scope_for_redefined_locals(
        context,
        &decl_tree,
        root_scope,
        &mut root_locals,
        &mut diagnostics,
    );

    // 添加诊断信息
    for decl_id in diagnostics {
        if let Some(decl) = decl_tree.get_decl(&decl_id) {
            context.add_diagnostic(
                DiagnosticCode::RedefinedLocal,
                decl.get_range(),
                t!("Redefined local variable `%{name}`", name = decl.get_name()).to_string(),
                None,
            );
        }
    }
    Some(())
}

fn check_scope_for_redefined_locals(
    context: &mut DiagnosticContext,
    decl_tree: &LuaDeclarationTree,
    scope: &LuaScope,
    parent_locals: &mut HashMap<String, LuaDeclId>,
    diagnostics: &mut HashSet<LuaDeclId>,
) {
    let is_normal = scope.get_kind() == LuaScopeKind::Normal;

    let mut current_locals = parent_locals.clone();

    // 检查当前作用域中的声明
    for child in scope.get_children() {
        if let ScopeOrDeclId::Decl(decl_id) = child {
            if let Some(decl) = decl_tree.get_decl(decl_id) {
                let name = decl.get_name().to_string();
                if decl.is_local() && !name.starts_with("_") {
                    if current_locals.contains_key(&name) {
                        // 发现重定义，记录诊断
                        diagnostics.insert(*decl_id);
                    }
                    // 将当前声明加入映射
                    current_locals.insert(name.clone(), *decl_id);
                }
            }
        }
    }

    // 检查子作用域
    for child in scope.get_children() {
        if let ScopeOrDeclId::Scope(scope_id) = child {
            if let Some(child_scope) = decl_tree.get_scope(scope_id) {
                check_scope_for_redefined_locals(
                    context,
                    decl_tree,
                    child_scope,
                    &mut current_locals,
                    diagnostics,
                );
            }
        }
    }

    // 更新到父作用域
    if !is_normal {
        for (name, decl_id) in current_locals {
            parent_locals.insert(name, decl_id);
        }
    }
}
