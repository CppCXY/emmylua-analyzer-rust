use emmylua_parser::{LuaAstNode, LuaCallExpr, LuaExpr, LuaIndexKey};

use crate::{InFiled, LuaOperatorMetaMethod, LuaOperatorOwner};

use super::LuaAnalyzer;

pub fn analyze_setmetatable(analyzer: &mut LuaAnalyzer, call_expr: LuaCallExpr) -> Option<()> {
    let arg_list = call_expr.get_args_list()?;
    let args = arg_list.get_args().collect::<Vec<_>>();
    // uncomplete setmetatable call
    if args.len() != 2 {
        return Some(());
    }

    let metatable = args[1].clone();
    let LuaExpr::TableExpr(table_expr) = metatable else {
        return Some(());
    };

    let file_id = analyzer.file_id;
    let operator_owner = LuaOperatorOwner::Table(InFiled::new(file_id, table_expr.get_range()));
    for field in table_expr.get_fields() {
        let field_name = match field.get_field_key() {
            Some(LuaIndexKey::Name(n)) => n.get_name_text().to_string(),
            Some(LuaIndexKey::String(s)) => s.get_value(),
            _ => continue,
        };

        let meta_method = LuaOperatorMetaMethod::from_metatable_name(&field_name) else {
            continue;
        };

        let operator_id = analyzer
            .db
            .get_operator_index()
            .get_operators(&operator_owner, meta_method)?;
    }

    Some(())
}
