use std::collections::{HashMap, HashSet};

use prettyplease::unparse;
use syn::fold::{self, Fold};
use syn::{
    Attribute, Fields, Ident, Item, ItemImpl, Path, parse::Parser, parse_file,
    punctuated::Punctuated,
};
use syn::{ItemUse, Meta};

mod extract;
use extract::*;

#[derive(Default)]
struct OldIndex<'a> {
    model_attrs: &'a [Attribute],
    model_field_attrs: HashMap<&'a Ident, &'a [Attribute]>,

    relation_attrs: &'a [Attribute],
    relation_variant_attrs: HashMap<&'a Ident, &'a [Attribute]>,
}

impl<'a> OldIndex<'a> {
    fn from_file(file: &'a syn::File) -> OldIndex<'a> {
        let mut idx = OldIndex::default();

        if let Some(model) = find_model_struct(file) {
            idx.model_attrs = &model.attrs;
            if let Fields::Named(named) = &model.fields {
                for field in &named.named {
                    if let Some(ident) = &field.ident {
                        idx.model_field_attrs.insert(ident, &field.attrs);
                    }
                }
            }
        }

        if let Some(relation) = find_relation_enum(file) {
            idx.relation_attrs = &relation.attrs;
            for variant in &relation.variants {
                idx.relation_variant_attrs
                    .insert(&variant.ident, &variant.attrs);
            }
        }

        idx
    }
}

struct Merger<'a> {
    old: OldIndex<'a>,
    seen_use: HashSet<ItemUse>,
    extra_uses: Vec<&'a syn::ItemUse>,
    old_behaviors: Vec<&'a syn::ItemImpl>,
}

impl<'a> Merger<'a> {
    fn new(
        old: OldIndex<'a>,
        extra_uses: Vec<&'a syn::ItemUse>,
        old_behaviors: Vec<&'a syn::ItemImpl>,
    ) -> Self {
        Self {
            old,
            seen_use: HashSet::new(),
            extra_uses,
            old_behaviors,
        }
    }
}

impl<'a> Fold for Merger<'a> {
    fn fold_item_struct(&mut self, i: syn::ItemStruct) -> syn::ItemStruct {
        let mut i = fold::fold_item_struct(self, i);
        if i.ident == "Model" {
            merge_item_attributes(&mut i.attrs, self.old.model_attrs);
            if let Fields::Named(named) = &mut i.fields {
                for field in &mut named.named {
                    if let Some(ident) = &field.ident {
                        if let Some(old_attrs) = self.old.model_field_attrs.get(ident) {
                            merge_item_attributes(&mut field.attrs, old_attrs);
                        }
                    }
                }
            }
        }
        i
    }

    fn fold_item_enum(&mut self, i: syn::ItemEnum) -> syn::ItemEnum {
        let mut i = fold::fold_item_enum(self, i);
        if i.ident == "Relation" {
            merge_item_attributes(&mut i.attrs, self.old.relation_attrs);
            for variant in &mut i.variants {
                if let Some(old_attrs) = self.old.relation_variant_attrs.get(&variant.ident) {
                    merge_item_attributes(&mut variant.attrs, old_attrs);
                }
            }
        }
        i
    }

    fn fold_file(&mut self, mut file: syn::File) -> syn::File {
        let mut uses: Vec<Item> = Vec::new();
        let mut others: Vec<Item> = Vec::new();

        for item in file.items {
            match item {
                Item::Use(u) => {
                    let u = fold::fold_item_use(self, u);
                    if self.seen_use.insert(u.clone()) {
                        uses.push(Item::Use(u));
                    }
                }
                Item::Impl(i) => {
                    if is_active_model_behavior_impl(&i) {
                        continue;
                    }
                    let i = fold::fold_item_impl(self, i);
                    others.push(Item::Impl(i));
                }
                other => {
                    let other = fold::fold_item(self, other);
                    others.push(other);
                }
            }
        }

        for &u in &self.extra_uses {
            if self.seen_use.insert(u.clone()) {
                uses.push(Item::Use(u.clone()));
            }
        }

        for &i in &self.old_behaviors {
            others.push(Item::Impl(i.clone()));
        }

        file.items = {
            let mut v = Vec::with_capacity(uses.len() + others.len());
            v.extend(uses);
            v.extend(others);
            v
        };
        file
    }
}

