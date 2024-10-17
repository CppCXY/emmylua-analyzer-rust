use crate::{
    grammar::ParseResult,
    kind::{LuaSyntaxKind, LuaTokenKind},
    lexer::LuaDocLexerState,
    parser::{CompleteMarker, LuaDocParser, MarkerEventContainer},
    parser_error::LuaParseError,
};

use super::{
    expect_token, if_token_bump, parse_description,
    types::{parse_fun_type, parse_type, parse_type_list},
};

pub fn parse_tag(p: &mut LuaDocParser) {
    match parse_tag_detail(p) {
        Ok(_) => {}
        Err(error) => {
            p.push_error(error);
        }
    }
}

pub fn parse_long_tag(p: &mut LuaDocParser) {
    parse_tag(p);
}

fn parse_tag_detail(p: &mut LuaDocParser) -> ParseResult {
    match p.current_token() {
        // main tag
        LuaTokenKind::TkTagClass | LuaTokenKind::TkTagInterface => parse_tag_class(p),
        LuaTokenKind::TkTagEnum => parse_tag_enum(p),
        LuaTokenKind::TkTagAlias => parse_tag_alias(p),
        LuaTokenKind::TkTagField => parse_tag_field(p),
        LuaTokenKind::TkTagType => parse_tag_type(p),
        LuaTokenKind::TkTagParam => parse_tag_param(p),
        LuaTokenKind::TkTagReturn => parse_tag_return(p),
        // other tag
        LuaTokenKind::TkTagModule => parse_tag_module(p),
        LuaTokenKind::TkTagSee => parse_tag_see(p),
        LuaTokenKind::TkTagGeneric => parse_tag_generic(p),
        LuaTokenKind::TkTagAs => parse_tag_as(p),
        LuaTokenKind::TkTagOverload => parse_tag_overload(p),
        LuaTokenKind::TkTagCast => parse_tag_cast(p),
        LuaTokenKind::TkTagSource => parse_tag_source(p),
        LuaTokenKind::TkTagDiagnostic => parse_tag_diagnostic(p),
        LuaTokenKind::TkTagVersion => parse_tag_version(p),
        LuaTokenKind::TkTagOperator => parse_tag_operator(p),
        LuaTokenKind::TkTagMapping => parse_tag_mapping(p),
        LuaTokenKind::TkTagNamespace => parse_tag_namespace(p),
        LuaTokenKind::TkTagUsing => parse_tag_using(p),

        // simple tag
        LuaTokenKind::TkTagVisibility => parse_tag_simple(p, LuaSyntaxKind::DocTagVisibility),
        LuaTokenKind::TkTagReadonly => parse_tag_simple(p, LuaSyntaxKind::DocTagReadonly),
        LuaTokenKind::TkTagDeprecated => parse_tag_simple(p, LuaSyntaxKind::DocTagDeprecated),
        LuaTokenKind::TkTagAsync => parse_tag_simple(p, LuaSyntaxKind::DocTagAsync),
        LuaTokenKind::TkTagNodiscard => parse_tag_simple(p, LuaSyntaxKind::DocTagNodiscard),
        LuaTokenKind::TkTagMeta => parse_tag_simple(p, LuaSyntaxKind::DocTagMeta),
        LuaTokenKind::TkTagOther => parse_tag_simple(p, LuaSyntaxKind::DocTagOther),
        _ => Ok(CompleteMarker::empty()),
    }
}

fn parse_tag_simple(p: &mut LuaDocParser, kind: LuaSyntaxKind) -> ParseResult {
    p.set_state(LuaDocLexerState::Description);
    let m = p.mark(kind);

    parse_description(p);

    Ok(m.complete(p))
}

// ---@class <class name>
fn parse_tag_class(p: &mut LuaDocParser) -> ParseResult {
    p.set_state(LuaDocLexerState::Normal);
    let m = p.mark(LuaSyntaxKind::DocTagClass);
    if p.current_token() == LuaTokenKind::TkLeftParen {
        parse_tag_attribute(p)?;
    }

    expect_token(p, LuaTokenKind::TkName)?;
    // TODO suffixed
    if p.current_token() == LuaTokenKind::TkLt {
        parse_generic_decl_list(p)?;
    }

    if p.current_token() == LuaTokenKind::TkColon {
        p.bump();
        parse_type_list(p)?;
    }

    p.set_state(LuaDocLexerState::Description);
    parse_description(p);
    Ok(m.complete(p))
}

