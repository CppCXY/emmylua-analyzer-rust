mod syntax_error;
mod analyze_error;

use lsp_types::{Diagnostic, DiagnosticSeverity, DiagnosticTag, NumberOrString};
use rowan::TextRange;
use std::{collections::HashSet, fmt::Debug, sync::Arc};

use crate::{db_index::DbIndex, semantic::SemanticModel, FileId};

use super::{lua_diagnostic_code::{get_default_severity, is_code_default_enable}, DiagnosticCode};

pub trait LuaChecker: Debug + Send + Sync {
    fn check(&self, context: &mut DiagnosticContext) -> Option<()>;

    fn support_codes(&self) -> &[DiagnosticCode];
}

macro_rules! checker {
    ($name:ident) => {
        Box::new($name::Checker())
    };
}

pub fn init_checkers() -> Vec<Box<dyn LuaChecker>> {
    vec![
        checker!(syntax_error),
        checker!(analyze_error),
    ]
}

pub struct DiagnosticContext<'a> {
    semantic_model: SemanticModel<'a>,
    diagnostics: Vec<Diagnostic>,
    workspace_enabled: Arc<HashSet<DiagnosticCode>>,
    workspace_disabled: Arc<HashSet<DiagnosticCode>>,
}

impl<'a> DiagnosticContext<'a> {
    pub fn new(
        semantic_model: SemanticModel<'a>,
        workspace_enabled: Arc<HashSet<DiagnosticCode>>,
        workspace_disabled: Arc<HashSet<DiagnosticCode>>,
    ) -> Self {
        Self {
            semantic_model,
            diagnostics: Vec::new(),
            workspace_disabled,
            workspace_enabled
        }
    }

    pub fn get_db(&self) -> &DbIndex {
        &self.semantic_model.get_db()
    }

    pub fn get_file_id(&self) -> FileId {
        self.semantic_model.get_file_id()
    }

    pub fn add_diagnostic(
        &mut self,
        code: DiagnosticCode,
        range: TextRange,
        message: String,
        data: Option<serde_json::Value>,
    ) {
        if !self.should_report_diagnostic(&code, &range) {
            return;
        }

        let diagnostic = Diagnostic {
            message,
            range: self.translate_range(range).unwrap_or(lsp_types::Range {
                start: lsp_types::Position {
                    line: 0,
                    character: 0,
                },
                end: lsp_types::Position {
                    line: 0,
                    character: 0,
                },
            }),
            severity: self.get_severity(code),
            code: Some(NumberOrString::String(code.get_name().to_string())),
            source: Some("EmmyLua".into()),
            tags: self.get_tags(code),
            data,
            ..Default::default()
        };

        self.diagnostics.push(diagnostic);
    }

    fn should_report_diagnostic(&self, code: &DiagnosticCode, range: &TextRange) -> bool {
        let diagnostic_index = self.get_db().get_diagnostic_index();

        !diagnostic_index.is_file_diagnostic_code_disabled(&self.get_file_id(), code, range)
    }

    fn get_severity(&self, code: DiagnosticCode) -> Option<DiagnosticSeverity> {
        Some(get_default_severity(code))
    }

    fn get_tags(&self, code: DiagnosticCode) -> Option<Vec<DiagnosticTag>> {
        match code {
            DiagnosticCode::Unused | DiagnosticCode::UnreachableCode => {
                Some(vec![DiagnosticTag::UNNECESSARY])
            }
            DiagnosticCode::Deprecated => Some(vec![DiagnosticTag::DEPRECATED]),
            _ => None,
        }
    }

    fn translate_range(&self, range: TextRange) -> Option<lsp_types::Range> {
        let document = self.semantic_model.get_document();
        let (start_line, start_character) = document.get_line_col(range.start())?;
        let (end_line, end_character) = document.get_line_col(range.end())?;

        Some(lsp_types::Range {
            start: lsp_types::Position {
                line: start_line as u32,
                character: start_character as u32,
            },
            end: lsp_types::Position {
                line: end_line as u32,
                character: end_character as u32,
            },
        })
    }

    pub fn get_diagnostics(self) -> Vec<Diagnostic> {
        self.diagnostics
    }

    pub fn is_checker_enable_by_code(
        &self,
        code: &DiagnosticCode,
    ) -> bool {
        let file_id = self.get_file_id();
        let db = self.get_db();
        let diagnostic_index = db.get_diagnostic_index();
        // force enable
        if diagnostic_index.is_file_enabled(&file_id, &code) {
            return true;
        }

        // workspace force disabled
        if self.workspace_disabled.contains(&code) {
            return false;
        }

        let meta_index = db.get_meta_file();
        // ignore meta file diagnostic
        if meta_index.is_meta_file(&file_id) {
            return false;
        }

        // is file disabled this code
        if diagnostic_index.is_file_disabled(&file_id, &code) {
            return false;
        }

        // workspace force enabled
        if self.workspace_enabled.contains(&code) {
            return true;
        }

        // default setting
        is_code_default_enable(&code)
    }
}
