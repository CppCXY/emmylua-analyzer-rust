use emmylua_parser::{LuaAst, LuaAstNode, LuaBlock};

use crate::{DbIndex, LuaFlowChain, TypeAssertion, VarRefId};

use super::{broadcast_down::broadcast_down, VarTrace};

pub fn broadcast_outside_block(
    db: &mut DbIndex,
    var_trace: &mut VarTrace,
    block: LuaBlock,
    type_assert: TypeAssertion,
) -> Option<()> {
    let parent = block.get_parent::<LuaAst>()?;
    match &parent {
        LuaAst::LuaIfStat(_)
        | LuaAst::LuaDoStat(_)
        | LuaAst::LuaWhileStat(_)
        | LuaAst::LuaForStat(_)
        | LuaAst::LuaForRangeStat(_)
        | LuaAst::LuaRepeatStat(_) => {
            broadcast_down(db, var_trace, parent, type_assert, false);
        }
        LuaAst::LuaElseIfClauseStat(_) | LuaAst::LuaElseClauseStat(_) => {
            broadcast_down(
                db,
                var_trace,
                parent.get_parent::<LuaAst>()?,
                type_assert,
                false,
            );
        }
        _ => {}
    }

    Some(())
}
