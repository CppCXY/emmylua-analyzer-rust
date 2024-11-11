use std::collections::{HashMap, HashSet};

pub use signature::{LuaSignature, LuaSignatureId, LuaDocParamInfo, LuaDocReturnInfo};

use crate::FileId;

use super::traits::LuaIndex;

mod signature;

#[derive(Debug)]
pub struct LuaSignatureIndex {
    signatures: HashMap<LuaSignatureId, LuaSignature>,
    in_file_signatures: HashMap<FileId, HashSet<LuaSignatureId>>,
}

impl LuaSignatureIndex {
    pub fn new() -> Self {
        Self {
            signatures: HashMap::new(),
            in_file_signatures: HashMap::new(),
        }
    }

    pub fn get_or_create(&mut self, signature_id: LuaSignatureId) -> &mut LuaSignature {
        self.in_file_signatures.entry(signature_id.file_id).or_default().insert(signature_id);
        self.signatures.entry(signature_id).or_insert_with(LuaSignature::new)
    }

    pub fn get(&self, signature_id: &LuaSignatureId) -> Option<&LuaSignature> {
        self.signatures.get(signature_id)
    }
}

impl LuaIndex for LuaSignatureIndex {
    fn remove(&mut self, file_id: FileId) {
        if let Some(signature_ids) = self.in_file_signatures.remove(&file_id) {
            for signature_id in signature_ids {
                self.signatures.remove(&signature_id);
            }
        }
    }
}