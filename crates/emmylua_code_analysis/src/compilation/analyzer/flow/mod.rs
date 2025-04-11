mod build_flow_tree;
mod cast_analyze;
mod flow_node;
mod flow_tree;
mod var_analyze;

use std::collections::HashMap;

use crate::{db_index::DbIndex, profile::Profile, FileId, LuaFlowChain};
use build_flow_tree::build_flow_tree;
use cast_analyze::analyze_cast;
use flow_tree::{FlowTree, VarRefNode};
use var_analyze::{analyze_ref_assign, analyze_ref_expr, VarTrace};

use super::AnalyzeContext;

pub(crate) fn analyze(db: &mut DbIndex, context: &mut AnalyzeContext) {
    let _p = Profile::cond_new("flow analyze", context.tree_list.len() > 1);
    let tree_list = context.tree_list.clone();
    // build decl and ref flow chain
    for in_filed_tree in &tree_list {
        let flow_tree = build_flow_tree(db, in_filed_tree.file_id, in_filed_tree.value.clone());
        analyze_flow(db, in_filed_tree.file_id, flow_tree, context);
    }
}

fn analyze_flow(
    db: &mut DbIndex,
    file_id: FileId,
    flow_tree: FlowTree,
    context: &mut AnalyzeContext,
) {
    let var_ref_ids = flow_tree.get_var_ref_ids();
    // let mut flow_chain_map = HashMap::new();
    for var_ref_id in var_ref_ids {
        let var_ref_nodes = match flow_tree.get_var_ref_nodes(&var_ref_id) {
            Some(nodes) => nodes,
            None => continue,
        };

        let mut var_trace = VarTrace::new(var_ref_id.clone());
        for (var_ref_node, flow_id) in var_ref_nodes {
            // let mut flow_chain = flow_chain_map
            //     .entry(flow_id)
            //     .or_insert_with(|| LuaFlowChain::new(*flow_id));
            var_trace.set_current_flow_id(*flow_id);
            match var_ref_node {
                VarRefNode::UseRef(var_expr) => {
                    analyze_ref_expr(db, &mut var_trace, &var_expr);
                }
                VarRefNode::AssignRef(var_expr) => {
                    analyze_ref_assign(db, &mut var_trace, &var_expr, file_id);
                }
                VarRefNode::CastRef(tag_cast) => {
                    analyze_cast(&mut var_trace, file_id, tag_cast.clone(), context);
                }
            }
        }
    }

    // for (_, flow_chain) in flow_chain_map {
    //     db.get_flow_index_mut().add_flow_chain(file_id, flow_chain);
    // }
}
