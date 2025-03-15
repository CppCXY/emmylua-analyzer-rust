use emmylua_parser::{
    LuaAstNode, LuaAstToken, LuaClosureExpr, LuaExpr, LuaIndexExpr, LuaNameExpr, LuaStat,
    LuaSyntaxKind,
};

use crate::{
    semantic::member::{get_buildin_type_map_type_id, infer_member_map, without_members},
    DbIndex, LuaDeclId, LuaDeclOrMemberId, LuaInferCache, LuaInstanceType, LuaMemberId,
    LuaMemberKey, LuaPropertyOwnerId, LuaType, LuaTypeDeclId, LuaUnionType,
};

use super::{infer_expr, infer_token_property_owner, owner_guard::OwnerGuard};

pub fn infer_expr_property_owner(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    expr: LuaExpr,
    owner_guard: OwnerGuard,
) -> Option<LuaPropertyOwnerId> {
    let file_id = cache.get_file_id();
    let maybe_decl_id = LuaDeclId::new(file_id, expr.get_position());
    if let Some(_) = db.get_decl_index().get_decl(&maybe_decl_id) {
        return Some(LuaPropertyOwnerId::LuaDecl(maybe_decl_id));
    };

    match expr {
        LuaExpr::NameExpr(name_expr) => {
            infer_name_expr_property_owner(db, cache, name_expr, owner_guard.next_level()?)
        }
        LuaExpr::IndexExpr(index_expr) => {
            infer_index_expr_property_owner(db, cache, index_expr, owner_guard.next_level()?)
        }
        LuaExpr::ClosureExpr(closure_expr) => {
            infer_closure_expr_property_owner(db, cache, closure_expr, owner_guard.next_level()?)
        }
        _ => {
            let member_id = LuaMemberId::new(expr.get_syntax_id(), file_id);
            if let Some(_) = db.get_member_index().get_member(&member_id) {
                return Some(LuaPropertyOwnerId::Member(member_id));
            };

            None
        }
    }
}

fn infer_name_expr_property_owner(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    name_expr: LuaNameExpr,
    owner_guard: OwnerGuard,
) -> Option<LuaPropertyOwnerId> {
    let name_token = name_expr.get_name_token()?;
    let name = name_token.get_name_text().to_string();
    if name == "self" {
        return infer_self_property_owner(db, cache, name_expr);
    }

    let decl_id = get_name_decl_id(db, cache, &name, name_expr.clone())?;
    let decl = db.get_decl_index().get_decl(&decl_id)?;
    if owner_guard.reached_limit() {
        return Some(LuaPropertyOwnerId::LuaDecl(decl_id));
    }

    let is_function = decl.get_type().map_or(false, |typ| typ.is_function());
    if decl.is_local() && !is_function {
        return Some(LuaPropertyOwnerId::LuaDecl(decl_id));
    }

    if let Some(value_expr_id) = decl.get_value_syntax_id() {
        if matches!(
            value_expr_id.get_kind(),
            LuaSyntaxKind::NameExpr | LuaSyntaxKind::IndexExpr
        ) {
            let file_id = decl.get_file_id();
            let tree = db.get_vfs().get_syntax_tree(&file_id)?;
            // second infer
            let value_expr = LuaExpr::cast(value_expr_id.to_node(tree)?)?;
            if let Some(property_owner_id) =
                infer_expr_property_owner(db, cache, value_expr, owner_guard.next_level()?)
            {
                return Some(property_owner_id);
            }
        }
    }

    Some(LuaPropertyOwnerId::LuaDecl(decl_id))
}

fn get_name_decl_id(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    name: &str,
    name_expr: LuaNameExpr,
) -> Option<LuaDeclId> {
    let file_id = cache.get_file_id();
    let references_index = db.get_reference_index();
    let range = name_expr.get_range();
    let local_ref = references_index.get_local_reference(&file_id)?;
    let decl_id = local_ref.get_decl_id(&range);

    if let Some(decl_id) = decl_id {
        let decl = db.get_decl_index().get_decl(&decl_id)?;
        if decl.is_local() {
            return Some(decl_id);
        }
    }

    let decl_id = db
        .get_decl_index()
        .get_global_decl_id(&LuaMemberKey::Name(name.into()));

    decl_id
}

fn infer_self_property_owner(
    db: &DbIndex,
    cache: &LuaInferCache,
    name_expr: LuaNameExpr,
) -> Option<LuaPropertyOwnerId> {
    let file_id = cache.get_file_id();
    let tree = db.get_decl_index().get_decl_tree(&file_id)?;
    let id = tree.find_self_decl(db, name_expr)?;
    match id {
        LuaDeclOrMemberId::Decl(decl_id) => {
            return Some(LuaPropertyOwnerId::LuaDecl(decl_id));
        }
        LuaDeclOrMemberId::Member(member_id) => {
            return Some(LuaPropertyOwnerId::Member(member_id));
        }
    }
}

fn infer_index_expr_property_owner(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    index_expr: LuaIndexExpr,
    owner_guard: OwnerGuard,
) -> Option<LuaPropertyOwnerId> {
    let prefix_expr = index_expr.get_prefix_expr()?;
    let mut prefix_type = infer_expr(db, cache, prefix_expr.into())?;
    if let LuaType::DocStringConst(_) = &prefix_type {
        prefix_type = LuaType::String;
    }
    let member_key = index_expr.get_index_key()?.into();
    if let Some(member_info) = infer_member_property_owner_by_member_key(
        db,
        cache,
        &prefix_type,
        &member_key,
        owner_guard.next_level()?,
    ) {
        return Some(member_info);
    }

    None
}

