use crate::{kind::LuaSyntaxKind, syntax::traits::LuaAstNode, LuaAstChildren, LuaAstToken, LuaAstTokenChildren, LuaDocVisibilityToken, LuaNameToken, LuaNumberToken, LuaStringToken, LuaSyntaxNode, LuaTokenKind};

use super::{description::{LuaDocDescriptionOwner, LuaDocDetailOwner}, LuaDocGenericDeclList, LuaDocType, LuaDocTypeList};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LuaDocTag {
    Class(LuaDocTagClass),
    Enum(LuaDocTagEnum),
    Alias(LuaDocTagAlias),
    Type(LuaDocTagType),
    Param(LuaDocTagParam),
    Return(LuaDocTagReturn),
    Overload(LuaDocTagOverload),
    Field(LuaDocTagField),
    Module(LuaDocTagModule),
    See(LuaDocTagSee),
    Diagnostic(LuaDocTagDiagnostic),
    Deprecated(LuaDocTagDeprecated),
    Version(LuaDocTagVersion),
    Cast(LuaDocTagCast),
    Source(LuaDocTagSource),
    Other(LuaDocTagOther),
    Namespace(LuaDocTagNamespace),
    Using(LuaDocTagUsing),
    Meta(LuaDocTagMeta),
}

impl LuaAstNode for LuaDocTag {
    fn syntax(&self) -> &LuaSyntaxNode {
        match self {
            LuaDocTag::Class(it) => it.syntax(),
            LuaDocTag::Enum(it) => it.syntax(),
            LuaDocTag::Alias(it) => it.syntax(),
            LuaDocTag::Type(it) => it.syntax(),
            LuaDocTag::Param(it) => it.syntax(),
            LuaDocTag::Return(it) => it.syntax(),
            LuaDocTag::Overload(it) => it.syntax(),
            LuaDocTag::Field(it) => it.syntax(),
            LuaDocTag::Module(it) => it.syntax(),
            LuaDocTag::See(it) => it.syntax(),
            LuaDocTag::Diagnostic(it) => it.syntax(),
            LuaDocTag::Deprecated(it) => it.syntax(),
            LuaDocTag::Version(it) => it.syntax(),
            LuaDocTag::Cast(it) => it.syntax(),
            LuaDocTag::Source(it) => it.syntax(),
            LuaDocTag::Other(it) => it.syntax(),
            LuaDocTag::Namespace(it) => it.syntax(),
            LuaDocTag::Using(it) => it.syntax(),
            LuaDocTag::Meta(it) => it.syntax(),
        }
    }

    fn can_cast(kind: LuaSyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == LuaSyntaxKind::DocTagClass
            || kind == LuaSyntaxKind::DocTagEnum
            || kind == LuaSyntaxKind::DocTagAlias
            || kind == LuaSyntaxKind::DocTagType
            || kind == LuaSyntaxKind::DocTagParam
            || kind == LuaSyntaxKind::DocTagReturn
            || kind == LuaSyntaxKind::DocTagOverload
            || kind == LuaSyntaxKind::DocTagField
            || kind == LuaSyntaxKind::DocTagModule
            || kind == LuaSyntaxKind::DocTagSee
            || kind == LuaSyntaxKind::DocTagDiagnostic
            || kind == LuaSyntaxKind::DocTagDeprecated
            || kind == LuaSyntaxKind::DocTagVersion
            || kind == LuaSyntaxKind::DocTagCast
            || kind == LuaSyntaxKind::DocTagSource
            || kind == LuaSyntaxKind::DocTagOther
            || kind == LuaSyntaxKind::DocTagNamespace
            || kind == LuaSyntaxKind::DocTagUsing
            || kind == LuaSyntaxKind::DocTagMeta
    }