// (partial, global, local)
fn parse_tag_attribute(p: &mut LuaDocParser) -> ParseResult {
    let m = p.mark(LuaSyntaxKind::DocAttribute);
    p.bump();
    expect_token(p, LuaTokenKind::TkName)?;
    while p.current_token() == LuaTokenKind::TkComma {
        p.bump();
        expect_token(p, LuaTokenKind::TkName)?;
    }

    expect_token(p, LuaTokenKind::TkRightParen)?;
    Ok(m.complete(p))
}

// <T, R, C: AAA>
fn parse_generic_decl_list(p: &mut LuaDocParser) -> ParseResult {
    let m = p.mark(LuaSyntaxKind::DocGenericDeclareList);
    expect_token(p, LuaTokenKind::TkLt)?;
    parse_generic_param(p)?;
    while p.current_token() == LuaTokenKind::TkComma {
        p.bump();
        parse_generic_param(p)?;
    }
    expect_token(p, LuaTokenKind::TkGt)?;
    Ok(m.complete(p))
}

// A : type
// A
fn parse_generic_param(p: &mut LuaDocParser) -> ParseResult {
    let m = p.mark(LuaSyntaxKind::DocGenericParameter);
    expect_token(p, LuaTokenKind::TkName)?;
    if p.current_token() == LuaTokenKind::TkColon {
        p.bump();
        parse_type(p)?;
    }
    Ok(m.complete(p))
}

// ---@enum A
// ---@enum A : number
fn parse_tag_enum(p: &mut LuaDocParser) -> ParseResult {
    p.set_state(LuaDocLexerState::Normal);
    let m = p.mark(LuaSyntaxKind::DocTagEnum);
    if p.current_token() == LuaTokenKind::TkLeftParen {
        parse_tag_attribute(p)?;
    }

    expect_token(p, LuaTokenKind::TkName)?;
    if p.current_token() == LuaTokenKind::TkColon {
        p.bump();
        parse_type(p)?;
    }
    p.set_state(LuaDocLexerState::Description);
    parse_description(p);
    Ok(m.complete(p))
}

// ---@alias A string
// ---@alias A<T> keyof T
fn parse_tag_alias(p: &mut LuaDocParser) -> ParseResult {
    p.set_state(LuaDocLexerState::Normal);
    let m = p.mark(LuaSyntaxKind::DocTagAlias);
    expect_token(p, LuaTokenKind::TkName)?;
    if p.current_token() == LuaTokenKind::TkLt {
        parse_generic_decl_list(p)?;
    }

    parse_type(p)?;

    p.set_state(LuaDocLexerState::Description);
    parse_description(p);
    Ok(m.complete(p))
}

// ---@module "aaa.bbb.ccc" force require path to be "aaa.bbb.ccc"
// ---@module no-require
fn parse_tag_module(p: &mut LuaDocParser) -> ParseResult {
    p.set_state(LuaDocLexerState::Normal);
    let m = p.mark(LuaSyntaxKind::DocTagModule);
    if p.current_token() == LuaTokenKind::TkName || p.current_token() == LuaTokenKind::TkString {
        p.bump();
    }

    p.set_state(LuaDocLexerState::Description);
    parse_description(p);
    Ok(m.complete(p))
}

// ---@field aaa string
// ---@field aaa? number
// ---@field [string] number
// ---@field [1] number
fn parse_tag_field(p: &mut LuaDocParser) -> ParseResult {
    p.set_state(LuaDocLexerState::FieldStart);
    let m = p.mark(LuaSyntaxKind::DocTagField);
    if p.current_token() == LuaTokenKind::TkLeftParen {
        parse_tag_attribute(p)?;
    }

    p.set_state(LuaDocLexerState::Normal);
    if_token_bump(p, LuaTokenKind::TkDocVisibility);
    match p.current_token() {
        LuaTokenKind::TkName => p.bump(),
        LuaTokenKind::TkLeftBracket => {
            p.bump();
            if p.current_token() == LuaTokenKind::TkInt
                || p.current_token() == LuaTokenKind::TkString
            {
                p.bump();
            } else {
                parse_type(p)?;
            }
            expect_token(p, LuaTokenKind::TkRightBracket)?;
        }
        _ => {
            return Err(LuaParseError::from_source_range(
                &format!("expect field name or '[', but get {:?}", p.current_token()),
                p.current_token_range(),
            ))
        }
    }
    if_token_bump(p, LuaTokenKind::TkDocQuestion);
    parse_type(p)?;

    p.set_state(LuaDocLexerState::Description);
    parse_description(p);
    Ok(m.complete(p))
}