#[derive(Debug, Default)]
#[doc(hidden)]
pub struct MergeReport {
    pub output: String,
    pub warnings: Vec<String>,
    pub fallback_applied: bool,
}

impl MergeReport {
    fn fallback(old_src: &str, new_src: &str) -> Self {
        let mut warnings = Vec::new();
        let mut appended = String::from(new_src);

        warnings.push(
            "Parsing failed. The previous file content has been preserved as comments at the end."
                .to_owned(),
        );

        if !appended.ends_with("\n\n") {
            appended.push('\n');
        }

        appended.push_str(
            "// --- Preserved original file content (could not be merged automatically) ---\n",
        );
        appended.push_str("/*\n");
        appended.push_str(old_src);
        if !old_src.ends_with('\n') {
            appended.push('\n');
        }
        appended.push_str("*/\n\n");

        MergeReport {
            output: appended,
            warnings,
            fallback_applied: true,
        }
    }
}

#[doc(hidden)]
pub fn merge_entity_files(old_src: &str, new_src: &str) -> Result<String, MergeReport> {
    let new_file = match parse_file(new_src) {
        Ok(file) => file,
        Err(err) => {
            let mut report = MergeReport::fallback(old_src, new_src);
            report.warnings.push(format!(
                "Unable to parse new file, generated fallback: {err}"
            ));
            return Err(report);
        }
    };

    let old_file = match parse_file(old_src) {
        Ok(file) => file,
        Err(err) => {
            let mut report = MergeReport::fallback(old_src, new_src);
            report.warnings.push(format!(
                "Unable to parse old file, generated fallback: {err}"
            ));
            return Err(report);
        }
    };

    let old_index = OldIndex::from_file(&old_file);
    let extra_uses = extract_top_level_uses(&old_file).collect::<Vec<_>>();
    let old_behaviors = extract_active_model_behavior_impls(&old_file).collect::<Vec<_>>();
    let mut folder = Merger::new(old_index, extra_uses, old_behaviors);
    let merged_file = folder.fold_file(new_file);

    let output = render_file_with_spacing(merged_file);
    Ok(output)
}

fn merge_item_attributes(new_attrs: &mut Vec<Attribute>, old_attrs: &[Attribute]) {
    merge_derives(new_attrs, old_attrs);

    let mut path_idx_map = HashMap::<Path, usize>::new();
    let mut doc_attr_set = HashSet::<Attribute>::new();

    for (idx, attr) in new_attrs.iter().enumerate() {
        if is_doc_attribute(attr) {
            doc_attr_set.insert(attr.clone());
            continue;
        }
        if attr.path().is_ident("derive") {
            continue;
        }
        path_idx_map.entry(attr.path().clone()).or_insert(idx);
    }

    for old_attr in old_attrs {
        if old_attr.path().is_ident("derive") {
            continue;
        }

        if is_doc_attribute(old_attr) {
            if !doc_attr_set.contains(old_attr) {
                doc_attr_set.insert(old_attr.clone());
                new_attrs.push(old_attr.clone());
            }
            continue;
        }

        if let Some(&idx) = path_idx_map.get(old_attr.path()) {
            let current = &new_attrs[idx];
            if let Some(merged) = merge_non_derive_attribute(old_attr, current) {
                new_attrs[idx] = merged;
            }
        } else {
            path_idx_map.insert(old_attr.path().clone(), new_attrs.len());
            new_attrs.push(old_attr.clone());
        }
    }
}

