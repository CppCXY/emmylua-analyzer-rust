use emmylua_parser::{
    BinaryOperator, LuaAst, LuaAstNode, LuaExpr, LuaLiteralToken, UnaryOperator,
};
use smol_str::SmolStr;

use crate::{DbIndex, LuaType, TypeAssertion};

use super::{
    broadcast_inside::broadcast_inside_block,
    infer_call_arg_list, VarTrace,
};

pub fn broadcast_up(
    db: &mut DbIndex,
    var_trace: &mut VarTrace,
    parent: LuaAst,
    origin: LuaAst,
    type_assert: TypeAssertion,
) -> Option<()> {
    match parent {
        LuaAst::LuaIfStat(if_stat) => {
            if let Some(block) = if_stat.get_block() {
                broadcast_inside_block(db, var_trace, block, type_assert.clone());
            }

            if let Some(ne_type_assert) = type_assert.get_negation() {
                if let Some(else_stat) = if_stat.get_else_clause() {
                    broadcast_inside_block(db, var_trace, else_stat.get_block()?, type_assert);
                }

                for else_if_clause in if_stat.get_else_if_clause_list() {
                    let range = else_if_clause.get_range();
                    var_trace.add_assert(ne_type_assert.clone(), range);
                }
            }
        }
        LuaAst::LuaWhileStat(while_stat) => {
            // this mean the name_expr is a condition and the name_expr is not nil and is not false
            let block = while_stat.get_block()?;
            broadcast_inside_block(db, var_trace, block, type_assert);
        }
        LuaAst::LuaElseIfClauseStat(else_if_clause_stat) => {
            // this mean the name_expr is a condition and the name_expr is not nil and is not false
            if let Some(block) = else_if_clause_stat.get_block() {
                broadcast_inside_block(db, var_trace, block, type_assert.clone());
            }

            // todo
            // for else_if_clause
            // if let Some(ne_type_assert) = type_assert.get_negation() {
            //     if let Some(else_stat) = if_stat.get_else_clause() {
            //         broadcast_inside_block(db, var_trace, else_stat.get_block()?, type_assert);
            //     }

            //     for else_if_clause in if_stat.get_else_if_clause_list() {
            //         let range = else_if_clause.get_range();
            //         var_trace.add_and_assert(ne_type_assert.clone(), range);
            //     }
            // }
        }
        LuaAst::LuaParenExpr(paren_expr) => {
            broadcast_up(
                db,
                var_trace,
                paren_expr.get_parent::<LuaAst>()?,
                LuaAst::LuaParenExpr(paren_expr),
                type_assert,
            );
        }
        LuaAst::LuaBinaryExpr(binary_expr) => {
            let op = binary_expr.get_op_token()?;
            match op.get_op() {
                BinaryOperator::OpAnd => {
                    let (left, right) = binary_expr.get_exprs()?;
                    if left.get_position() == origin.get_position() {
                        var_trace.add_assert(type_assert.clone(), right.get_range());
                    }

                    var_trace.add_and_assert(type_assert, binary_expr);
                }
                BinaryOperator::OpOr => {
                    let (left, right) = binary_expr.get_exprs()?;
                    if left.get_position() == origin.get_position() {
                        if let Some(ne) = type_assert.get_negation() {
                            var_trace.add_assert(ne, right.get_range());
                        }
                    }

                    var_trace.add_or_assert(type_assert, binary_expr);
                }
                BinaryOperator::OpEq => {
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
                            LuaLiteralToken::Number(i) => {
                                if i.is_int() {
                                    TypeAssertion::Narrow(LuaType::IntegerConst(i.get_int_value()))
                                } else {
                                    TypeAssertion::Narrow(LuaType::Number)
                                }
                            }
                            LuaLiteralToken::String(s) => TypeAssertion::Narrow(
                                LuaType::StringConst(SmolStr::new(s.get_value()).into()),
                            ),
                            _ => return None,
                        };

                        broadcast_up(
                            db,
                            var_trace,
                            binary_expr.get_parent::<LuaAst>()?,
                            LuaAst::LuaBinaryExpr(binary_expr),
                            type_assert,
                        );
                    }
                }
                BinaryOperator::OpNe => {
                    let (left, right) = binary_expr.get_exprs()?;
                    let expr = if left.get_position() == origin.get_position() {
                        right
                    } else {
                        left
                    };

                    if let LuaExpr::LiteralExpr(literal) = expr {
                        let type_assert = match literal.get_literal()? {
                            LuaLiteralToken::Nil(_) => TypeAssertion::Exist,
                            LuaLiteralToken::Bool(b) => {
                                if b.is_true() {
                                    TypeAssertion::NotExist
                                } else {
                                    TypeAssertion::Exist
                                }
                            }
                            LuaLiteralToken::Number(i) => {
                                if i.is_int() {
                                    TypeAssertion::Remove(LuaType::IntegerConst(i.get_int_value()))
                                } else {
                                    TypeAssertion::Remove(LuaType::Number)
                                }
                            }
                            LuaLiteralToken::String(s) => TypeAssertion::Remove(
                                LuaType::StringConst(SmolStr::new(s.get_value()).into()),
                            ),
                            _ => return None,
                        };

                        broadcast_up(
                            db,
                            var_trace,
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
            infer_call_arg_list(db, var_trace, type_assert, call_args_list)?;
        }
        LuaAst::LuaUnaryExpr(unary_expr) => {
            let op = unary_expr.get_op_token()?;
            match op.get_op() {
                UnaryOperator::OpNot => {
                    if let Some(ne_type_assert) = type_assert.get_negation() {
                        broadcast_up(
                            db,
                            var_trace,
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
