use crate::{
    grammar::ParseResult,
    kind::{BinaryOperator, LuaOpKind, LuaSyntaxKind, LuaTokenKind, UnaryOperator, UNARY_PRIORITY},
    parser::{LuaParser, MarkerEventContainer},
    parser_error::LuaParseError,
    SpecialFunction,
};

use super::{expect_token, if_token_bump, parse_block};

pub fn parse_expr(p: &mut LuaParser) -> ParseResult {
    return parse_sub_expr(p, 0);
}

fn parse_sub_expr(p: &mut LuaParser, limit: i32) -> ParseResult {
    let uop = LuaOpKind::to_unary_operator(p.current_token());
    let mut cm = if uop != UnaryOperator::OpNop {
        let m = p.mark(LuaSyntaxKind::UnaryExpr);
        let range = p.current_token_range();
        p.bump();
        match parse_sub_expr(p, UNARY_PRIORITY) {
            Ok(_) => {}
            Err(err) => {
                p.push_error(LuaParseError::from_source_range(
                    &t!("unary operator not followed by expression"),
                    range,
                ));
                return Err(err);
            }
        }
        m.complete(p)
    } else {
        parse_simple_expr(p)?
    };

    let mut bop = LuaOpKind::to_binary_operator(p.current_token());
    while bop != BinaryOperator::OpNop && bop.get_priority().left > limit {
        let range = p.current_token_range();
        let m = cm.precede(p, LuaSyntaxKind::BinaryExpr);
        p.bump();
        match parse_sub_expr(p, bop.get_priority().right) {
            Ok(_) => {}
            Err(err) => {
                p.push_error(LuaParseError::from_source_range(
                    &t!("binary operator not followed by expression"),
                    range,
                ));

                return Err(err);
            }
        }

        cm = m.complete(p);
        bop = LuaOpKind::to_binary_operator(p.current_token());
    }

    Ok(cm)
}

fn parse_simple_expr(p: &mut LuaParser) -> ParseResult {
    match p.current_token() {
        LuaTokenKind::TkInt
        | LuaTokenKind::TkFloat
        | LuaTokenKind::TkComplex
        | LuaTokenKind::TkNil
        | LuaTokenKind::TkTrue
        | LuaTokenKind::TkFalse
        | LuaTokenKind::TkDots
        | LuaTokenKind::TkString
        | LuaTokenKind::TkLongString => {
            let m = p.mark(LuaSyntaxKind::LiteralExpr);
            p.bump();
            Ok(m.complete(p))
        }
        LuaTokenKind::TkLeftBrace => parse_table_expr(p),
        LuaTokenKind::TkFunction => parse_closure_expr(p),
        _ => parse_suffixed_expr(p),
    }
}

pub fn parse_closure_expr(p: &mut LuaParser) -> ParseResult {
    let m = p.mark(LuaSyntaxKind::ClosureExpr);

    if_token_bump(p, LuaTokenKind::TkFunction);
    parse_param_list(p)?;

    if p.current_token() != LuaTokenKind::TkEnd {
        parse_block(p)?;
    }

    expect_token(p, LuaTokenKind::TkEnd)?;
    Ok(m.complete(p))
}

fn parse_param_list(p: &mut LuaParser) -> ParseResult {
    let m = p.mark(LuaSyntaxKind::ParamList);

    expect_token(p, LuaTokenKind::TkLeftParen)?;
    if p.current_token() != LuaTokenKind::TkRightParen {
        parse_param_name(p)?;
        while p.current_token() == LuaTokenKind::TkComma {
            p.bump();
            parse_param_name(p)?;
        }
    }

    expect_token(p, LuaTokenKind::TkRightParen)?;
    Ok(m.complete(p))
}

fn parse_param_name(p: &mut LuaParser) -> ParseResult {
    let m = p.mark(LuaSyntaxKind::ParamName);

    if p.current_token() == LuaTokenKind::TkName || p.current_token() == LuaTokenKind::TkDots {
        p.bump();
    } else {
        return Err(LuaParseError::from_source_range(
            &t!("expect parameter name"),
            p.current_token_range(),
        ));
    }

    Ok(m.complete(p))
}

fn parse_table_expr(p: &mut LuaParser) -> ParseResult {
    let mut m = p.mark(LuaSyntaxKind::TableEmptyExpr);
    p.bump();

    if p.current_token() == LuaTokenKind::TkRightBrace {
        p.bump();
        return Ok(m.complete(p));
    }

    let mut cm = parse_field(p)?;
    match cm.kind {
        LuaSyntaxKind::TableFieldAssign => {
            m.set_kind(p, LuaSyntaxKind::TableObjectExpr);
        }
        LuaSyntaxKind::TableFieldValue => {
            m.set_kind(p, LuaSyntaxKind::TableArrayExpr);
        }
        _ => {}
    }

    while p.current_token() == LuaTokenKind::TkComma
        || p.current_token() == LuaTokenKind::TkSemicolon
    {
        p.bump();
        if p.current_token() == LuaTokenKind::TkRightBrace {
            break;
        }
        cm = parse_field(p)?;
        if cm.kind == LuaSyntaxKind::TableFieldAssign {
            m.set_kind(p, LuaSyntaxKind::TableObjectExpr);
        }
    }

    expect_token(p, LuaTokenKind::TkRightBrace)?;
    Ok(m.complete(p))
}