fn merge_derives(new_attrs: &mut Vec<Attribute>, old_attrs: &[Attribute]) {
    let mut derive_paths: Vec<Path> = Vec::new();
    let mut seen = HashSet::new();
    let mut insert_index = None;
    let mut idx = 0usize;
    while idx < new_attrs.len() {
        if new_attrs[idx].path().is_ident("derive") {
            insert_index.get_or_insert(idx);
            if let Some(paths) = parse_derive_paths(&new_attrs[idx]) {
                for path in paths {
                    if seen.insert(path.clone()) {
                        derive_paths.push(path);
                    }
                }
            }
            new_attrs.remove(idx);
        } else {
            idx += 1;
        }
    }

    for attr in old_attrs
        .iter()
        .filter(|attr| attr.path().is_ident("derive"))
    {
        if let Some(paths) = parse_derive_paths(attr) {
            for path in paths {
                if seen.insert(path.clone()) {
                    derive_paths.push(path);
                }
            }
        }
    }

    if derive_paths.is_empty() {
        return;
    }

    if insert_index.is_none() {
        insert_index = Some(
            new_attrs
                .iter()
                .position(|attr| !is_doc_attribute(attr))
                .unwrap_or(0),
        );
    }

    let derive_attr: Attribute = syn::parse_quote!(#[derive(#(#derive_paths),*)]);
    new_attrs.insert(insert_index.unwrap_or(0), derive_attr);
}

fn parse_derive_paths(attr: &Attribute) -> Option<Vec<Path>> {
    attr.parse_args_with(Punctuated::<Path, syn::Token![,]>::parse_terminated)
        .ok()
        .map(|punct| punct.into_iter().collect())
}

fn merge_non_derive_attribute(old_attr: &Attribute, new_attr: &Attribute) -> Option<Attribute> {
    use syn::{Token, punctuated::Punctuated};

    let parse_attr = |attr: &Attribute| {
        attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            .ok()
    };

    let old_args = parse_attr(old_attr)?;
    let new_args = parse_attr(new_attr)?;

    fn meta_key(meta: &Meta) -> &Path {
        match meta {
            Meta::Path(p) => p,
            Meta::NameValue(nv) => &nv.path,
            Meta::List(list) => &list.path,
        }
    }

    let mut seen_keys = HashSet::<Path>::new();
    let mut merged: Vec<Meta> = Vec::new();

    for m in old_args.into_iter() {
        // Always preserve old attribute values on key conflict
        seen_keys.insert(meta_key(&m).clone());
        merged.push(m);
    }

    for m in new_args.into_iter() {
        let key = meta_key(&m);
        if !seen_keys.contains(key) {
            seen_keys.insert(key.clone());
            merged.push(m);
        }
    }

    let path = old_attr.path();
    let tokens = quote::quote!(#[#path(#(#merged),*)]);
    Attribute::parse_outer
        .parse2(tokens)
        .ok()
        .and_then(|v| v.into_iter().next())
}

fn is_active_model_behavior_impl(item_impl: &ItemImpl) -> bool {
    let trait_ident = item_impl
        .trait_
        .as_ref()
        .and_then(|(_, path, _)| path.segments.last())
        .map(|segment| &segment.ident);
    let self_ident = match item_impl.self_ty.as_ref() {
        syn::Type::Path(type_path) => type_path.path.segments.last().map(|seg| &seg.ident),
        _ => None,
    };
    matches!(trait_ident, Some(ident) if ident == "ActiveModelBehavior")
        && matches!(self_ident, Some(ident) if ident == "ActiveModel")
}

fn is_doc_attribute(attr: &Attribute) -> bool {
    attr.path().is_ident("doc")
}

fn render_file_with_spacing(file: syn::File) -> String {
    fn trim_trailing_newlines(s: &str) -> &str {
        s.trim_end_matches(['\n', '\r'])
    }

    fn render_items_with_sep(items: &[Item], separator: &str) -> Option<String> {
        let mut rendered_parts = Vec::with_capacity(items.len());
        for item in items {
            let single_item_file = syn::File {
                shebang: None,
                attrs: Vec::new(),
                items: vec![item.clone()],
            };
            let rendered = unparse(&single_item_file);
            let trimmed = trim_trailing_newlines(&rendered);
            if !trimmed.is_empty() {
                rendered_parts.push(trimmed.to_owned());
            }
        }
        if rendered_parts.is_empty() {
            None
        } else {
            Some(rendered_parts.join(separator))
        }
    }

    let syn::File {
        shebang,
        attrs,
        items,
    } = file;

    let (use_items, other_items): (Vec<Item>, Vec<Item>) =
        items.into_iter().partition(|it| matches!(it, Item::Use(_)));

    let mut out = String::new();

    if let Some(s) = shebang {
        out.push_str(s.trim_end_matches('\n'));
        out.push('\n');
    }

    if !attrs.is_empty() {
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        let attrs_file = syn::File {
            shebang: None,
            attrs,
            items: vec![],
        };
        let rendered = unparse(&attrs_file);
        let trimmed = trim_trailing_newlines(&rendered);
        if !trimmed.is_empty() {
            out.push_str(trimmed);
            out.push('\n');
            out.push('\n');
        }
    }

    if let Some(block) = render_items_with_sep(&use_items, "\n") {
        if !out.is_empty() && !out.ends_with('\n') {
            out.push('\n');
        }
        if !block.is_empty() {
            out.push_str(&block);
            out.push('\n');
        }
    }

    if let Some(block) = render_items_with_sep(&other_items, "\n\n") {
        if !out.is_empty() && !out.ends_with("\n\n") {
            if out.ends_with('\n') {
                out.push('\n');
            } else {
                out.push_str("\n\n");
            }
        }
        out.push_str(&block);
        out.push('\n');
    }

    if !out.ends_with('\n') {
        out.push('\n');
    }

    out
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;

    #[test]
    fn merge_preserves_behavior_and_attributes() {
        let old_file = indoc! {r#"
            use sea_orm::entity::prelude::*;
            use crate::helpers::Helper;

            #[derive(Clone, Debug, DeriveEntityModel, serde::Serialize)]
            #[sea_orm(table_name = "posts")]
            #[serde(rename_all = "camelCase")]
            /// Old model docs
            pub struct Model {
                /// Old id docs
                #[serde(rename = "postId")]
                #[cfg(feature = "serde")]
                pub id: i32,
                pub title: String,
            }

            #[derive(DeriveRelation, Eq, PartialEq)]
            #[ts(export)]
            pub enum Relation {
                #[sea_orm(has_one = "super::super::MyEntity")]
                MyRelation,
            }

            impl ActiveModelBehavior for ActiveModel {
                fn custom(&self) -> Helper {
                    Helper::default()
                }
            }
            "#
        };

        let curr = indoc! {r#"
            use sea_orm::entity::prelude::*;

            #[derive(Clone, Debug, DeriveEntityModel)]
            #[sea_orm(schema_name = "public")]
            pub struct Model {
                pub id: i32,
                pub title: String,
            }

            #[derive(DeriveRelation)]
            pub enum Relation {
                MyRelation,
            }

            impl ActiveModelBehavior for ActiveModel {}
            "#
        };

        let merged = merge_entity_files(old_file, curr).expect("merge failed");
        let expected = indoc! {r#"
            use sea_orm::entity::prelude::*;
            use crate::helpers::Helper;

            #[derive(Clone, Debug, DeriveEntityModel, serde::Serialize)]
            #[sea_orm(table_name = "posts", schema_name = "public")]
            #[serde(rename_all = "camelCase")]
            /// Old model docs
            pub struct Model {
                /// Old id docs
                #[serde(rename = "postId")]
                #[cfg(feature = "serde")]
                pub id: i32,
                pub title: String,
            }

            #[derive(DeriveRelation, Eq, PartialEq)]
            #[ts(export)]
            pub enum Relation {
                #[sea_orm(has_one = "super::super::MyEntity")]
                MyRelation,
            }

            impl ActiveModelBehavior for ActiveModel {
                fn custom(&self) -> Helper {
                    Helper::default()
                }
            }
            "#
        };

        assert_eq!(merged, expected);
    }

    #[test]
    fn merge_handles_field_changes() {
        let old_src = indoc! {r#"
            use sea_orm::entity::prelude::*;

            #[derive(DeriveEntityModel)]
            pub struct Model {
                #[serde(default)]
                pub id: i32,
                #[serde(rename = "name")]
                pub name: String,
                #[serde(rename = "foo")]
                pub removed: bool,
            }

            impl ActiveModelBehavior for ActiveModel {}
            "#};

        let new_src = indoc! {r#"
            use sea_orm::entity::prelude::*;

            #[derive(DeriveEntityModel)]
            pub struct Model {
                pub id: i32,
                pub name: String,
                #[serde(rename = "bar")]
                pub added: String,
            }

            impl ActiveModelBehavior for ActiveModel {}
            "#};

        let merged = merge_entity_files(old_src, new_src).expect("merge should succeed");

        let expected = indoc! {r#"
            use sea_orm::entity::prelude::*;

            #[derive(DeriveEntityModel)]
            pub struct Model {
                #[serde(default)]
                pub id: i32,
                #[serde(rename = "name")]
                pub name: String,
                #[serde(rename = "bar")]
                pub added: String,
            }

            impl ActiveModelBehavior for ActiveModel {}
            "#
        };

        assert_eq!(merged, expected);
    }

    #[test]
    fn complex_use() {
        let old_src = indoc! {r#"
            use crate::{
                B::{self, C},
                A,
                D as E,
            };
            use self::{
                helper::Tool as ToolAlias,
                super::shared::Common,
            };

            pub struct Placeholder;
            "#
        };

        let new_src = indoc! {r#"
            use sea_orm::entity::prelude::*;

            pub struct Placeholder;
            "#
        };

        let merged = merge_entity_files(old_src, new_src).expect("merge should succeed");

        let expected = indoc! {r#"
            use sea_orm::entity::prelude::*;
            use crate::{
                B::{self, C},
                A, D as E,
            };
            use self::{helper::Tool as ToolAlias, super::shared::Common};

            pub struct Placeholder;
            "#
        };

        assert_eq!(merged, expected);
    }

    #[test]
    fn conv_to_comment_fallback() {
        let old_src = indoc! {r#"
            this is not valid rust
            impl ActiveModelBehavior for ActiveModel {
                fn something(&self) {}
            }
            "#
        };

        let new_src = indoc! {r#"
            use sea_orm::entity::prelude::*;

            #[derive(DeriveEntityModel)]
            pub struct Model {
                pub id: i32,
            }

            impl ActiveModelBehavior for ActiveModel {}
            "#
        };

        let report = merge_entity_files(old_src, new_src).unwrap_err();
        assert!(report.fallback_applied);

        let expect = indoc! {r#"
            use sea_orm::entity::prelude::*;

            #[derive(DeriveEntityModel)]
            pub struct Model {
                pub id: i32,
            }

            impl ActiveModelBehavior for ActiveModel {}

            // --- Preserved original file content (could not be merged automatically) ---
            /*
            this is not valid rust
            impl ActiveModelBehavior for ActiveModel {
                fn something(&self) {}
            }
            */

        "#};

        assert_eq!(report.output, expect)
    }

    #[test]
    fn conflict_attrs_should_never_be_overwritten() {
        let old_src = indoc! {r#"
            use sea_orm::entity::prelude::*;

            #[derive(DeriveEntityModel)]
            #[serde(rename_all = "camelCase")]
            pub struct Model {
                #[serde(rename_all = "foo")]
                pub id: i32,
            }

            impl ActiveModelBehavior for ActiveModel {}
        "#};

        let new_src = indoc! {r#"
            use sea_orm::entity::prelude::*;

            #[derive(DeriveEntityModel)]
            #[serde(rename_all = "snake_case")]
            pub struct Model {
                #[serde(rename_all = "bar")]
                pub id: i32,
            }

            impl ActiveModelBehavior for ActiveModel {}
        "#};

        let merged = merge_entity_files(old_src, new_src).expect("merge should succeed");
        let expected = indoc! {r#"
            use sea_orm::entity::prelude::*;

            #[derive(DeriveEntityModel)]
            #[serde(rename_all = "camelCase")]
            pub struct Model {
                #[serde(rename_all = "foo")]
                pub id: i32,
            }

            impl ActiveModelBehavior for ActiveModel {}
        "#};

        assert_eq!(merged, expected);
    }

    #[test]
    fn conflict_attrs_should_be_merged() {
        let old_src = indoc! {r#"
            use sea_orm::entity::prelude::*;

            #[derive(DeriveEntityModel)]
            pub struct Model {
                #[serde(rename = "oldId")]
                pub id: i32,
            }

            impl ActiveModelBehavior for ActiveModel {}
        "#};

        let new_src = indoc! {r#"
            use sea_orm::entity::prelude::*;

            #[derive(DeriveEntityModel)]
            pub struct Model {
                #[serde(rename = "newId", default)]
                pub id: i32,
            }

            impl ActiveModelBehavior for ActiveModel {}
        "#};

        let merged = merge_entity_files(old_src, new_src).expect("merge should succeed");
        let expected = indoc! {r#"
            use sea_orm::entity::prelude::*;

            #[derive(DeriveEntityModel)]
            pub struct Model {
                #[serde(rename = "oldId", default)]
                pub id: i32,
            }

            impl ActiveModelBehavior for ActiveModel {}
        "#};

        assert_eq!(merged, expected);
    }
}
