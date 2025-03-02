use emmylua_parser::{
    BinaryOperator, LuaAst, LuaAstNode, LuaBinaryExpr, LuaBlock, LuaCallArgList, LuaCallExpr,
    LuaExpr, LuaIndexExpr, LuaLiteralToken, LuaNameExpr, LuaStat, LuaSyntaxKind, PathTrait,
    UnaryOperator,
};
use rowan::TextRange;

use crate::{
    db_index::{LuaType, TypeAssertion},
    LuaTypeDeclId,
};

use super::FlowAnalyzer;

pub fn infer_name_expr(analyzer: &mut FlowAnalyzer, name_expr: LuaNameExpr) -> Option<()> {
    let parent = name_expr.get_parent::<LuaAst>()?;
    match &parent {
        LuaAst::LuaIndexExpr(_) => return None,
        LuaAst::LuaAssignStat(_) => {

        }
        _ => {}
    }

    let path = name_expr.get_access_path()?;
    broadcast_up(
        analyzer,
        &path,
        parent,
        LuaAst::LuaNameExpr(name_expr),
        TypeAssertion::Exist,
    );

    Some(())
}

pub fn infer_index_expr(analyzer: &mut FlowAnalyzer, index_expr: LuaIndexExpr) -> Option<()> {
    let parent = index_expr.get_parent::<LuaAst>()?;
    match &parent {
        LuaAst::LuaIndexExpr(_) => return None,
        LuaAst::LuaAssignStat(_) => {
            
        }
        _ => {}
    }

    let path = index_expr.get_access_path()?;
    broadcast_up(
        analyzer,
        &path,
        parent,
        LuaAst::LuaIndexExpr(index_expr),
        TypeAssertion::Exist,
    );

    Some(())
}

