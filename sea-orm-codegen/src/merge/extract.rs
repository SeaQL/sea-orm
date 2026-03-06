use super::is_active_model_behavior_impl;
use syn::File;
use syn::Item;
use syn::ItemEnum;
use syn::ItemImpl;
use syn::ItemStruct;
use syn::ItemUse;

pub(super) fn extract_active_model_behavior_impls(file: &File) -> impl Iterator<Item = &ItemImpl> {
    file.items.iter().filter_map(|item| match item {
        Item::Impl(item_impl) if is_active_model_behavior_impl(item_impl) => Some(item_impl),
        _ => None,
    })
}

pub(super) fn extract_top_level_uses(file: &File) -> impl Iterator<Item = &ItemUse> {
    file.items.iter().filter_map(|item| match item {
        Item::Use(item_use) => Some(item_use),
        _ => None,
    })
}

pub(super) fn find_model_struct(file: &File) -> Option<&ItemStruct> {
    file.items.iter().find_map(|item| {
        if let Item::Struct(item_struct) = item {
            if item_struct.ident == "Model" {
                return Some(item_struct);
            }
        }
        None
    })
}

pub(super) fn find_relation_enum(file: &File) -> Option<&ItemEnum> {
    file.items.iter().find_map(|item| {
        if let Item::Enum(item_enum) = item {
            if item_enum.ident == "Relation" {
                return Some(item_enum);
            }
        }
        None
    })
}
