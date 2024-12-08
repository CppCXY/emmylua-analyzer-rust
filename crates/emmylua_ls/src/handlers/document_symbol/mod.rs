mod builder;
mod stats;

use builder::{DocumentSymbolBuilder, LuaSymbol};
use code_analysis::{LuaDeclId, SemanticModel};
use emmylua_parser::{LuaAst, LuaAstNode, LuaBlock, LuaChunk, LuaClosureExpr};
use lsp_types::{DocumentSymbol, DocumentSymbolParams, DocumentSymbolResponse, SymbolKind};
use stats::{
    build_assign_stat_symbol, build_for_range_stat_symbol, build_for_stat_symbol,
    build_func_stat_symbol, build_local_func_stat_symbol, build_local_stat_symbol,
};
use tokio_util::sync::CancellationToken;

use crate::context::ServerContextSnapshot;

pub async fn on_document_symbol(
    context: ServerContextSnapshot,
    params: DocumentSymbolParams,
    _: CancellationToken,
) -> Option<DocumentSymbolResponse> {
    let uri = params.text_document.uri;
    let analysis = context.analysis.read().await;
    let file_id = analysis.get_file_id(&uri)?;
    let mut semantic_model = analysis.compilation.get_semantic_model(file_id)?;
    let document_symbol_root = build_document_symbol(&mut semantic_model)?;
    let respone = DocumentSymbolResponse::Nested(vec![document_symbol_root]);
    Some(respone)
}

fn build_document_symbol(semantic_model: &mut SemanticModel) -> Option<DocumentSymbol> {
    let document = semantic_model.get_document();
    let root = semantic_model.get_root();
    let file_id = semantic_model.get_file_id();
    let decl_tree = semantic_model
        .get_db()
        .get_decl_index()
        .get_decl_tree(&file_id)?;
    let db = semantic_model.get_db();

    let mut builder = DocumentSymbolBuilder::new(db, decl_tree, &document);
    let symbol = LuaSymbol::new(
        document.get_file_name().unwrap_or("LuaChunk".to_string()),
        None,
        SymbolKind::FILE,
        root.get_range(),
    );
    builder.add_node_symbol(root.syntax().clone(), symbol);
    build_child_document_symbols(&mut builder, root);

    Some(builder.build(root))
}

fn build_child_document_symbols(buider: &mut DocumentSymbolBuilder, root: &LuaChunk) -> Option<()> {
    for child in root.descendants::<LuaAst>() {
        match child {
            LuaAst::LuaBlock(block) => {
                build_block_symbol(buider, block);
            }
            LuaAst::LuaLocalStat(local_stat) => {
                build_local_stat_symbol(buider, local_stat);
            }
            LuaAst::LuaAssignStat(assign_stat) => {
                build_assign_stat_symbol(buider, assign_stat);
            }
            LuaAst::LuaForStat(for_stat) => {
                build_for_stat_symbol(buider, for_stat);
            }
            LuaAst::LuaForRangeStat(for_range_stat) => {
                build_for_range_stat_symbol(buider, for_range_stat);
            }
            LuaAst::LuaLocalFuncStat(local_func) => {
                build_local_func_stat_symbol(buider, local_func);
            }
            LuaAst::LuaFuncStat(func) => {
                build_func_stat_symbol(buider, func);
            }
            LuaAst::LuaClosureExpr(closure) => {
                build_closure_expr_symbol(buider, closure);
            }
            _ => {}
        }
    }

    Some(())
}

fn build_block_symbol(builder: &mut DocumentSymbolBuilder, block: LuaBlock) -> Option<()> {
    let symbol = LuaSymbol::new(
        "block".to_string(),
        None,
        SymbolKind::MODULE,
        block.get_range(),
    );

    builder.add_node_symbol(block.syntax().clone(), symbol);
    Some(())
}

fn build_closure_expr_symbol(
    builder: &mut DocumentSymbolBuilder,
    closure: LuaClosureExpr,
) -> Option<()> {
    let symbol = LuaSymbol::new(
        "closure".to_string(),
        None,
        SymbolKind::MODULE,
        closure.get_range(),
    );

    builder.add_node_symbol(closure.syntax().clone(), symbol);

    let file_id = builder.get_file_id();
    let param_list = closure.get_params_list()?;
    for param in param_list.get_params() {
        let decl_id = LuaDeclId::new(file_id, param.get_position());
        let decl = builder.get_decl(&decl_id)?;
        let desc = builder.get_symbol_kind_and_detail(decl.get_type());
        let symbol = LuaSymbol::new(
            decl.get_name().to_string(),
            desc.1,
            desc.0,
            decl.get_range(),
        );

        builder.add_node_symbol(param.syntax().clone(), symbol);
    }

    Some(())
}