fn broadcast_up(
    analyzer: &mut FlowAnalyzer,
    path: &str,
    parent: LuaAst,
    origin: LuaAst,
    type_assert: TypeAssertion,
) -> Option<()> {
    match parent {
        LuaAst::LuaIfStat(if_stat) => {
            // this mean the name_expr is a condition and the name_expr is not nil and is not false
            if let Some(block) = if_stat.get_block() {
                analyzer.add_type_assert(path, type_assert.clone(), block.get_range());
            }

            if let Some(ne_type_assert) = type_assert.get_negation() {
                if let Some(else_stat) = if_stat.get_else_clause() {
                    let range = else_stat.get_range();
                    analyzer.add_type_assert(path, ne_type_assert, range);
                } else if is_block_has_return(if_stat.get_block()?).unwrap_or(false) {
                    let parent_block = if_stat.get_parent::<LuaBlock>()?;
                    let parent_range = parent_block.get_range();
                    let if_range = if_stat.get_range();
                    if if_range.end() < parent_range.end() {
                        let range = TextRange::new(if_range.end(), parent_range.end());
                        analyzer.add_type_assert(path, ne_type_assert, range);
                    }
                }
            }
        }
        LuaAst::LuaWhileStat(while_stat) => {
            // this mean the name_expr is a condition and the name_expr is not nil and is not false
            let block = while_stat.get_block()?;
            analyzer.add_type_assert(path, type_assert, block.get_range());
        }
        LuaAst::LuaElseIfClauseStat(else_if_clause_stat) => {
            // this mean the name_expr is a condition and the name_expr is not nil and is not false
            let block = else_if_clause_stat.get_block()?;
            analyzer.add_type_assert(path, type_assert, block.get_range());
        }
        LuaAst::LuaIndexExpr(index_expr) => {
            if index_expr.get_position() != origin.get_position() {
                return None;
            }

            let member_path = index_expr.get_member_path()?;

            let type_assert = TypeAssertion::MemberPathExist(member_path.into());
            broadcast_up(
                analyzer,
                path,
                index_expr.get_parent::<LuaAst>()?,
                LuaAst::LuaIndexExpr(index_expr),
                type_assert,
            );
        }
        LuaAst::LuaBinaryExpr(binary_expr) => {
            let op = binary_expr.get_op_token()?;
            match op.get_op() {
                BinaryOperator::OpAnd => {
                    let (left, right) = binary_expr.get_exprs()?;
                    if left.get_position() == origin.get_position() {
                        analyzer.add_type_assert(path, type_assert.clone(), right.get_range());
                    }

                    broadcast_up(
                        analyzer,
                        path,
                        binary_expr.get_parent::<LuaAst>()?,
                        LuaAst::LuaBinaryExpr(binary_expr),
                        type_assert,
                    );
                }
                BinaryOperator::OpEq => {
                    if origin.syntax().kind() != LuaSyntaxKind::NameExpr.into() {
                        return None;
                    }

                    let (left, right) = binary_expr.get_exprs()?;
                    let expr = if left.get_position() == origin.get_position() {
                        right
                    } else {
                        left
                    };

                    if let LuaExpr::LiteralExpr(literal) = expr {
                        let type_assert = match literal.get_literal()? {
                            LuaLiteralToken::Nil(_) => TypeAssertion::NotExist,
                            LuaLiteralToken::Bool(b) => {
                                if b.is_true() {
                                    TypeAssertion::Exist
                                } else {
                                    TypeAssertion::NotExist
                                }
                            }
                            LuaLiteralToken::Number(_) => TypeAssertion::Narrow(LuaType::Number),
                            LuaLiteralToken::String(_) => TypeAssertion::Narrow(LuaType::String),
                            _ => return None,
                        };

                        broadcast_up(
                            analyzer,
                            path,
                            binary_expr.get_parent::<LuaAst>()?,
                            LuaAst::LuaBinaryExpr(binary_expr),
                            type_assert,
                        );
                    }
                }
                _ => {}
            }
        }
        LuaAst::LuaCallArgList(call_args_list) => {
            if type_assert == TypeAssertion::Exist {
                infer_call_arg_list(analyzer, path, call_args_list)?;
            }
        }
        LuaAst::LuaUnaryExpr(unary_expr) => {
            let op = unary_expr.get_op_token()?;
            match op.get_op() {
                UnaryOperator::OpNot => {
                    if let Some(ne_type_assert) = type_assert.get_negation() {
                        broadcast_up(
                            analyzer,
                            path,
                            unary_expr.get_parent::<LuaAst>()?,
                            LuaAst::LuaUnaryExpr(unary_expr),
                            ne_type_assert,
                        );
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
    Some(())
}

fn infer_call_arg_list(
    analyzer: &mut FlowAnalyzer,
    path: &str,
    call_arg: LuaCallArgList,
) -> Option<()> {
    let parent = call_arg.get_parent::<LuaAst>()?;
    match parent {
        LuaAst::LuaCallExpr(call_expr) => {
            if let LuaExpr::NameExpr(prefix_expr) = call_expr.get_prefix_expr()? {
                let name_text = prefix_expr.get_name_text()?;
                if name_text == "type" {
                    infer_lua_type_assert(analyzer, path, call_expr);
                }
            }
        }
        _ => {}
    }

    Some(())
}

fn infer_lua_type_assert(
    analyzer: &mut FlowAnalyzer,
    path: &str,
    call_expr: LuaCallExpr,
) -> Option<()> {
    let binary_expr = call_expr.get_parent::<LuaBinaryExpr>()?;
    let op = binary_expr.get_op_token()?;
    match op.get_op() {
        BinaryOperator::OpEq => {}
        _ => return None,
    };

    let operands = binary_expr.get_exprs()?;
    let literal_expr = if let LuaExpr::LiteralExpr(literal) = operands.0 {
        literal
    } else if let LuaExpr::LiteralExpr(literal) = operands.1 {
        literal
    } else {
        return None;
    };

    let type_literal = match literal_expr.get_literal()? {
        LuaLiteralToken::String(string) => string.get_value(),
        _ => return None,
    };

    let type_assert = match type_literal.as_str() {
        "number" => TypeAssertion::Narrow(LuaType::Number),
        "string" => TypeAssertion::Narrow(LuaType::String),
        "boolean" => TypeAssertion::Narrow(LuaType::Boolean),
        "table" => TypeAssertion::Narrow(LuaType::Table),
        "function" => TypeAssertion::Narrow(LuaType::Function),
        "thread" => TypeAssertion::Narrow(LuaType::Thread),
        "userdata" => TypeAssertion::Narrow(LuaType::Userdata),
        "nil" => TypeAssertion::Narrow(LuaType::Nil),
        // extend usage
        str => TypeAssertion::Narrow(LuaType::Ref(LuaTypeDeclId::new(str))),
    };

    broadcast_up(
        analyzer,
        path,
        binary_expr.get_parent::<LuaAst>()?,
        LuaAst::LuaBinaryExpr(binary_expr),
        type_assert,
    );

    Some(())
}

fn is_block_has_return(block: LuaBlock) -> Option<bool> {
    for stat in block.get_stats() {
        match stat {
            LuaStat::ReturnStat(_) => return Some(true),
            LuaStat::DoStat(do_stat) => {
                if is_block_has_return(do_stat.get_block()?).unwrap_or(false) {
                    return Some(true);
                }
            }
            _ => {}
        }
    }

    Some(false)
}