fn parse_field(p: &mut LuaParser) -> ParseResult {
    let mut m = p.mark(LuaSyntaxKind::TableFieldValue);

    if p.current_token() == LuaTokenKind::TkLeftBracket {
        m.set_kind(p, LuaSyntaxKind::TableFieldAssign);
        p.bump();
        parse_expr(p)?;
        expect_token(p, LuaTokenKind::TkRightBracket)?;
        expect_token(p, LuaTokenKind::TkAssign)?;
        parse_expr(p)?;
    } else if p.current_token() == LuaTokenKind::TkName {
        if p.peek_next_token() == LuaTokenKind::TkAssign {
            m.set_kind(p, LuaSyntaxKind::TableFieldAssign);
            p.bump();
            p.bump();
            parse_expr(p)?;
        } else {
            parse_expr(p)?;
        }
    } else {
        parse_expr(p)?;
    }

    Ok(m.complete(p))
}

fn parse_suffixed_expr(p: &mut LuaParser) -> ParseResult {
    let mut cm = match p.current_token() {
        LuaTokenKind::TkName => parse_name_or_special_function(p)?,
        LuaTokenKind::TkLeftParen => {
            let m = p.mark(LuaSyntaxKind::ParenExpr);
            p.bump();
            parse_expr(p)?;
            expect_token(p, LuaTokenKind::TkRightParen)?;
            m.complete(p)
        }
        _ => {
            return Err(LuaParseError::from_source_range(
                &t!("expect primary expression"),
                p.current_token_range(),
            ))
        }
    };

    loop {
        match p.current_token() {
            LuaTokenKind::TkDot | LuaTokenKind::TkColon | LuaTokenKind::TkLeftBracket => {
                let m = cm.precede(p, LuaSyntaxKind::IndexExpr);
                parse_index_struct(p)?;
                cm = m.complete(p);
            }
            LuaTokenKind::TkLeftParen
            | LuaTokenKind::TkLongString
            | LuaTokenKind::TkString
            | LuaTokenKind::TkLeftBrace => {
                let m = cm.precede(p, LuaSyntaxKind::CallExpr);
                parse_args(p)?;
                cm = m.complete(p);
            }
            _ => {
                return Ok(cm);
            }
        }
    }
}

fn parse_name_or_special_function(p: &mut LuaParser) -> ParseResult {
    let m = p.mark(LuaSyntaxKind::NameExpr);
    let special_kind = match p.parse_config.get_special_function(p.current_token_text()) {
        SpecialFunction::Require => LuaSyntaxKind::RequireCallExpr,
        SpecialFunction::Assert => LuaSyntaxKind::AssertCallExpr,
        SpecialFunction::Error => LuaSyntaxKind::ErrorCallExpr,
        SpecialFunction::Type => LuaSyntaxKind::TypeCallExpr,
        SpecialFunction::Setmatable => LuaSyntaxKind::SetmetatableCallExpr,
        _ => LuaSyntaxKind::None,
    };
    p.bump();
    let mut cm = m.complete(p);
    if special_kind == LuaSyntaxKind::None {
        return Ok(cm);
    }

    if matches!(
        p.current_token(),
        LuaTokenKind::TkLeftParen
            | LuaTokenKind::TkLongString
            | LuaTokenKind::TkString
            | LuaTokenKind::TkLeftBrace
    ) {
        let m1 = cm.precede(p, special_kind);
        parse_args(p)?;
        cm = m1.complete(p);
    }

    Ok(cm)
}

fn parse_index_struct(p: &mut LuaParser) -> Result<(), LuaParseError> {
    match p.current_token() {
        LuaTokenKind::TkLeftBracket => {
            p.bump();
            parse_expr(p)?;
            expect_token(p, LuaTokenKind::TkRightBracket)?;
        }
        LuaTokenKind::TkDot => {
            p.bump();
            expect_token(p, LuaTokenKind::TkName)?;
        }
        LuaTokenKind::TkColon => {
            p.bump();
            expect_token(p, LuaTokenKind::TkName)?;
            if !matches!(
                p.current_token(),
                LuaTokenKind::TkLeftParen
                    | LuaTokenKind::TkLeftBrace
                    | LuaTokenKind::TkString
                    | LuaTokenKind::TkLongString
            ) {
                return Err(LuaParseError::from_source_range(
                    &t!("colon accessor must be followed by a function call or table constructor or string literal"),
                    p.current_token_range(),
                ));
            }
        }
        _ => {
            return Err(LuaParseError::from_source_range(
                &t!("expect index struct"),
                p.current_token_range(),
            ));
        }
    }

    Ok(())
}

fn parse_args(p: &mut LuaParser) -> ParseResult {
    let m = p.mark(LuaSyntaxKind::CallArgList);
    match p.current_token() {
        LuaTokenKind::TkLeftParen => {
            p.bump();
            if p.current_token() != LuaTokenKind::TkRightParen {
                parse_expr(p)?;
                while p.current_token() == LuaTokenKind::TkComma {
                    p.bump();
                    if p.current_token() == LuaTokenKind::TkRightParen {
                        p.push_error(LuaParseError::from_source_range(
                            &t!("expect expression"),
                            p.current_token_range(),
                        ));
                        break;
                    }
                    parse_expr(p)?;
                }
            }
            expect_token(p, LuaTokenKind::TkRightParen)?;
        }
        LuaTokenKind::TkLeftBrace => {
            parse_table_expr(p)?;
        }
        LuaTokenKind::TkString | LuaTokenKind::TkLongString => {
            let m1 = p.mark(LuaSyntaxKind::LiteralExpr);
            p.bump();
            m1.complete(p);
        }
        _ => {
            return Err(LuaParseError::from_source_range(
                &t!("expect args"),
                p.current_token_range(),
            ));
        }
    }

    Ok(m.complete(p))
}
