#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum LuaOperatorMetaMethod {
    Add,    // +
    Sub,    // -
    Mul,    // *
    Div,    // /
    Mod,    // %
    Pow,    // ^
    Unm,    // -
    IDiv,   // //
    BAnd,   // &
    BOr,    // |
    BXor,   // ~
    BNot,   // ~
    Shl,    // <<
    Shr,    // >>
    Concat, // ..
    Len,    // #
    Eq,     // ==
    Lt,     // <
    Le,     // <=
    Index,  // __index
    Call,   // __call
    Pairs,  // __pairs unimplemented
}

impl LuaOperatorMetaMethod {
    pub fn from_operator_name(op: &str) -> Option<Self> {
        match op {
            "add" => Some(LuaOperatorMetaMethod::Add),
            "sub" => Some(LuaOperatorMetaMethod::Sub),
            "mul" => Some(LuaOperatorMetaMethod::Mul),
            "div" => Some(LuaOperatorMetaMethod::Div),
            "mod" => Some(LuaOperatorMetaMethod::Mod),
            "pow" => Some(LuaOperatorMetaMethod::Pow),
            "unm" => Some(LuaOperatorMetaMethod::Unm),
            "idiv" => Some(LuaOperatorMetaMethod::IDiv),
            "band" => Some(LuaOperatorMetaMethod::BAnd),
            "bor" => Some(LuaOperatorMetaMethod::BOr),
            "bxor" => Some(LuaOperatorMetaMethod::BXor),
            "bnot" => Some(LuaOperatorMetaMethod::BNot),
            "shl" => Some(LuaOperatorMetaMethod::Shl),
            "shr" => Some(LuaOperatorMetaMethod::Shr),
            "concat" => Some(LuaOperatorMetaMethod::Concat),
            "len" => Some(LuaOperatorMetaMethod::Len),
            "eq" => Some(LuaOperatorMetaMethod::Eq),
            "lt" => Some(LuaOperatorMetaMethod::Lt),
            "le" => Some(LuaOperatorMetaMethod::Le),
            "call" => Some(LuaOperatorMetaMethod::Call),
            "pairs" => Some(LuaOperatorMetaMethod::Pairs),
            _ => None,
        }
    }

    pub fn from_metatable_name(name: &str) -> Option<Self> {
        match name {
            "__add" => Some(LuaOperatorMetaMethod::Add),
            "__sub" => Some(LuaOperatorMetaMethod::Sub),
            "__mul" => Some(LuaOperatorMetaMethod::Mul),
            "__div" => Some(LuaOperatorMetaMethod::Div),
            "__mod" => Some(LuaOperatorMetaMethod::Mod),
            "__pow" => Some(LuaOperatorMetaMethod::Pow),
            "__unm" => Some(LuaOperatorMetaMethod::Unm),
            "__idiv" => Some(LuaOperatorMetaMethod::IDiv),
            "__band" => Some(LuaOperatorMetaMethod::BAnd),
            "__bor" => Some(LuaOperatorMetaMethod::BOr),
            "__bxor" => Some(LuaOperatorMetaMethod::BXor),
            "__bnot" => Some(LuaOperatorMetaMethod::BNot),
            "__shl" => Some(LuaOperatorMetaMethod::Shl),
            "__shr" => Some(LuaOperatorMetaMethod::Shr),
            "__concat" => Some(LuaOperatorMetaMethod::Concat),
            "__len" => Some(LuaOperatorMetaMethod::Len),
            "__eq" => Some(LuaOperatorMetaMethod::Eq),
            "__lt" => Some(LuaOperatorMetaMethod::Lt),
            "__le" => Some(LuaOperatorMetaMethod::Le),
            "__index" => Some(LuaOperatorMetaMethod::Index),
            "__call" => Some(LuaOperatorMetaMethod::Call),
            _ => None,
        }
    }
}
