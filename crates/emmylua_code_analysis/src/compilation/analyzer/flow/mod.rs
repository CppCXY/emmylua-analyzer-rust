mod assign_stats;
mod expr_analyze;

use std::collections::HashMap;

use crate::{db_index::DbIndex, profile::Profile, FileId, LuaFlowChain, LuaFlowId, TypeAssertion};
use emmylua_parser::{LuaAstNode, LuaChunk, LuaExpr};
use expr_analyze::{infer_index_expr, infer_name_expr};
use rowan::{TextRange, WalkEvent};

use super::AnalyzeContext;

pub(crate) fn analyze(db: &mut DbIndex, context: &mut AnalyzeContext) {
    let _p = Profile::cond_new("flow analyze", context.tree_list.len() > 1);
    let tree_list = context.tree_list.clone();
    // build decl and ref flow chain
    for in_filed_tree in &tree_list {
        flow_analyze(db, in_filed_tree.file_id, in_filed_tree.value.clone());
    }
}

fn flow_analyze(db: &mut DbIndex, file_id: FileId, root: LuaChunk) -> Option<()> {
    let mut analyzer = FlowAnalyzer::new(db, file_id, root.clone());
    for walk_expr in root.walk_descendants::<LuaExpr>() {
        match walk_expr {
            WalkEvent::Enter(expr) => match expr {
                LuaExpr::NameExpr(name_expr) => {
                    infer_name_expr(&mut analyzer, name_expr);
                }
                LuaExpr::IndexExpr(index_expr) => {
                    infer_index_expr(&mut analyzer, index_expr);
                }
                LuaExpr::ClosureExpr(closure) => {
                    analyzer.push_flow(LuaFlowId::from_closure(closure));
                }
                _ => {}
            },
            WalkEvent::Leave(LuaExpr::ClosureExpr(_)) => analyzer.pop_flow(),
            _ => {}
        }
    }

    analyzer.finish()
}


#[allow(unused)]
#[derive(Debug)]
struct FlowAnalyzer<'a> {
    db: &'a mut DbIndex,
    file_id: FileId,
    root: LuaChunk,
    current_flow_id: LuaFlowId,
    flow_id_stack: Vec<LuaFlowId>,
    flow_chains: HashMap<LuaFlowId, LuaFlowChain>,
}

impl FlowAnalyzer<'_> {
    pub fn new<'a>(db: &'a mut DbIndex, file_id: FileId, root: LuaChunk) -> FlowAnalyzer<'a> {
        FlowAnalyzer {
            db,
            file_id,
            root,
            current_flow_id: LuaFlowId::chunk(),
            flow_id_stack: vec![LuaFlowId::chunk()],
            flow_chains: HashMap::new(),
        }
    }

    pub fn push_flow(&mut self, flow_id: LuaFlowId) {
        self.flow_id_stack.push(flow_id);
        self.current_flow_id = flow_id;
    }

    pub fn pop_flow(&mut self) {
        self.flow_id_stack.pop();
        self.current_flow_id = self
            .flow_id_stack
            .last()
            .cloned()
            .unwrap_or(LuaFlowId::chunk());
    }

    pub fn add_type_assert(&mut self, path: &str, type_assert: TypeAssertion, range: TextRange) {
        self.flow_chains
            .entry(self.current_flow_id)
            .or_insert_with(|| LuaFlowChain::new(self.current_flow_id))
            .add_type_assert(path, type_assert, range);
    }

    pub fn finish(self) -> Option<()> {
        let flow_index = self.db.get_flow_index_mut();
        for (_, flow_chain) in self.flow_chains {
            flow_index.add_flow_chain(self.file_id, flow_chain);
        }

        Some(())
    }
}
