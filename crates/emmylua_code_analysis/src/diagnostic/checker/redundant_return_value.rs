use emmylua_parser::{LuaAstNode, LuaBlock, LuaClosureExpr, LuaReturnStat};

use crate::{DiagnosticCode, LuaSignatureId, LuaType, SemanticModel, SignatureReturnStatus};

use super::DiagnosticContext;

pub const CODES: &[DiagnosticCode] = &[DiagnosticCode::RedundantReturnValue];

pub fn check(context: &mut DiagnosticContext, semantic_model: &SemanticModel) -> Option<()> {
    let root = semantic_model.get_root().clone();
    for return_stat in root.descendants::<LuaReturnStat>() {
        check_return_stat(context, semantic_model, &return_stat);
    }
    Some(())
}

fn check_return_stat(
    context: &mut DiagnosticContext,
    semantic_model: &SemanticModel,
    return_stat: &LuaReturnStat,
) -> Option<()> {
    let closure_expr = return_stat
        .get_parent::<LuaBlock>()?
        .ancestors::<LuaClosureExpr>()
        .next()?;

    let signature_id = LuaSignatureId::from_closure(semantic_model.get_file_id(), &closure_expr);
    let signature = context.db.get_signature_index().get(&signature_id)?;
    let return_types = signature.get_return_types();

    if signature.resolve_return != SignatureReturnStatus::DocResolve {
        return None;
    }

    if return_types.iter().any(|ty| ty.is_variadic()) {
        return Some(());
    }
    let return_types_len = return_types.len();

    let mut current_expr_len = 0;
    let mut diagnostics = Vec::new();
    for expr in return_stat.get_expr_list() {
        let expr_type = semantic_model.infer_expr(expr.clone())?;
        match expr_type {
            LuaType::MuliReturn(types) => {
                current_expr_len += types.get_len().map(|len| len as usize).unwrap_or(1);
            }
            _ => current_expr_len += 1,
        };

        if current_expr_len > return_types_len {
            diagnostics.push(expr.get_range());
        }
    }

    for range in diagnostics {
        context.add_diagnostic(
            DiagnosticCode::RedundantReturnValue,
            range,
            t!(
                    "Annotations specify that at most %{max} return value(s) are required, found %{rmax} returned here instead.",
                    max = return_types_len,
                    rmax = current_expr_len
                )
                .to_string(),
                None,
            );
    }
    Some(())
}
