use emmylua_parser::LuaBinaryExpr;
use rowan::{TextRange, TextSize};

use crate::{LuaFlowId, TypeAssertion, VarRefId};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VarTrace {
    var_ref_id: VarRefId,
    current_flow_id: LuaFlowId,
    assertions: Vec<(TypeAssertion, TextRange)>,
}

#[allow(unused)]
impl VarTrace {
    pub fn new(var_ref_id: VarRefId) -> Self {
        Self {
            var_ref_id,
            current_flow_id: LuaFlowId::chunk(),
            assertions: Vec::new(),
        }
    }

    pub fn set_current_flow_id(&mut self, flow_id: LuaFlowId) {
        self.current_flow_id = flow_id;
    }

    pub fn get_current_flow_id(&self) -> LuaFlowId {
        self.current_flow_id
    }

    pub fn get_var_ref_id(&self) -> &VarRefId {
        &self.var_ref_id
    }

    pub fn add_assert(&mut self, assertion: TypeAssertion, range: TextRange) {
        self.assertions.push((assertion, range));
    }

    // add for later analysis
    pub fn add_and_assert(
        &mut self,
        assertion: TypeAssertion,
        expr: LuaBinaryExpr,
    ) {
    }

    pub fn add_or_assert(
        &mut self,
        assertion: TypeAssertion,
        expr: LuaBinaryExpr,
    ) {
    }

    pub fn add_union_assert(
        &mut self,
        assertion: TypeAssertion,
        range: TextRange,
        position: TextSize,
    ) {
    }
}

