mod assign_stats;
mod name_ref;

use crate::{db_index::DbIndex, profile::Profile, FileId, LuaFlowChain};
use assign_stats::infer_from_assign_stats;
use emmylua_parser::{LuaAstNode, LuaChunk, LuaNameExpr, LuaSyntaxId, LuaSyntaxKind};
use name_ref::infer_name_expr;

use super::AnalyzeContext;

pub(crate) fn analyze(db: &mut DbIndex, context: &mut AnalyzeContext) {
    let _p = Profile::cond_new("flow analyze", context.tree_list.len() > 1);
    let tree_list = context.tree_list.clone();
    // build decl and ref flow chain
    for in_filed_tree in &tree_list {
        local_flow_analyze(db, in_filed_tree.file_id, in_filed_tree.value.clone());
    }
}

// local analyze
fn local_flow_analyze(db: &mut DbIndex, file_id: FileId, root: LuaChunk) -> Option<()> {
    let references_index = db.get_reference_index();
    let refs_map = references_index.get_decl_references_map(&file_id)?.clone();

    let mut analyzer = FlowAnalyzer::new(db, root.clone());
    for (decl_id, decl_refs) in refs_map {
        let mut flow_chains = LuaFlowChain::new(decl_id);

        let mut need_assign_infer = false;
        for decl_ref in &decl_refs {
            if !decl_ref.is_write {
                let syntax_id =
                    LuaSyntaxId::new(LuaSyntaxKind::NameExpr.into(), decl_ref.range.clone());
                if let Some(name_expr) =
                    LuaNameExpr::cast(syntax_id.to_node_from_root(root.syntax())?)
                {
                    infer_name_expr(&mut analyzer, &mut flow_chains, name_expr);
                }
            } else {
                need_assign_infer = true;
            }
        }

        if need_assign_infer {
            infer_from_assign_stats(&mut analyzer, &mut flow_chains, decl_refs.iter().collect());
        }

        analyzer
            .db
            .get_flow_index_mut()
            .add_flow_chain(file_id, flow_chains);
    }

    Some(())
}

// todo self analyze

#[derive(Debug)]
struct FlowAnalyzer<'a> {
    db: &'a mut DbIndex,
    root: LuaChunk,
}

impl FlowAnalyzer<'_> {
    pub fn new<'a>(db: &'a mut DbIndex, root: LuaChunk) -> FlowAnalyzer<'a> {
        FlowAnalyzer { db, root }
    }
}
