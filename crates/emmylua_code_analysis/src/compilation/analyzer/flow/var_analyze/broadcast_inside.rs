use emmylua_parser::{LuaAst, LuaAstNode, LuaBlock, LuaStat};

use crate::{DbIndex, LuaFlowChain, TypeAssertion, VarRefId};

use super::{broadcast_down::broadcast_down, VarTrace};

pub fn broadcast_inside_block(
    db: &mut DbIndex,
    var_trace: &mut VarTrace,
    block: LuaBlock,
    type_assert: TypeAssertion,
) -> Option<()> {
    // let parent = node.get_parent::<LuaAst>()?;
    // match &parent {
    //     LuaAst::LuaIfStat(_)
    //     | LuaAst::LuaDoStat(_)
    //     | LuaAst::LuaWhileStat(_)
    //     | LuaAst::LuaForStat(_)
    //     | LuaAst::LuaForRangeStat(_)
    //     | LuaAst::LuaRepeatStat(_) => {
    //         broadcast_down(db, flow_chain, var_ref_id, parent, type_assert, false);
    //     }
    //     LuaAst::LuaElseIfClauseStat(_) | LuaAst::LuaElseClauseStat(_) => {
    //         broadcast_down(
    //             db,
    //             flow_chain,
    //             var_ref_id,
    //             parent.get_parent::<LuaAst>()?,
    //             type_assert,
    //             false,
    //         );
    //     }
    //     _ => {}
    // }

    // Some(())
    todo!()
}


fn is_block_has_return(block: Option<LuaBlock>) -> Option<bool> {
    if let Some(block) = block {
        for stat in block.get_stats() {
            if is_stat_change_flow(stat.clone()).unwrap_or(false) {
                return Some(true);
            }
        }
    }

    Some(false)
}

fn is_stat_change_flow(stat: LuaStat) -> Option<bool> {
    match stat {
        LuaStat::CallExprStat(call_stat) => {
            let call_expr = call_stat.get_call_expr()?;
            if call_expr.is_error() {
                return Some(true);
            }
            Some(false)
        }
        LuaStat::ReturnStat(_) => Some(true),
        LuaStat::DoStat(do_stat) => Some(is_block_has_return(do_stat.get_block()).unwrap_or(false)),
        LuaStat::BreakStat(_) => Some(true),
        _ => Some(false),
    }
}
