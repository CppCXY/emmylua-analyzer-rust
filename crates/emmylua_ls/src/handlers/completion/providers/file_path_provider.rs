use std::path::{Path, PathBuf};

use code_analysis::file_path_to_uri;
use emmylua_parser::{LuaAstToken, LuaStringToken};
use lsp_types::CompletionItem;

use crate::handlers::completion::completion_builder::CompletionBuilder;

pub fn add_completion(builder: &mut CompletionBuilder) -> Option<()> {
    if builder.is_cancelled() {
        return None;
    }

    let string_token = LuaStringToken::cast(builder.trigger_token.clone())?;
    let maybe_file_path = string_token.get_value();
    if maybe_file_path.find(|c| c == '/' || c == '\\').is_none() {
        return None;
    }

    let prefix = if let Some(last_sep) = maybe_file_path.rfind(|c| c == '/' || c == '\\') {
        let (path, _) = maybe_file_path.split_at(last_sep + 1);
        path
    } else {
        ""
    };

    let resources = builder.semantic_model.get_emmyrc().resource.paths.clone();

    let suffix = prefix;

    for resource in resources {
        let path = Path::new(&resource);
        let folder = path.join(suffix);
        if folder.exists() && folder.is_dir() {
            if let Ok(entries) = std::fs::read_dir(folder) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        add_file_path_completion(builder, &path, name, prefix);
                    }
                }
            }
        }
    }

    builder.stop_here();

    Some(())
}

fn add_file_path_completion(
    builder: &mut CompletionBuilder,
    path: &PathBuf,
    name: &str,
    prefix: &str,
) -> Option<()> {
    let kind: lsp_types::CompletionItemKind = if path.is_dir() {
        lsp_types::CompletionItemKind::FOLDER
    } else {
        lsp_types::CompletionItemKind::FILE
    };

    let detail = match file_path_to_uri(path) {
        Some(uri) => Some(uri.to_string()),
        None => None,
    };

    let filter_text = format!("{}{}", prefix, name);
    let completion_item = CompletionItem {
        label: name.to_string(),
        kind: Some(kind),
        filter_text: Some(filter_text.clone()),
        insert_text: Some(filter_text),
        detail,
        ..Default::default()
    };

    builder.add_completion_item(completion_item)?;

    Some(())
}