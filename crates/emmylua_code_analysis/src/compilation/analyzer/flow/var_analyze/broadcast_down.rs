use emmylua_parser::{LuaAst, LuaAstNode, LuaBlock};
use rowan::TextRange;

use crate::{DbIndex, LuaFlowChain, TypeAssertion, VarRefId};

use super::{broadcast_outside::broadcast_outside_block, VarTrace};

pub fn broadcast_down(
    db: &mut DbIndex,
    var_trace: &mut VarTrace,
    node: LuaAst,
    type_assert: TypeAssertion,
    continue_broadcast_outside: bool,
) -> Option<()> {
    // let parent_block = node.get_parent::<LuaBlock>()?;
    // let parent_range = parent_block.get_range();
    // let range = node.get_range();
    // if range.end() < parent_range.end() {
    //     let range = TextRange::new(range.end(), parent_range.end());
    //     flow_chain.add_type_assert(var_ref_id, type_assert.clone(), range, range);
    // }

    // if continue_broadcast_outside {
    //     broadcast_outside(db, flow_chain, var_ref_id, parent_block, type_assert);
    // }
    todo!()

    // Some(())
}
