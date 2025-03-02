// use std::collections::HashMap;

// use emmylua_parser::{LuaAssignStat, LuaAstNode, LuaNameExpr, LuaSyntaxId, LuaSyntaxKind};
// use emmylua_parser::{LuaAst, LuaBlock, LuaExpr, LuaIfStat, LuaSyntaxNode};
// use rowan::TextRange;

// use crate::{DeclReference, LuaFlowChain, TypeAssertion};

// use super::FlowAnalyzer;

// pub fn infer_from_assign_stats(
//     analyzer: &FlowAnalyzer,
//     flow_chains: &mut LuaFlowChain,
//     decl_refs: Vec<&DeclReference>,
// ) -> Option<()> {
//     let mut assign_value_exprs = Vec::new();

//     for decl_ref in decl_refs {
//         if decl_ref.is_write {
//             infer_one_assign_stat(analyzer, flow_chains, &mut assign_value_exprs, decl_ref);
//         }
//     }

//     let decl_id = flow_chains.get_decl_id();
//     let syntax_id = analyzer
//         .db
//         .get_decl_index()
//         .get_decl(&decl_id)?
//         .get_syntax_id();
//     let decl_root = syntax_id.to_node_from_root(analyzer.root.syntax())?;
//     let mut if_stat_related_exprs: HashMap<LuaIfStat, Vec<(LuaExpr, i32)>> = HashMap::new();
//     for value_expr in assign_value_exprs {
//         let if_stat = match find_if_stat(&value_expr.0, &decl_root) {
//             Some(if_stat) => if_stat,
//             None => continue,
//         };

//         if let Some(exprs) = if_stat_related_exprs.get_mut(&if_stat) {
//             exprs.push(value_expr);
//         } else {
//             if_stat_related_exprs.insert(if_stat, vec![value_expr]);
//         }
//     }

//     for (if_stat, if_related_exprs) in if_stat_related_exprs {
//         analyze_if_stat(flow_chains, if_related_exprs, if_stat);
//     }

//     Some(())
// }

// fn infer_one_assign_stat(
//     analyzer: &FlowAnalyzer,
//     flow_chains: &mut LuaFlowChain,
//     value_exprs: &mut Vec<(LuaExpr, i32)>,
//     decl_ref: &DeclReference,
// ) -> Option<()> {
//     let assign_stat = find_assign_stat(analyzer, decl_ref)?;
//     let value_expr = find_assign_value_expr(&assign_stat, decl_ref)?;
//     value_exprs.push(value_expr.clone());

//     let effect_range = get_effect_range(&assign_stat)?;
//     flow_chains.add_type_assert(
//         TypeAssertion::Reassign((value_expr.0.get_syntax_id(), value_expr.1)),
//         effect_range,
//     );

//     Some(())
// }

// fn find_assign_stat(analyzer: &FlowAnalyzer, decl_ref: &DeclReference) -> Option<LuaAssignStat> {
//     let syntax_id = LuaSyntaxId::new(LuaSyntaxKind::NameExpr.into(), decl_ref.range.clone());
//     let name_expr = LuaNameExpr::cast(syntax_id.to_node_from_root(analyzer.root.syntax())?)?;
//     name_expr.get_parent::<LuaAssignStat>()
// }

// fn find_assign_value_expr(
//     assign_stat: &LuaAssignStat,
//     decl_ref: &DeclReference,
// ) -> Option<(LuaExpr, i32)> {
//     let (vars, exprs) = assign_stat.get_var_and_expr_list();
//     if exprs.len() == 0 {
//         return None;
//     }

//     let var_index = vars
//         .iter()
//         .position(|var| var.get_position() == decl_ref.range.start())?;

//     match exprs.get(var_index) {
//         Some(expr) => Some((expr.clone(), 0)),
//         None => Some((
//             exprs.last().unwrap().clone(),
//             (var_index - exprs.len()) as i32,
//         )),
//     }
// }

// fn get_effect_range(assign_stat: &LuaAssignStat) -> Option<TextRange> {
//     let block = assign_stat.get_parent::<LuaBlock>()?;
//     let block_range = block.get_range();
//     let assign_end_offset = assign_stat.get_range().end();
//     let block_end_offset = block_range.end();
//     if block_end_offset <= assign_end_offset {
//         return None;
//     }
//     Some(TextRange::new(assign_end_offset, block_range.end()))
// }

// fn find_if_stat(value_expr: &LuaExpr, decl_root: &LuaSyntaxNode) -> Option<LuaIfStat> {
//     let mut parent = value_expr.get_parent::<LuaAst>()?;
//     loop {
//         if parent.syntax() == decl_root {
//             return None;
//         }

//         if let Some(if_stat) = LuaIfStat::cast(parent.syntax().clone()) {
//             return Some(if_stat);
//         }

//         parent = parent.get_parent::<LuaAst>()?;
//     }
// }

// fn analyze_if_stat(
//     flow_chains: &mut LuaFlowChain,
//     if_related_exprs: Vec<(LuaExpr, i32)>,
//     if_stat: LuaIfStat,
// ) -> Option<()> {
//     let parent_block_range = if_stat.get_parent::<LuaBlock>()?.get_range();
//     let if_end_offset = if_stat.get_range().end();
//     if if_end_offset >= parent_block_range.end() {
//         return None;
//     }
//     let effect_range = TextRange::new(if_end_offset, parent_block_range.end());
//     let block = if_stat.get_block()?;
//     flow_chains.add_type_assert(crate::TypeAssertion::Exist, block.get_range());

//     let mut all_branch_has_reassign = true;
//     if let Some(block) = if_stat.get_block() {
//         all_branch_has_reassign = is_block_has_reassign(block, &if_related_exprs).unwrap_or(false);
//     }

//     // it is not correct to infer all branch has reassign, need optimize
//     if all_branch_has_reassign {
//         for clause in if_stat.get_all_clause() {
//             if let Some(block) = clause.get_block() {
//                 if !is_block_has_reassign(block, &if_related_exprs).unwrap_or(false) {
//                     all_branch_has_reassign = false;
//                     break;
//                 }
//             }
//         }
//     }

//     if all_branch_has_reassign {
//         flow_chains.add_type_assert(TypeAssertion::Exist, effect_range);
//     }

//     let reassign_vec = if_related_exprs
//         .iter()
//         .map(|expr| (expr.0.get_syntax_id(), expr.1))
//         .collect();

//     flow_chains.add_type_assert(TypeAssertion::AddUnion(reassign_vec), effect_range);

//     Some(())
// }

// fn is_block_has_reassign(block: LuaBlock, if_related_exprs: &Vec<(LuaExpr, i32)>) -> Option<bool> {
//     let mut has_reassign = false;
//     let range = block.get_range();
//     for (expr, _) in if_related_exprs {
//         if range.contains_range(expr.get_range()) {
//             has_reassign = true;
//             break;
//         }
//     }

//     Some(has_reassign)
// }
