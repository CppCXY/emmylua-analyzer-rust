use emmylua_code_analysis::{
    LuaFunctionType, LuaOperatorMetaMethod, LuaSemanticDeclId, LuaSignatureId, LuaType,
    LuaTypeDeclId, RenderLevel, SemanticModel,
};
use emmylua_parser::{LuaAstNode, LuaCallExpr, LuaSyntaxToken, LuaTokenKind};
use lsp_types::{
    Documentation, MarkupContent, ParameterInformation, ParameterLabel, SignatureHelp,
    SignatureInformation,
};
use rowan::NodeOrToken;

use emmylua_code_analysis::humanize_type;

pub fn build_signature_helper(
    semantic_model: &SemanticModel,
    call_expr: LuaCallExpr,
    token: LuaSyntaxToken,
) -> Option<SignatureHelp> {
    let prefix_expr = call_expr.get_prefix_expr()?;
    let prefix_expr_type = semantic_model.infer_expr(prefix_expr.clone()).ok()?;
    let colon_call = call_expr.is_colon_call();
    let current_idx = get_current_param_index(&call_expr, &token)?;
    match prefix_expr_type {
        LuaType::DocFunction(func_type) => {
            build_doc_function_signature_help(semantic_model, &func_type, colon_call, current_idx)
        }
        LuaType::Signature(signature_id) => {
            build_sig_id_signature_help(semantic_model, signature_id, colon_call, current_idx)
        }
        LuaType::Ref(type_decl_id) => {
            build_type_signature_help(semantic_model, &type_decl_id, colon_call, current_idx)
        }
        LuaType::Def(type_decl_id) => {
            build_type_signature_help(semantic_model, &type_decl_id, colon_call, current_idx)
        }
        LuaType::Union(union_types) => build_union_type_signature_help(
            semantic_model,
            union_types.get_types(),
            colon_call,
            current_idx,
        ),
        _ => None,
    }
}

pub fn get_current_param_index(call_expr: &LuaCallExpr, token: &LuaSyntaxToken) -> Option<usize> {
    let arg_list = call_expr.get_args_list()?;
    let mut current_idx = 0;
    let token_position = token.text_range().start();
    for node_or_token in arg_list.syntax().children_with_tokens() {
        if let NodeOrToken::Token(token) = node_or_token {
            if token.kind() == LuaTokenKind::TkComma.into() {
                if token.text_range().start() <= token_position {
                    current_idx += 1;
                }
            }
        }
    }

    Some(current_idx)
}

fn build_doc_function_signature_help(
    semantic_model: &SemanticModel,
    func_type: &LuaFunctionType,
    colon_call: bool,
    current_idx: usize,
) -> Option<SignatureHelp> {
    let mut current_idx = current_idx;
    let mut params = func_type
        .get_params()
        .iter()
        .map(|param| param.clone())
        .collect::<Vec<_>>();
    let colon_define = func_type.is_colon_define();
    match (colon_define, colon_call) {
        (true, false) => {
            params.insert(0, ("self".to_string(), None));
        }
        (false, true) => {
            if !params.is_empty() {
                params.remove(0);
            }
        }
        _ => {}
    }

    if let Some((name, _)) = params.last() {
        if name == "..." && current_idx >= params.len() {
            current_idx = params.len() - 1;
        }
    }

    let params_str = params
        .iter()
        .map(|param| param.0.clone())
        .collect::<Vec<_>>();
    let db = semantic_model.get_db();
    let param_infos = params
        .iter()
        .map(|param| {
            let label = param.0.clone();
            let typ = param.1.clone();
            let documentation = if let Some(typ) = typ {
                Some(Documentation::String(humanize_type(
                    db,
                    &typ,
                    RenderLevel::Simple,
                )))
            } else {
                None
            };
            ParameterInformation {
                label: ParameterLabel::Simple(label),
                documentation,
            }
        })
        .collect::<Vec<_>>();

    let label = format!("{}", params_str.join(", "));
    let signature_info = SignatureInformation {
        label,
        documentation: None,
        parameters: Some(param_infos),
        active_parameter: Some(current_idx as u32),
    };

    Some(SignatureHelp {
        signatures: vec![signature_info],
        active_signature: Some(0),
        active_parameter: Some(current_idx as u32),
    })
}