// ---@type string
// ---@type number, string
fn parse_tag_type(p: &mut LuaDocParser) -> ParseResult {
    p.set_state(LuaDocLexerState::Normal);
    let m = p.mark(LuaSyntaxKind::DocTagType);
    parse_type(p)?;
    while p.current_token() == LuaTokenKind::TkComma {
        p.bump();
        parse_type(p)?;
    }

    p.set_state(LuaDocLexerState::Description);
    parse_description(p);
    Ok(m.complete(p))
}

// ---@param a number
// ---@param a? number
// ---@param ... string
fn parse_tag_param(p: &mut LuaDocParser) -> ParseResult {
    p.set_state(LuaDocLexerState::Normal);
    let m = p.mark(LuaSyntaxKind::DocTagParam);

    if matches!(
        p.current_token(),
        LuaTokenKind::TkName | LuaTokenKind::TkDots
    ) {
        p.bump();
    }

    if_token_bump(p, LuaTokenKind::TkDocQuestion);
    // compact luals
    if_token_bump(p, LuaTokenKind::TkDocContinueOr);
    parse_type(p)?;

    p.set_state(LuaDocLexerState::Description);
    parse_description(p);
    Ok(m.complete(p))
}

// ---@return number
// ---@return number, string
// ---@return number <name> , this just compact luals
fn parse_tag_return(p: &mut LuaDocParser) -> ParseResult {
    p.set_state(LuaDocLexerState::Normal);
    let m = p.mark(LuaSyntaxKind::DocTagReturn);
    parse_type(p)?;
    if_token_bump(p, LuaTokenKind::TkName);

    while p.current_token() == LuaTokenKind::TkComma {
        p.bump();
        parse_type(p)?;
        if_token_bump(p, LuaTokenKind::TkName);
    }

    p.set_state(LuaDocLexerState::Description);
    parse_description(p);
    Ok(m.complete(p))
}

// ---@generic T
// ---@generic T, R
// ---@generic T, R : number
fn parse_tag_generic(p: &mut LuaDocParser) -> ParseResult {
    p.set_state(LuaDocLexerState::Normal);
    let m = p.mark(LuaSyntaxKind::DocTagGeneric);
    expect_token(p, LuaTokenKind::TkName)?;
    if p.current_token() == LuaTokenKind::TkLt {
        parse_generic_decl_list(p)?;
    }

    p.set_state(LuaDocLexerState::Description);
    parse_description(p);
    Ok(m.complete(p))
}

// ---@see <name>
// ---@see <name>#<name>
fn parse_tag_see(p: &mut LuaDocParser) -> ParseResult {
    p.set_state(LuaDocLexerState::See);
    let m = p.mark(LuaSyntaxKind::DocTagSee);
    expect_token(p, LuaTokenKind::TkName)?;
    while p.current_token() == LuaTokenKind::TkLen {
        p.bump();
        expect_token(p, LuaTokenKind::TkName)?;
    }

    p.set_state(LuaDocLexerState::Description);
    parse_description(p);
    Ok(m.complete(p))
}

// ---@as number
// --[[@as number]]
fn parse_tag_as(p: &mut LuaDocParser) -> ParseResult {
    p.set_state(LuaDocLexerState::Normal);
    let m = p.mark(LuaSyntaxKind::DocTagAs);
    parse_type(p)?;
    p.set_state(LuaDocLexerState::Description);
    parse_description(p);
    Ok(m.complete(p))
}

// ---@overload fun(a: number): string
// ---@overload async fun(a: number): string
fn parse_tag_overload(p: &mut LuaDocParser) -> ParseResult {
    p.set_state(LuaDocLexerState::Normal);
    let m = p.mark(LuaSyntaxKind::DocTagOverload);
    parse_fun_type(p)?;
    p.set_state(LuaDocLexerState::Description);
    parse_description(p);
    Ok(m.complete(p))
}

// ---@cast a number
// ---@cast a +string
// ---@cast a -string
// ---@cast a +?
// ---@cast a +string, -number
fn parse_tag_cast(p: &mut LuaDocParser) -> ParseResult {
    p.set_state(LuaDocLexerState::Normal);
    let m = p.mark(LuaSyntaxKind::DocTagCast);
    expect_token(p, LuaTokenKind::TkName)?;

    parse_op_type(p)?;
    while p.current_token() == LuaTokenKind::TkComma {
        p.bump();
        parse_op_type(p)?;
    }

    p.set_state(LuaDocLexerState::Description);
    parse_description(p);
    Ok(m.complete(p))
}