    fn cast(syntax: LuaSyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        match syntax.kind().into() {
            LuaSyntaxKind::DocTagClass => Some(LuaDocTag::Class(LuaDocTagClass::cast(syntax).unwrap())),
            LuaSyntaxKind::DocTagEnum => Some(LuaDocTag::Enum(LuaDocTagEnum::cast(syntax).unwrap())),
            LuaSyntaxKind::DocTagAlias => Some(LuaDocTag::Alias(LuaDocTagAlias::cast(syntax).unwrap())),
            LuaSyntaxKind::DocTagType => Some(LuaDocTag::Type(LuaDocTagType::cast(syntax).unwrap())),
            LuaSyntaxKind::DocTagParam => Some(LuaDocTag::Param(LuaDocTagParam::cast(syntax).unwrap())),
            LuaSyntaxKind::DocTagReturn => Some(LuaDocTag::Return(LuaDocTagReturn::cast(syntax).unwrap())),
            LuaSyntaxKind::DocTagOverload => Some(LuaDocTag::Overload(LuaDocTagOverload::cast(syntax).unwrap())),
            LuaSyntaxKind::DocTagField => Some(LuaDocTag::Field(LuaDocTagField::cast(syntax).unwrap())),
            LuaSyntaxKind::DocTagModule => Some(LuaDocTag::Module(LuaDocTagModule::cast(syntax).unwrap())),
            LuaSyntaxKind::DocTagSee => Some(LuaDocTag::See(LuaDocTagSee::cast(syntax).unwrap())),
            LuaSyntaxKind::DocTagDiagnostic => Some(LuaDocTag::Diagnostic(LuaDocTagDiagnostic::cast(syntax).unwrap())),
            LuaSyntaxKind::DocTagDeprecated => Some(LuaDocTag::Deprecated(LuaDocTagDeprecated::cast(syntax).unwrap())),
            LuaSyntaxKind::DocTagVersion => Some(LuaDocTag::Version(LuaDocTagVersion::cast(syntax).unwrap())),
            LuaSyntaxKind::DocTagCast => Some(LuaDocTag::Cast(LuaDocTagCast::cast(syntax).unwrap())),
            LuaSyntaxKind::DocTagSource => Some(LuaDocTag::Source(LuaDocTagSource::cast(syntax).unwrap())),
            LuaSyntaxKind::DocTagOther => Some(LuaDocTag::Other(LuaDocTagOther::cast(syntax).unwrap())),
            LuaSyntaxKind::DocTagNamespace => Some(LuaDocTag::Namespace(LuaDocTagNamespace::cast(syntax).unwrap())),
            LuaSyntaxKind::DocTagUsing => Some(LuaDocTag::Using(LuaDocTagUsing::cast(syntax).unwrap())),
            LuaSyntaxKind::DocTagMeta => Some(LuaDocTag::Meta(LuaDocTagMeta::cast(syntax).unwrap())),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LuaDocTagClass {
    syntax: LuaSyntaxNode,
}

impl LuaAstNode for LuaDocTagClass {
    fn syntax(&self) -> &LuaSyntaxNode {
        &self.syntax
    }

    fn can_cast(kind: LuaSyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == LuaSyntaxKind::DocTagClass
    }

    fn cast(syntax: LuaSyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        if syntax.kind() == LuaSyntaxKind::DocTagClass.into() {
            Some(Self { syntax })
        } else {
            None
        }
    }
}

impl LuaDocDescriptionOwner for LuaDocTagClass {}

impl LuaDocTagClass {
    pub fn get_name(&self) -> Option<LuaNameToken> {
        self.token()
    }

    pub fn get_generic_decl(&self) -> Option<LuaDocGenericDeclList> {
        self.child()
    }

    pub fn get_supers(&self) -> Option<LuaDocTypeList> {
        self.child()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LuaDocTagEnum {
    syntax: LuaSyntaxNode,
}

impl LuaAstNode for LuaDocTagEnum {
    fn syntax(&self) -> &LuaSyntaxNode {
        &self.syntax
    }

    fn can_cast(kind: LuaSyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == LuaSyntaxKind::DocTagEnum
    }

    fn cast(syntax: LuaSyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        if syntax.kind() == LuaSyntaxKind::DocTagEnum.into() {
            Some(Self { syntax })
        } else {
            None
        }
    }
}

impl LuaDocDescriptionOwner for LuaDocTagEnum {}

impl LuaDocTagEnum {
    pub fn get_name(&self) -> Option<LuaNameToken> {
        self.token()
    }

    pub fn get_base_type(&self) -> Option<LuaDocType> {
        self.child()
    }

    pub fn get_fields(&self) -> Option<LuaDocEnumField> {
        self.child()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LuaDocEnumField {
    syntax: LuaSyntaxNode,
}

impl LuaAstNode for LuaDocEnumField {
    fn syntax(&self) -> &LuaSyntaxNode {
        &self.syntax
    }

    fn can_cast(kind: LuaSyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == LuaSyntaxKind::DocEnumField
    }

    fn cast(syntax: LuaSyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        if syntax.kind() == LuaSyntaxKind::DocEnumField.into() {
            Some(Self { syntax })
        } else {
            None
        }
    }
}

impl LuaDocDetailOwner for LuaDocEnumField {}

impl LuaDocEnumField {
    pub fn get_name(&self) -> Option<LuaNameToken> {
        self.token()
    }

    pub fn get_type(&self) -> Option<LuaDocType> {
        self.child()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LuaDocTagAlias {
    syntax: LuaSyntaxNode,
}

impl LuaAstNode for LuaDocTagAlias {
    fn syntax(&self) -> &LuaSyntaxNode {
        &self.syntax
    }

    fn can_cast(kind: LuaSyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == LuaSyntaxKind::DocTagAlias
    }

    fn cast(syntax: LuaSyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        if syntax.kind() == LuaSyntaxKind::DocTagAlias.into() {
            Some(Self { syntax })
        } else {
            None
        }
    }
}

impl LuaDocDescriptionOwner for LuaDocTagAlias {}

impl LuaDocTagAlias {
    pub fn get_name(&self) -> Option<LuaNameToken> {
        self.token()
    }

    pub fn get_generic_decl_list(&self) -> Option<LuaDocGenericDeclList> {
        self.child()
    }

    pub fn get_type(&self) -> Option<LuaDocType> {
        self.child()
    }

    pub fn get_alias_fields(&self) -> Option<LuaDocAliasFieldList> {
        self.child()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LuaDocAliasFieldList {
    syntax: LuaSyntaxNode,
}

impl LuaAstNode for LuaDocAliasFieldList {
    fn syntax(&self) -> &LuaSyntaxNode {
        &self.syntax
    }

    fn can_cast(kind: LuaSyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == LuaSyntaxKind::DocAliasOrTypeList
    }

    fn cast(syntax: LuaSyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        if syntax.kind() == LuaSyntaxKind::DocAliasOrTypeList.into() {
            Some(Self { syntax })
        } else {
            None
        }
    }
}

impl LuaDocAliasFieldList {
    pub fn get_fields(&self) -> LuaAstChildren<LuaDocAliasField> {
        self.children()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LuaDocAliasField {
    syntax: LuaSyntaxNode,
}

impl LuaAstNode for LuaDocAliasField {
    fn syntax(&self) -> &LuaSyntaxNode {
        &self.syntax
    }

    fn can_cast(kind: LuaSyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == LuaSyntaxKind::DocAliasOrType
    }

    fn cast(syntax: LuaSyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        if syntax.kind() == LuaSyntaxKind::DocAliasOrType.into() {
            Some(Self { syntax })
        } else {
            None
        }
    }
}

impl LuaDocDetailOwner for LuaDocAliasField {}

impl LuaDocAliasField {
    pub fn get_type(&self) -> Option<LuaDocType> {
        self.child()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LuaDocTagType {
    syntax: LuaSyntaxNode,
}

impl LuaAstNode for LuaDocTagType {
    fn syntax(&self) -> &LuaSyntaxNode {
        &self.syntax
    }

    fn can_cast(kind: LuaSyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == LuaSyntaxKind::DocTagType
    }

    fn cast(syntax: LuaSyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        if syntax.kind() == LuaSyntaxKind::DocTagType.into() {
            Some(Self { syntax })
        } else {
            None
        }
    }
}

impl LuaDocDescriptionOwner for LuaDocTagType {}

impl LuaDocTagType {
    pub fn get_type_list(&self) -> LuaAstChildren<LuaDocType> {
        self.children()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LuaDocTagParam {
    syntax: LuaSyntaxNode,
}

impl LuaAstNode for LuaDocTagParam {
    fn syntax(&self) -> &LuaSyntaxNode {
        &self.syntax
    }

    fn can_cast(kind: LuaSyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == LuaSyntaxKind::DocTagParam
    }

    fn cast(syntax: LuaSyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        if syntax.kind() == LuaSyntaxKind::DocTagParam.into() {
            Some(Self { syntax })
        } else {
            None
        }
    }
}

impl LuaDocDescriptionOwner for LuaDocTagParam {}

impl LuaDocTagParam {
    pub fn get_name(&self) -> Option<LuaNameToken> {
        self.token()
    }

    pub fn is_vararg(&self) -> bool {
        self.token_by_kind(LuaTokenKind::TkDots).is_some()
    }

    pub fn is_nullable(&self) -> bool {
        self.token_by_kind(LuaTokenKind::TkDocQuestion).is_some()
    }

    pub fn get_type(&self) -> Option<LuaDocType> {
        self.child()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LuaDocTagReturn {
    syntax: LuaSyntaxNode,
}

impl LuaAstNode for LuaDocTagReturn {
    fn syntax(&self) -> &LuaSyntaxNode {
        &self.syntax
    }

    fn can_cast(kind: LuaSyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == LuaSyntaxKind::DocTagReturn
    }

    fn cast(syntax: LuaSyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        if syntax.kind() == LuaSyntaxKind::DocTagReturn.into() {
            Some(Self { syntax })
        } else {
            None
        }
    }
}

impl LuaDocDescriptionOwner for LuaDocTagReturn {}

impl LuaDocTagReturn {
    pub fn get_first_type(&self) -> Option<LuaDocType> {
        self.child()
    }

    pub fn get_types(&self) -> LuaAstChildren<LuaDocType> {
        self.children()
    }

    // todo
    // pub fn get_type_name_pairs(&self) -> LuaAstChildren<LuaDocTypeNamePair> {
    //     self.children()
    // }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LuaDocTagOverload {
    syntax: LuaSyntaxNode,
}

impl LuaAstNode for LuaDocTagOverload {
    fn syntax(&self) -> &LuaSyntaxNode {
        &self.syntax
    }

    fn can_cast(kind: LuaSyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == LuaSyntaxKind::DocTagOverload
    }

    fn cast(syntax: LuaSyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        if syntax.kind() == LuaSyntaxKind::DocTagOverload.into() {
            Some(Self { syntax })
        } else {
            None
        }
    }
}

impl LuaDocDescriptionOwner for LuaDocTagOverload {}

impl LuaDocTagOverload {
    // todo use luaFuncType
    pub fn get_type(&self) -> Option<LuaDocType> {
        self.child()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LuaDocTagField {
    syntax: LuaSyntaxNode,
}

impl LuaAstNode for LuaDocTagField {
    fn syntax(&self) -> &LuaSyntaxNode {
        &self.syntax
    }

    fn can_cast(kind: LuaSyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == LuaSyntaxKind::DocTagField
    }

    fn cast(syntax: LuaSyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        if syntax.kind() == LuaSyntaxKind::DocTagField.into() {
            Some(Self { syntax })
        } else {
            None
        }
    }
}

impl LuaDocDescriptionOwner for LuaDocTagField {}

impl LuaDocTagField {
    pub fn get_name(&self) -> Option<LuaDocFieldKey> {
        let mut meet_left_bracket = false;
        for child in self.syntax.children_with_tokens() {
            if meet_left_bracket {
                match child {
                    rowan::NodeOrToken::Node(node) => {
                        if LuaDocType::can_cast(node.kind().into()) {
                            return Some(LuaDocFieldKey::Type(LuaDocType::cast(node).unwrap()));
                        }
                    }
                    rowan::NodeOrToken::Token(token) => {
                        match token.kind().into() {
                            LuaTokenKind::TkString => {
                                return Some(LuaDocFieldKey::String(LuaStringToken::cast(token.clone()).unwrap()));
                            }
                            LuaTokenKind::TkInt => {
                                return Some(LuaDocFieldKey::Integer(LuaNumberToken::cast(token.clone()).unwrap()));
                            }
                            _ => {}
                        }
                    },
                }
            } else {
                if let Some(token) = child.as_token() {
                    if token.kind() == LuaTokenKind::TkLeftBracket.into() {
                        meet_left_bracket = true;
                    } else if token.kind() == LuaTokenKind::TkName.into() {
                        return Some(LuaDocFieldKey::Name(LuaNameToken::cast(token.clone()).unwrap()));
                    }
                }
            }
        }

        None
    }

    pub fn get_type(&self) -> Option<LuaDocType> {
        self.children().last()
    }

    pub fn is_nullable(&self) -> bool {
        self.token_by_kind(LuaTokenKind::TkDocQuestion).is_some()
    }

    pub fn get_visibility_token(&self) -> Option<LuaDocVisibilityToken> {
        self.token()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LuaDocFieldKey {
    Name(LuaNameToken),
    String(LuaStringToken),
    Integer(LuaNumberToken),
    Type(LuaDocType),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LuaDocTagModule {
    syntax: LuaSyntaxNode,
}

impl LuaAstNode for LuaDocTagModule {
    fn syntax(&self) -> &LuaSyntaxNode {
        &self.syntax
    }

    fn can_cast(kind: LuaSyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == LuaSyntaxKind::DocTagModule
    }

    fn cast(syntax: LuaSyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        if syntax.kind() == LuaSyntaxKind::DocTagModule.into() {
            Some(Self { syntax })
        } else {
            None
        }
    }
}

impl LuaDocDescriptionOwner for LuaDocTagModule {}

impl LuaDocTagModule {
    pub fn get_string(&self) -> Option<LuaStringToken> {
        self.token()
    }

    pub fn get_name(&self) -> Option<LuaNameToken> {
        self.token()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LuaDocTagSee {
    syntax: LuaSyntaxNode,
}

impl LuaAstNode for LuaDocTagSee {
    fn syntax(&self) -> &LuaSyntaxNode {
        &self.syntax
    }

    fn can_cast(kind: LuaSyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == LuaSyntaxKind::DocTagSee
    }

    fn cast(syntax: LuaSyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        if syntax.kind() == LuaSyntaxKind::DocTagSee.into() {
            Some(Self { syntax })
        } else {
            None
        }
    }
}

impl LuaDocDescriptionOwner for LuaDocTagSee {}

impl LuaDocTagSee {
    pub fn get_names(&self) -> LuaAstTokenChildren<LuaNameToken> {
        self.tokens()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LuaDocTagDiagnostic {
    syntax: LuaSyntaxNode,
}

impl LuaAstNode for LuaDocTagDiagnostic {
    fn syntax(&self) -> &LuaSyntaxNode {
        &self.syntax
    }

    fn can_cast(kind: LuaSyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == LuaSyntaxKind::DocTagDiagnostic
    }

    fn cast(syntax: LuaSyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        if syntax.kind() == LuaSyntaxKind::DocTagDiagnostic.into() {
            Some(Self { syntax })
        } else {
            None
        }
    }
}

impl LuaDocDescriptionOwner for LuaDocTagDiagnostic {}

impl LuaDocTagDiagnostic {
    pub fn get_action_token(&self) -> Option<LuaNameToken> {
        self.token()
    }

    pub fn get_code_list(&self) -> Option<LuaDocDiagnosticCodeList> {
        self.child()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LuaDocDiagnosticCodeList {
    syntax: LuaSyntaxNode,
}

impl LuaAstNode for LuaDocDiagnosticCodeList {
    fn syntax(&self) -> &LuaSyntaxNode {
        &self.syntax
    }

    fn can_cast(kind: LuaSyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == LuaSyntaxKind::DocDiagnosticCodeList
    }

    fn cast(syntax: LuaSyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        if syntax.kind() == LuaSyntaxKind::DocDiagnosticCodeList.into() {
            Some(Self { syntax })
        } else {
            None
        }
    }
}

impl LuaDocDiagnosticCodeList {
    pub fn get_codes(&self) -> LuaAstTokenChildren<LuaNameToken> {
        self.tokens()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LuaDocTagDeprecated {
    syntax: LuaSyntaxNode,
}

impl LuaAstNode for LuaDocTagDeprecated {
    fn syntax(&self) -> &LuaSyntaxNode {
        &self.syntax
    }

    fn can_cast(kind: LuaSyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == LuaSyntaxKind::DocTagDeprecated
    }

    fn cast(syntax: LuaSyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        if syntax.kind() == LuaSyntaxKind::DocTagDeprecated.into() {
            Some(Self { syntax })
        } else {
            None
        }
    }
}

impl LuaDocDescriptionOwner for LuaDocTagDeprecated {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LuaDocTagVersion {
    syntax: LuaSyntaxNode,
}

impl LuaAstNode for LuaDocTagVersion {
    fn syntax(&self) -> &LuaSyntaxNode {
        &self.syntax
    }

    fn can_cast(kind: LuaSyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == LuaSyntaxKind::DocTagVersion
    }

    fn cast(syntax: LuaSyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        if syntax.kind() == LuaSyntaxKind::DocTagVersion.into() {
            Some(Self { syntax })
        } else {
            None
        }
    }
}

impl LuaDocDescriptionOwner for LuaDocTagVersion {}

impl LuaDocTagVersion {
    // TODO
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LuaDocTagCast {
    syntax: LuaSyntaxNode,
}

impl LuaAstNode for LuaDocTagCast {
    fn syntax(&self) -> &LuaSyntaxNode {
        &self.syntax
    }

    fn can_cast(kind: LuaSyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == LuaSyntaxKind::DocTagCast
    }

    fn cast(syntax: LuaSyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        if syntax.kind() == LuaSyntaxKind::DocTagCast.into() {
            Some(Self { syntax })
        } else {
            None
        }
    }
}

impl LuaDocDescriptionOwner for LuaDocTagCast {}

impl LuaDocTagCast {
    pub fn get_name(&self) -> Option<LuaNameToken> {
        self.token()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LuaDocTagSource {
    syntax: LuaSyntaxNode,
}

impl LuaAstNode for LuaDocTagSource {
    fn syntax(&self) -> &LuaSyntaxNode {
        &self.syntax
    }

    fn can_cast(kind: LuaSyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == LuaSyntaxKind::DocTagSource
    }

    fn cast(syntax: LuaSyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        if syntax.kind() == LuaSyntaxKind::DocTagSource.into() {
            Some(Self { syntax })
        } else {
            None
        }
    }
}

impl LuaDocDescriptionOwner for LuaDocTagSource {}

impl LuaDocTagSource {
    pub fn get_string(&self) -> Option<LuaStringToken> {
        self.token()
    }

    pub fn get_name(&self) -> Option<LuaNameToken> {
        self.token()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LuaDocTagOther {
    syntax: LuaSyntaxNode,
}

impl LuaAstNode for LuaDocTagOther {
    fn syntax(&self) -> &LuaSyntaxNode {
        &self.syntax
    }

    fn can_cast(kind: LuaSyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == LuaSyntaxKind::DocTagOther
    }

    fn cast(syntax: LuaSyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        if syntax.kind() == LuaSyntaxKind::DocTagOther.into() {
            Some(Self { syntax })
        } else {
            None
        }
    }
}

impl LuaDocDescriptionOwner for LuaDocTagOther {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LuaDocTagNamespace {
    syntax: LuaSyntaxNode,
}

impl LuaAstNode for LuaDocTagNamespace {
    fn syntax(&self) -> &LuaSyntaxNode {
        &self.syntax
    }

    fn can_cast(kind: LuaSyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == LuaSyntaxKind::DocTagNamespace
    }

    fn cast(syntax: LuaSyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        if syntax.kind() == LuaSyntaxKind::DocTagNamespace.into() {
            Some(Self { syntax })
        } else {
            None
        }
    }
}

impl LuaDocDescriptionOwner for LuaDocTagNamespace {}

impl LuaDocTagNamespace {
    pub fn get_name(&self) -> Option<LuaNameToken> {
        self.token()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LuaDocTagUsing {
    syntax: LuaSyntaxNode,
}

impl LuaAstNode for LuaDocTagUsing {
    fn syntax(&self) -> &LuaSyntaxNode {
        &self.syntax
    }

    fn can_cast(kind: LuaSyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == LuaSyntaxKind::DocTagUsing
    }

    fn cast(syntax: LuaSyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        if syntax.kind() == LuaSyntaxKind::DocTagUsing.into() {
            Some(Self { syntax })
        } else {
            None
        }
    }
}

impl LuaDocDescriptionOwner for LuaDocTagUsing {}

impl LuaDocTagUsing {
    pub fn get_name(&self) -> Option<LuaNameToken> {
        self.token()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LuaDocTagMeta {
    syntax: LuaSyntaxNode,
}

impl LuaAstNode for LuaDocTagMeta {
    fn syntax(&self) -> &LuaSyntaxNode {
        &self.syntax
    }

    fn can_cast(kind: LuaSyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == LuaSyntaxKind::DocTagMeta
    }

    fn cast(syntax: LuaSyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        if syntax.kind() == LuaSyntaxKind::DocTagMeta.into() {
            Some(Self { syntax })
        } else {
            None
        }
    }
}