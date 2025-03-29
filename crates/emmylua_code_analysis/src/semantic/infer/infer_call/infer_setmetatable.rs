use emmylua_parser::{LuaAstNode, LuaCallExpr, LuaExpr};

use crate::{
    infer_expr, semantic::infer::InferResult, DbIndex, InFiled, InferFailReason, LuaInferCache,
    LuaInstanceType, LuaType,
};

pub fn infer_setmetatable_call(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    call_expr: LuaCallExpr,
) -> InferResult {
    let arg_list = call_expr.get_args_list().ok_or(InferFailReason::None)?;
    let args = arg_list.get_args().collect::<Vec<LuaExpr>>();
    // uncomplete setmetatable call
    if args.len() != 2 {
        return Ok(LuaType::Any);
    }

    let basic_table = args[0].clone();
    let metatable = args[1].clone();

    let meta_type = infer_expr(db, cache, metatable)?;
    match &basic_table {
        LuaExpr::TableExpr(table_expr) => {
            if table_expr.is_empty() {
                return Ok(meta_type);
            } else {
                let file_id = cache.get_file_id();
                let inst = LuaType::Instance(
                    LuaInstanceType::new(meta_type, InFiled::new(file_id, basic_table.get_range()))
                        .into(),
                );
                return Ok(inst);
            }
        }
        _ => Ok(meta_type),
    }
}
