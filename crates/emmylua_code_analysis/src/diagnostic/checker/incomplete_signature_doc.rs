use std::collections::HashSet;

use emmylua_parser::{LuaAstNode, LuaClosureExpr, LuaDocTagParam, LuaDocTagReturn, LuaStat};

use crate::{DiagnosticCode, LuaSemanticDeclId, LuaType, SemanticDeclLevel, SemanticModel};

use super::{get_closure_expr_comment, get_own_return_stats, Checker, DiagnosticContext};

pub struct IncompleteSignatureDocChecker;

impl Checker for IncompleteSignatureDocChecker {
    const CODES: &[DiagnosticCode] = &[
        DiagnosticCode::IncompleteSignatureDoc,
        DiagnosticCode::MissingGlobalDoc,
    ];

    fn check(context: &mut DiagnosticContext, semantic_model: &SemanticModel) {
        let root = semantic_model.get_root();
        for closure_expr in root.descendants::<LuaClosureExpr>() {
            check_doc(context, semantic_model, &closure_expr);
        }
    }
}

fn check_doc(
    context: &mut DiagnosticContext,
    semantic_model: &SemanticModel,
    closure_expr: &LuaClosureExpr,
) -> Option<()> {
    let semantic_decl = semantic_model.find_decl(
        rowan::NodeOrToken::Node(closure_expr.syntax().clone()),
        SemanticDeclLevel::default(),
    )?;
    let (is_global, function_name) = match semantic_decl {
        LuaSemanticDeclId::LuaDecl(decl_id) => {
            let decl = semantic_model
                .get_db()
                .get_decl_index()
                .get_decl(&decl_id)?;
            (decl.is_global(), decl.get_name().to_string())
        }
        _ => (false, String::new()),
    };

    let comment = get_closure_expr_comment(closure_expr);

    if comment.is_none() && is_global {
        if let Some(stat) = closure_expr.get_parent::<LuaStat>() {
            context.add_diagnostic(
                DiagnosticCode::MissingGlobalDoc,
                stat.get_range(),
                t!(
                    "Missing comment for global function `%{name}`.",
                    name = function_name
                )
                .to_string(),
                None,
            );
        }
        return Some(());
    }

    let Some(comment) = comment else {
        return Some(());
    };

    let code = if is_global {
        DiagnosticCode::MissingGlobalDoc
    } else {
        DiagnosticCode::IncompleteSignatureDoc
    };

    let doc_param_names: HashSet<String> = comment
        .children::<LuaDocTagParam>()
        .filter_map(|param| {
            param
                .get_name_token()
                .map(|token| token.get_name_text().to_string())
        })
        .collect();

    let doc_return_len: usize = comment
        .children::<LuaDocTagReturn>()
        .map(|return_doc| return_doc.get_types().count())
        .sum();

    // 如果文档中没有参数和返回值注解, 且不是全局函数, 则不检查
    if doc_param_names.is_empty() && doc_return_len == 0 && !is_global {
        return Some(());
    }

    check_params(
        context,
        closure_expr,
        &doc_param_names,
        code,
        is_global,
        &function_name,
    );

    check_returns(
        context,
        semantic_model,
        closure_expr,
        doc_return_len,
        code,
        is_global,
        &function_name,
    );

    Some(())
}

fn check_params(
    context: &mut DiagnosticContext,
    closure_expr: &LuaClosureExpr,
    doc_param_names: &HashSet<String>,
    code: DiagnosticCode,
    is_global: bool,
    function_name: &str,
) {
    let Some(params_list) = closure_expr.get_params_list() else {
        return;
    };

    for param in params_list.get_params() {
        let Some(name_token) = param.get_name_token() else {
            continue;
        };

        let name = name_token.get_name_text();
        if !doc_param_names.contains(name) && name != "_" {
            let message = if is_global {
                t!(
                    "Missing @param annotation for parameter `%{name}` in global function `%{function_name}`.",
                    name = name,
                    function_name = function_name
                )
            } else {
                t!(
                    "Incomplete signature. Missing @param annotation for parameter `%{name}`.",
                    name = name
                )
            };

            context.add_diagnostic(code, param.get_range(), message.to_string(), None);
        }
    }
}

fn check_returns(
    context: &mut DiagnosticContext,
    semantic_model: &SemanticModel,
    closure_expr: &LuaClosureExpr,
    doc_return_len: usize,
    code: DiagnosticCode,
    is_global: bool,
    function_name: &str,
) {
    for return_stat in get_own_return_stats(closure_expr) {
        let mut return_stat_len: usize = 0;

        for (i, expr) in return_stat.get_expr_list().enumerate() {
            let Some(infer_type) = semantic_model.infer_expr(expr.clone()).ok() else {
                continue;
            };

            let expr_return_count = match infer_type {
                LuaType::MuliReturn(types) => types.get_len().unwrap_or(1) as usize,
                _ => 1,
            };

            return_stat_len += expr_return_count;

            if return_stat_len > doc_return_len {
                let message = if is_global {
                    t!(
                        "Missing @return annotation at index `%{index}` in global function `%{function_name}`.",
                        index = i + 1,
                        function_name = function_name
                    )
                } else {
                    t!(
                        "Incomplete signature. Missing @return annotation at index `%{index}`.",
                        index = i + 1
                    )
                };

                context.add_diagnostic(code, expr.get_range(), message.to_string(), None);
            }
        }
    }
}