// +<type>, -<type>, +?, <type>
fn parse_op_type(p: &mut LuaDocParser) -> ParseResult {
    p.set_state(LuaDocLexerState::Normal);
    let m = p.mark(LuaSyntaxKind::DocOpType);
    if p.current_token() == LuaTokenKind::TkPlus || p.current_token() == LuaTokenKind::TkMinus {
        p.bump();
        if p.current_token() == LuaTokenKind::TkDocQuestion {
            p.bump();
        } else {
            parse_type(p)?;
        }
    } else {
        parse_type(p)?;
    }

    Ok(m.complete(p))
}

// ---@source <path>
// ---@source "<path>"
fn parse_tag_source(p: &mut LuaDocParser) -> ParseResult {
    p.set_state(LuaDocLexerState::Source);

    let m = p.mark(LuaSyntaxKind::DocTagSource);
    expect_token(p, LuaTokenKind::TKDocPath)?;

    Ok(m.complete(p))
}

// ---@diagnostic <action>: <diagnostic-code>, ...
fn parse_tag_diagnostic(p: &mut LuaDocParser) -> ParseResult {
    p.set_state(LuaDocLexerState::Normal);
    let m = p.mark(LuaSyntaxKind::DocTagDiagnostic);
    expect_token(p, LuaTokenKind::TkName)?;
    if p.current_token() == LuaTokenKind::TkColon {
        p.bump();

        expect_token(p, LuaTokenKind::TkName)?;
        while p.current_token() == LuaTokenKind::TkComma {
            p.bump();
            expect_token(p, LuaTokenKind::TkName)?;
        }
    }

    Ok(m.complete(p))
}

// ---@version Lua 5.1
// ---@version Lua JIT
// ---@version 5.1, JIT
// ---@version > Lua 5.1, Lua JIT
// ---@version > 5.1, 5.2, 5.3
fn parse_tag_version(p: &mut LuaDocParser) -> ParseResult {
    p.set_state(LuaDocLexerState::Version);
    let m = p.mark(LuaSyntaxKind::DocTagVersion);
    parse_version(p)?;
    while p.current_token() == LuaTokenKind::TkComma {
        p.bump();
        parse_version(p)?;
    }

    Ok(m.complete(p))
}

// 5.1
// JIT
// > 5.1
// < 5.4
// > Lua 5.1
fn parse_version(p: &mut LuaDocParser) -> ParseResult {
    let m = p.mark(LuaSyntaxKind::DocVersion);
    if matches!(p.current_token(), LuaTokenKind::TkLt | LuaTokenKind::TkGt) {
        p.bump();
    }

    if p.current_token() == LuaTokenKind::TkName {
        p.bump();
    }

    expect_token(p, LuaTokenKind::TkVersionNumber)?;
    Ok(m.complete(p))
}

// ---@operator add(number): number
fn parse_tag_operator(p: &mut LuaDocParser) -> ParseResult {
    p.set_state(LuaDocLexerState::Normal);
    let m = p.mark(LuaSyntaxKind::DocTagOperator);
    expect_token(p, LuaTokenKind::TkName)?;
    if p.current_token() == LuaTokenKind::TkLeftParen {
        p.bump();
        parse_type_list(p)?;
    }
    expect_token(p, LuaTokenKind::TkRightParen)?;

    if p.current_token() == LuaTokenKind::TkColon {
        p.bump();
        parse_type(p)?;
    }

    p.set_state(LuaDocLexerState::Description);
    parse_description(p);
    Ok(m.complete(p))
}

// ---@mapping <new name>
fn parse_tag_mapping(p: &mut LuaDocParser) -> ParseResult {
    p.set_state(LuaDocLexerState::Normal);
    let m = p.mark(LuaSyntaxKind::DocTagMapping);
    expect_token(p, LuaTokenKind::TkName)?;
    p.set_state(LuaDocLexerState::Description);
    parse_description(p);
    Ok(m.complete(p))
}

// ---@namespace path
// ---@namespace System.Net
fn parse_tag_namespace(p: &mut LuaDocParser) -> ParseResult {
    p.set_state(LuaDocLexerState::Namespace);
    let m = p.mark(LuaSyntaxKind::DocTagNamespace);
    expect_token(p, LuaTokenKind::TkName)?;
    Ok(m.complete(p))
}

// ---@using path
fn parse_tag_using(p: &mut LuaDocParser) -> ParseResult {
    p.set_state(LuaDocLexerState::Namespace);
    let m = p.mark(LuaSyntaxKind::DocTagUsing);
    expect_token(p, LuaTokenKind::TkName)?;
    Ok(m.complete(p))
}