fn build_sig_id_signature_help(
    semantic_model: &SemanticModel,
    signature_id: LuaSignatureId,
    colon_call: bool,
    current_idx: usize,
) -> Option<SignatureHelp> {
    let origin_current_idx = current_idx;
    let db = semantic_model.get_db();
    let signature = db.get_signature_index().get(&signature_id)?;
    let mut current_idx = current_idx;
    let params = signature.get_type_params();
    let colon_define = signature.is_colon_define;
    let mut params_str = params
        .iter()
        .map(|param| param.0.clone())
        .collect::<Vec<_>>();
    match (colon_define, colon_call) {
        (true, false) => {
            params_str.insert(0, "self".to_string());
        }
        (false, true) => {
            if !params_str.is_empty() {
                params_str.remove(0);
            }
        }
        _ => {}
    }

    if let Some((name, _)) = params.last() {
        if name == "..." && current_idx >= params.len() {
            current_idx = params.len() - 1;
        }
    }

    let param_infos = params
        .iter()
        .map(|param| {
            let label = param.0.clone();
            let typ = param.1.clone();
            let mut documentation_string = String::new();
            if let Some(typ) = typ {
                documentation_string.push_str(
                    format!(
                        "```lua\n(parameter) {}: {}\n```\n\n",
                        label,
                        humanize_type(db, &typ, RenderLevel::Simple)
                    )
                    .as_str(),
                );
            };

            if let Some(desc) = signature.get_param_info_by_name(&label) {
                if let Some(description) = &desc.description {
                    documentation_string.push_str(description);
                }
            }

            let documentation = if documentation_string.is_empty() {
                None
            } else {
                Some(Documentation::MarkupContent(MarkupContent {
                    kind: lsp_types::MarkupKind::Markdown,
                    value: documentation_string,
                }))
            };

            ParameterInformation {
                label: ParameterLabel::Simple(label),
                documentation,
            }
        })
        .collect::<Vec<_>>();

    let label = format!("{}", params_str.join(", "));
    let property_owner = LuaSemanticDeclId::Signature(signature_id);
    let documentation =
        if let Some(property) = db.get_property_index().get_property(&property_owner) {
            if let Some(description) = &property.description {
                Some(Documentation::MarkupContent(MarkupContent {
                    kind: lsp_types::MarkupKind::Markdown,
                    value: description.to_string(),
                }))
            } else {
                None
            }
        } else {
            None
        };

    let signature_info = SignatureInformation {
        label,
        documentation,
        parameters: Some(param_infos),
        active_parameter: Some(current_idx as u32),
    };

    let mut signatures = vec![signature_info];
    for overload in &signature.overloads {
        let signature = build_doc_function_signature_help(
            &semantic_model,
            &overload,
            colon_call,
            origin_current_idx,
        );
        if let Some(signature) = signature {
            signatures.push(signature.signatures[0].clone());
        }
    }

    Some(SignatureHelp {
        signatures,
        active_signature: Some(0),
        active_parameter: Some(current_idx as u32),
    })
}

// todo support overload
fn build_type_signature_help(
    semantic_model: &SemanticModel,
    type_decl_id: &LuaTypeDeclId,
    colon_call: bool,
    current_idx: usize,
) -> Option<SignatureHelp> {
    let db = semantic_model.get_db();
    let operator_ids = db
        .get_operator_index()
        .get_operators(&type_decl_id.clone().into(), LuaOperatorMetaMethod::Call)?;

    for operator_id in operator_ids {
        let operator = db.get_operator_index().get_operator(operator_id)?;
        let call_type = operator.get_operator_func();
        if let Some(LuaType::DocFunction(f)) = call_type {
            return build_doc_function_signature_help(semantic_model, &f, colon_call, current_idx);
        }
    }

    None
}

fn build_union_type_signature_help(
    semantic_model: &SemanticModel,
    union_types: &[LuaType],
    colon_call: bool,
    current_idx: usize,
) -> Option<SignatureHelp> {
    let mut signatures = vec![];
    let active_parameter = current_idx as u32;
    for typ in union_types {
        match typ {
            LuaType::DocFunction(func_type) => {
                let sig = build_doc_function_signature_help(
                    semantic_model,
                    &func_type,
                    colon_call,
                    current_idx,
                );

                if let Some(sig) = sig {
                    signatures.push(sig.signatures[0].clone());
                }
            }
            LuaType::Signature(signature_id) => {
                let sig = build_sig_id_signature_help(
                    semantic_model,
                    *signature_id,
                    colon_call,
                    current_idx,
                );

                if let Some(sig) = sig {
                    signatures.extend(sig.signatures);
                }
            }
            LuaType::Ref(type_decl_id) => {
                let sig = build_type_signature_help(
                    semantic_model,
                    &type_decl_id,
                    colon_call,
                    current_idx,
                );

                if let Some(sig) = sig {
                    signatures.extend(sig.signatures);
                }
            }
            LuaType::Def(type_decl_id) => {
                let sig = build_type_signature_help(
                    semantic_model,
                    &type_decl_id,
                    colon_call,
                    current_idx,
                );

                if let Some(sig) = sig {
                    signatures.extend(sig.signatures);
                }
            }
            _ => {}
        }
    }

    Some(SignatureHelp {
        signatures,
        active_signature: Some(0),
        active_parameter: Some(active_parameter),
    })
}