fn infer_closure_expr_property_owner(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    closure_expr: LuaClosureExpr,
    owner_guard: OwnerGuard,
) -> Option<LuaPropertyOwnerId> {
    let parent = closure_expr.get_parent::<LuaStat>()?;
    match parent {
        LuaStat::LocalFuncStat(local_func_stat) => {
            let local_name = local_func_stat.get_local_name()?;
            let name_token = local_name.get_name_token()?;
            infer_token_property_owner(db, cache, name_token.syntax().clone())
        }
        LuaStat::FuncStat(func_stat) => {
            let func_name = func_stat.get_func_name()?;
            infer_expr_property_owner(db, cache, func_name.into(), owner_guard.next_level()?)
        }
        _ => None,
    }
}

fn infer_member_property_owner_by_member_key(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    prefix_type: &LuaType,
    member_key: &LuaMemberKey,
    owner_guard: OwnerGuard,
) -> Option<LuaPropertyOwnerId> {
    if without_members(prefix_type) {
        return None;
    }

    match &prefix_type {
        LuaType::TableConst(id) => {
            let table_type = LuaType::TableConst(id.clone());
            infer_table_member_property_owner(db, cache, table_type, member_key)
        }
        LuaType::String | LuaType::Io | LuaType::StringConst(_) => {
            let decl_id = get_buildin_type_map_type_id(&prefix_type)?;
            infer_custom_type_member_property_owner(
                db,
                cache,
                decl_id,
                member_key,
                owner_guard.next_level()?,
            )
        }
        LuaType::Ref(decl_id) => infer_custom_type_member_property_owner(
            db,
            cache,
            decl_id.clone(),
            member_key,
            owner_guard.next_level()?,
        ),
        LuaType::Def(decl_id) => infer_custom_type_member_property_owner(
            db,
            cache,
            decl_id.clone(),
            member_key,
            owner_guard.next_level()?,
        ),
        LuaType::Nullable(inner_type) => infer_member_property_owner_by_member_key(
            db,
            cache,
            &inner_type,
            member_key,
            owner_guard.next_level()?,
        ),
        LuaType::Union(union_type) => infer_union_member_semantic_info(
            db,
            cache,
            &union_type,
            member_key,
            owner_guard.next_level()?,
        ),
        LuaType::Generic(generic_type) => infer_custom_type_member_property_owner(
            db,
            cache,
            generic_type.get_base_type_id(),
            member_key,
            owner_guard.next_level()?,
        ),
        LuaType::Instance(inst) => infer_instance_member_property_by_member_key(
            db,
            cache,
            inst,
            member_key,
            owner_guard.next_level()?,
        ),
        LuaType::Global => infer_global_member_property_by_member_key(
            db,
            cache,
            member_key,
            owner_guard.next_level()?,
        ),
        _ => None,
    }
}

fn infer_table_member_property_owner(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    table_type: LuaType,
    member_key: &LuaMemberKey,
) -> Option<LuaPropertyOwnerId> {
    let member_map = infer_member_map(db, cache, &table_type)?;
    let member_infos = member_map.get(&member_key)?;
    member_infos.first()?.property_owner_id.clone()
}

fn infer_custom_type_member_property_owner(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    prefix_type_id: LuaTypeDeclId,
    member_key: &LuaMemberKey,
    owner_guard: OwnerGuard,
) -> Option<LuaPropertyOwnerId> {
    let type_index = db.get_type_index();
    let type_decl = type_index.get_type_decl(&prefix_type_id)?;
    if type_decl.is_alias() {
        if let Some(origin_type) = type_decl.get_alias_origin(db, None) {
            return infer_member_property_owner_by_member_key(
                db,
                cache,
                &origin_type,
                member_key,
                owner_guard.next_level()?,
            );
        } else {
            return infer_member_property_owner_by_member_key(
                db,
                cache,
                &LuaType::String,
                member_key,
                owner_guard.next_level()?,
            );
        }
    }

    let member_map = infer_member_map(db, cache, &&LuaType::Ref(prefix_type_id))?;

    let member_infos = member_map.get(&member_key)?;
    member_infos.first()?.property_owner_id.clone()
}

fn infer_union_member_semantic_info(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    union_type: &LuaUnionType,
    member_key: &LuaMemberKey,
    owner_guard: OwnerGuard,
) -> Option<LuaPropertyOwnerId> {
    for typ in union_type.get_types() {
        if let Some(property_owner_id) = infer_member_property_owner_by_member_key(
            db,
            cache,
            typ,
            member_key,
            owner_guard.next_level()?,
        ) {
            return Some(property_owner_id);
        }
    }

    None
}

fn infer_instance_member_property_by_member_key(
    db: &DbIndex,
    cache: &mut LuaInferCache,
    inst: &LuaInstanceType,
    member_key: &LuaMemberKey,
    owner_guard: OwnerGuard,
) -> Option<LuaPropertyOwnerId> {
    let range = inst.get_range();

    let origin_type = inst.get_base();
    if let Some(result) = infer_member_property_owner_by_member_key(
        db,
        cache,
        origin_type,
        member_key,
        owner_guard.next_level()?,
    ) {
        return Some(result);
    }

    let table_type = LuaType::TableConst(range.clone());
    infer_table_member_property_owner(db, cache, table_type, member_key)
}

fn infer_global_member_property_by_member_key(
    db: &DbIndex,
    _: &LuaInferCache,
    member_key: &LuaMemberKey,
    _: OwnerGuard,
) -> Option<LuaPropertyOwnerId> {
    let decl_id = db.get_decl_index().get_global_decl_id(member_key);

    if decl_id.is_some() {
        Some(LuaPropertyOwnerId::LuaDecl(decl_id.unwrap()))
    } else {
        None
    }
}
