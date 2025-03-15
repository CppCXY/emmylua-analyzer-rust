use std::collections::{HashMap, HashSet};

use emmylua_parser::{LuaAstNode, LuaTableExpr};

use crate::{DiagnosticCode, LuaMemberOwner, LuaType, LuaTypeDeclId, SemanticModel};

use super::{humanize_lint_type, DiagnosticContext};
use itertools::Itertools;

pub const CODES: &[DiagnosticCode] = &[DiagnosticCode::MissingFields];

pub fn check(context: &mut DiagnosticContext, semantic_model: &SemanticModel) -> Option<()> {
    let root = semantic_model.get_root().clone();

    let mut type_cache = HashMap::new();
    for expr in root.descendants::<LuaTableExpr>() {
        check_table_expr(context, semantic_model, &expr, &mut type_cache);
    }
    Some(())
}

fn check_table_expr(
    context: &mut DiagnosticContext,
    semantic_model: &SemanticModel,
    expr: &LuaTableExpr,
    type_cache: &mut HashMap<LuaType, HashSet<String>>,
) -> Option<()> {
    let db = context.db;
    // TODO 前缀为 idnex expr 的表推断异常
    let table_type = semantic_model.infer_table_should_be(expr.clone())?;

    let current_fields = expr
        .get_fields()
        .filter_map(|field| field.get_field_key().map(|key| key.get_path_part()))
        .collect();

    let required_fields = match &table_type {
        LuaType::Ref(type_decl_id) => type_cache.entry(table_type.clone()).or_insert_with(|| {
            let types = type_decl_id.collect_super_types_with_self(context.db, table_type.clone());
            get_required_fields(context, &types).unwrap_or_default()
        }),
        LuaType::Generic(generic_type) => {
            let type_decl_id = generic_type.get_base_type_id();
            type_cache.entry(table_type.clone()).or_insert_with(|| {
                let types =
                    type_decl_id.collect_super_types_with_self(context.db, table_type.clone());
                get_required_fields(context, &types).unwrap_or_default()
            })
        }
        LuaType::Object(_) => type_cache.entry(table_type.clone()).or_insert_with(|| {
            get_required_fields(context, &vec![table_type.clone()]).unwrap_or_default()
        }),
        _ => return Some(()),
    };

    let missing_fields = required_fields
        .difference(&current_fields)
        .map(|s| format!("`{}`", s))
        .sorted()
        .join(", ");

    if !missing_fields.is_empty() {
        context.add_diagnostic(
            DiagnosticCode::MissingFields,
            expr.get_range(),
            t!(
                "Missing required fields in type `%{typ}`: %{fields}",
                typ = humanize_lint_type(&db, &table_type),
                fields = missing_fields
            )
            .to_string(),
            None,
        );
    }

    Some(())
}

fn get_required_fields(
    context: &mut DiagnosticContext,
    // types 应为广度优先, 子类型会先于父类型被遍历, 而子类型的优先级高于父类型
    types: &Vec<LuaType>,
) -> Option<HashSet<String>> {
    let member_index = context.db.get_member_index();
    let mut required_fields: HashSet<String> = HashSet::new();

    let mut optional_type = HashSet::new();
    for super_type in types {
        match super_type {
            LuaType::Ref(type_decl_id) => process_type_decl_id(
                member_index,
                &mut required_fields,
                &mut optional_type,
                type_decl_id.clone(),
            ),
            LuaType::Generic(generic_type) => process_type_decl_id(
                member_index,
                &mut required_fields,
                &mut optional_type,
                generic_type.get_base_type_id().clone(),
            ),
            // 处理 ---@class test: { a: number }
            LuaType::Object(object_type) => {
                let fields = object_type.get_fields();
                for (key, decl_type) in fields {
                    let name = key.to_path();
                    record_required_fields(
                        &mut required_fields,
                        &mut optional_type,
                        name,
                        decl_type.clone(),
                    );
                }
                continue;
            }
            _ => continue,
        };
    }

    fn process_type_decl_id(
        member_index: &crate::LuaMemberIndex,
        required_fields: &mut HashSet<String>,
        optional_type: &mut HashSet<String>,
        type_decl_id: LuaTypeDeclId,
    ) -> Option<()> {
        let members = member_index.get_members(&LuaMemberOwner::Type(type_decl_id))?;
        for member in members {
            let name = member.get_key().to_path();
            let decl_type = member.get_decl_type();
            record_required_fields(required_fields, optional_type, name, decl_type);
        }

        Some(())
    }

    Some(required_fields)
}

fn record_required_fields(
    required_fields: &mut HashSet<String>,
    optional_type: &mut HashSet<String>,
    name: String,
    decl_type: LuaType,
) {
    if name.is_empty() {
        return;
    }

    if decl_type.is_optional() {
        optional_type.insert(name);
        return;
    }
    if optional_type.contains(&name) {
        return;
    }

    required_fields.insert(name);
}
