mod document;
mod in_filed;
mod loader;
mod test;

pub use document::LuaDocument;
use emmylua_parser::{LineIndex, LuaParser, LuaSyntaxTree};
pub use in_filed::InFiled;
pub use loader::{load_workspace_files, read_file_with_encoding, LuaFileInfo};
use lsp_types::Uri;
use rowan::{NodeCache, TextRange};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use url::Url;

use crate::Emmyrc;

#[derive(Eq, PartialEq, Hash, Debug, Clone, Copy)]
pub struct FileId {
    pub id: u32,
}

impl Serialize for FileId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u32(self.id)
    }
}

impl<'de> Deserialize<'de> for FileId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let id = u32::deserialize(deserializer)?;
        Ok(FileId { id })
    }
}

impl FileId {
    pub fn new() -> Self {
        FileId { id: 0 }
    }
}

#[derive(Debug)]
pub struct Vfs {
    file_id_map: HashMap<Uri, u32>,
    file_uri_map: HashMap<u32, Uri>,
    file_data: Vec<Option<String>>,
    line_index_map: HashMap<FileId, LineIndex>,
    tree_map: HashMap<FileId, LuaSyntaxTree>,
    emmyrc: Option<Arc<Emmyrc>>,
    node_cache: NodeCache,
}

impl Vfs {
    pub fn new() -> Self {
        Vfs {
            file_id_map: HashMap::new(),
            file_uri_map: HashMap::new(),
            file_data: Vec::new(),
            line_index_map: HashMap::new(),
            tree_map: HashMap::new(),
            emmyrc: None,
            node_cache: NodeCache::default(),
        }
    }

    pub fn file_id(&mut self, uri: &Uri) -> FileId {
        if let Some(&id) = self.file_id_map.get(uri) {
            FileId { id }
        } else {
            let id = self.file_data.len() as u32;
            self.file_id_map.insert(uri.clone(), id);
            self.file_uri_map.insert(id, uri.clone());
            self.file_data.push(None);
            FileId { id }
        }
    }

    pub fn get_file_id(&self, uri: &Uri) -> Option<FileId> {
        self.file_id_map.get(uri).map(|&id| FileId { id })
    }

    pub fn get_uri(&self, id: &FileId) -> Option<&Uri> {
        self.file_uri_map.get(&id.id)
    }

    pub fn set_file_content(&mut self, uri: &Uri, data: Option<String>) -> FileId {
        let fid = self.file_id(uri);
        if let Some(data) = &data {
            let line_index = LineIndex::parse(&data);
            let parse_config = self
                .emmyrc
                .as_ref()
                .unwrap()
                .get_parse_config(&mut self.node_cache);
            let tree = LuaParser::parse(&data, parse_config);
            self.tree_map.insert(fid, tree);
            self.line_index_map.insert(fid, line_index);
        } else {
            self.line_index_map.remove(&fid);
            self.tree_map.remove(&fid);
        }
        self.file_data[fid.id as usize] = data;
        fid
    }

    pub fn update_config(&mut self, emmyrc: Arc<Emmyrc>) {
        self.emmyrc = Some(emmyrc);
    }

    pub fn get_file_content(&self, id: &FileId) -> Option<&String> {
        let opt = &self.file_data[id.id as usize];
        if let Some(s) = opt {
            Some(s)
        } else {
            None
        }
    }

    pub fn get_document(&self, id: &FileId) -> Option<LuaDocument> {
        let uri = self.file_uri_map.get(&id.id)?;
        let text = self.get_file_content(id)?;
        let line_index = self.line_index_map.get(id)?;
        Some(LuaDocument::new(*id, uri, text, line_index))
    }

    pub fn get_syntax_tree(&self, id: &FileId) -> Option<&LuaSyntaxTree> {
        self.tree_map.get(id)
    }

    pub fn get_file_parse_error(&self, id: &FileId) -> Option<Vec<(String, TextRange)>> {
        let mut errors = Vec::new();
        let tree = self.tree_map.get(id)?;
        for error in tree.get_errors() {
            errors.push((error.message.clone(), error.range.clone()));
        }

        if errors.is_empty() {
            None
        } else {
            Some(errors)
        }
    }
}

pub fn file_path_to_uri(path: &PathBuf) -> Option<Uri> {
    match Url::from_file_path(path) {
        Ok(url) => Some(Uri::from_str(url.as_str()).unwrap()),
        Err(_) => None,
    }
}

pub fn uri_to_file_path(uri: &Uri) -> Option<PathBuf> {
    if uri.scheme().unwrap().as_str() != "file" {
        return None;
    }

    let url = Url::from_str(uri.as_str()).unwrap();
    Some(url.to_file_path().unwrap())
}
