mod actions;
mod build_actions;

use build_actions::build_actions;
use lsp_types::{
    ClientCapabilities, CodeActionParams, CodeActionProviderCapability, CodeActionResponse,
    ServerCapabilities,
};
use tokio_util::sync::CancellationToken;

use crate::context::ServerContextSnapshot;

#[allow(unused_variables)]
pub async fn on_code_action_handler(
    context: ServerContextSnapshot,
    params: CodeActionParams,
    _: CancellationToken,
) -> Option<CodeActionResponse> {
    let uri = params.text_document.uri;
    let diagnostics = params.context.diagnostics;
    let analysis = context.analysis.read().await;
    let file_id = analysis.get_file_id(&uri)?;
    let mut semantic_model = analysis.compilation.get_semantic_model(file_id)?;

    build_actions(&mut semantic_model, diagnostics)
}

pub fn register_capabilities(
    server_capabilities: &mut ServerCapabilities,
    _: &ClientCapabilities,
) -> Option<()> {
    server_capabilities.code_action_provider = Some(CodeActionProviderCapability::Simple(true));
    Some(())
}