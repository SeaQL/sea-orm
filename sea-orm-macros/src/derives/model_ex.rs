use crate::derives::util::consume_meta;

use super::attributes::compound_attr;
use super::entity_loader::{
    EntityLoaderField, EntityLoaderFieldKind, EntityLoaderSchema, expand_entity_loader,
};
use super::util::{
    CardinalityKind, CompoundKind, CompoundType, Junction, RelationColumns, combine_error,
    is_self_entity,
};
use super::{expand_typed_column, model::DeriveModel};
use heck::ToUpperCamelCase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use std::collections::{BTreeMap, HashMap};
use syn::{
    Attribute, Data, Expr, Field, Fields, ItemStruct, Lit, LitStr, Meta, Type, TypePath,
    Visibility, parse_quote, punctuated::Punctuated, token::Comma,
};

pub fn expand_sea_orm_model(input: ItemStruct, compact: bool) -> syn::Result<TokenStream> {
    let model = input.ident;
    let vis = input.vis;
    let mut all_fields = input.fields;

    let mut model_attrs: Vec<Attribute> = Vec::new();
    let mut model_ex_attrs: Vec<Attribute> = Vec::new();
    let mut has_arrow_schema = false;

    for attr in input.attrs {
        if !attr.path().is_ident("sea_orm") {
            model_attrs.push(attr.clone());
            model_ex_attrs.push(attr);
            continue;
        }

        let mut other_attrs = Punctuated::<Meta, Comma>::new();

        attr.parse_nested_meta(|meta| {
            let is_model = meta.path.is_ident("model_attrs");
            let is_model_ex = meta.path.is_ident("model_ex_attrs");

            if is_model || is_model_ex {
                let content;
                syn::parenthesized!(content in meta.input);
                use syn::parse::Parse;
                let nested_metas = content.parse_terminated(Meta::parse, Comma)?;
                for m in nested_metas {
                    let new_attr: Attribute = parse_quote!( #[#m] );
                    if is_model {
                        model_attrs.push(new_attr);
                    } else {
                        model_ex_attrs.push(new_attr);
                    }
                }
            } else if meta.path.is_ident("arrow_schema") {
                has_arrow_schema = true;
            } else {
                let path = &meta.path;
                if meta.input.peek(syn::Token![=]) {
                    let value: Expr = meta.value()?.parse()?;
                    other_attrs.push(parse_quote!( #path = #value ));
                } else if meta.input.is_empty() || meta.input.peek(Comma) {
                    other_attrs.push(parse_quote!( #path ));
                } else {
                    let content;
                    syn::parenthesized!(content in meta.input);
                    let tokens: TokenStream = content.parse()?;
                    other_attrs.push(parse_quote!( #path(#tokens) ));
                }
            }
            Ok(())
        })?;

        if !other_attrs.is_empty() {
            let attr: Attribute = parse_quote!( #[sea_orm(#other_attrs)] );
            model_attrs.push(attr.clone());
            model_ex_attrs.push(attr);
        }
    }

    if has_arrow_schema {
        model_attrs.push(parse_quote!(#[derive(DeriveArrowSchema)]));
    }

    let model_ex = format_ident!("{model}Ex");

    for attr in &mut model_ex_attrs {
        if !attr.path().is_ident("derive") {
            continue;
        }

        let Meta::List(list) = &mut attr.meta else {
            continue;
        };

        let mut new_list: Punctuated<_, Comma> = Punctuated::new();

        list.parse_nested_meta(|meta| {
            if meta.path.is_ident("Eq") {
                // skip
            } else if meta.path.is_ident("DeriveEntityModel") {
                // replace macro
                new_list.push(parse_quote!(DeriveModelEx));
                new_list.push(parse_quote!(DeriveActiveModelEx));
            } else {
                new_list.push(meta.path);
            }

            Ok(())
        })?;

        *attr = parse_quote!(#[derive( #new_list )]);
    }

    let compact_model = if compact {
        quote!(#[sea_orm(compact_model)])
    } else {
        quote!()
    };

    let mut model_fields = Vec::new();

    for field in &mut all_fields {
        if let Type::Path(type_path) = &field.ty
            && let Some(compound) = CompoundType::from_type(type_path)?
        {
            match compound.kind {
                CompoundKind::BelongsTo(cardinality) => {
                    let entity_type = &compound.entity;
                    field.ty = match cardinality {
                        CardinalityKind::Required => parse_quote!(BelongsTo<#entity_type>),
                        CardinalityKind::Optional => parse_quote!(BelongsTo<Option<#entity_type>>),
                    };
                }
                CompoundKind::HasOne => {
                    let entity_type = &compound.entity;
                    field.ty = parse_quote!(HasOne<#entity_type>);
                }
                CompoundKind::HasMany => {
                    let entity_type = &compound.entity;
                    field.ty = parse_quote!(HasMany<#entity_type>);
                }
            }
        } else {
            model_fields.push(field);
        }
    }

    Ok(quote! {
        #(#model_attrs)*
        #[sea_orm(model_ex)]
        #vis struct #model {
            #(#model_fields),*
        }

        #(#model_ex_attrs)*
        #compact_model
        #vis struct #model_ex #all_fields
    })
}

struct ScalarField<'a> {
    ty: &'a Type,
    unique: bool,
    unique_keys: Vec<Ident>,
}

impl<'a> ScalarField<'a> {
    fn from_field(field: &'a Field) -> syn::Result<Self> {
        let mut unique = false;
        let mut unique_keys = Vec::new();

        for attr in &field.attrs {
            if !attr.path().is_ident("sea_orm") {
                continue;
            }
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("unique") {
                    unique = true;
                }
                if meta.path.is_ident("unique_key") {
                    let lit = meta.value()?.parse()?;
                    if let Lit::Str(litstr) = lit {
                        unique_keys.push(litstr.parse()?);
                    } else {
                        return Err(meta.error(format!("Invalid unique_key {lit:?}")));
                    }
                } else {
                    consume_meta(meta);
                }
                Ok(())
            })?;
        }

        Ok(Self {
            ty: &field.ty,
            unique,
            unique_keys,
        })
    }
}

struct ModelExField<'a> {
    ident: &'a Ident,
    kind: ModelExFieldKind<'a>,
}

#[expect(clippy::large_enum_variant)]
enum ModelExFieldKind<'a> {
    Scalar(ScalarField<'a>),
    Compound(CompoundField),
}

impl<'a> ModelExField<'a> {
    fn from_field(field: &'a Field, compact: bool) -> syn::Result<Self> {
        let Some(ident) = &field.ident else {
            return Err(syn::Error::new_spanned(field, "expected named field"));
        };
        let kind = if let Type::Path(type_path) = &field.ty
            && let Some(compound) = CompoundType::from_type(type_path)?
        {
            ModelExFieldKind::Compound(CompoundField::from_field(field, compound, compact)?)
        } else {
            ModelExFieldKind::Scalar(ScalarField::from_field(field)?)
        };

        Ok(Self { ident, kind })
    }
}

enum RelationVariants {
    Inferred,
    Named(LitStr),
    SelfRef {
        forward_variant: LitStr,
        reverse_variant: Option<LitStr>,
    },
}

impl RelationVariants {
    fn from_attr(
        relation_enum: Option<LitStr>,
        relation_reverse: Option<LitStr>,
        self_ref: Option<()>,
        field: &Field,
    ) -> syn::Result<Self> {
        if self_ref.is_some() {
            Ok(Self::SelfRef {
                forward_variant: relation_enum.ok_or_else(|| {
                    syn::Error::new_spanned(field, "self_ref must specify `relation_enum`")
                })?,
                reverse_variant: relation_reverse,
            })
        } else if let Some(relation_enum) = relation_enum {
            Ok(Self::Named(relation_enum))
        } else {
            Ok(Self::Inferred)
        }
    }

    fn explicit_name(&self) -> Option<&LitStr> {
        match self {
            Self::Named(name)
            | Self::SelfRef {
                forward_variant: name,
                ..
            } => Some(name),
            Self::Inferred => None,
        }
    }

    fn forward_ident(&self, entity: &TypePath) -> Ident {
        if let Some(name) = self.explicit_name() {
            Ident::new(&name.value().to_upper_camel_case(), name.span())
        } else {
            Ident::new(
                &infer_relation_name_from_entity(entity).to_upper_camel_case(),
                Span::call_site(),
            )
        }
    }

    fn reverse_ident(&self, forward: &Ident) -> Option<Ident> {
        if let Self::SelfRef {
            reverse_variant, ..
        } = self
        {
            Some(if let Some(reverse_variant) = reverse_variant {
                Ident::new(&reverse_variant.value(), reverse_variant.span())
            } else {
                Ident::new(&format!("{forward}Reverse"), forward.span())
            })
        } else {
            None
        }
    }

    fn is_self_ref(&self) -> bool {
        matches!(self, Self::SelfRef { .. })
    }
}

#[derive(Default)]
struct ModelExRelationOutput {
    relation_enum_variants: Punctuated<TokenStream, Comma>,
    related_entity_enum_variants: Punctuated<TokenStream, Comma>,
    impl_related_trait: TokenStream,
}

impl ModelExRelationOutput {
    fn from_schema(schema: &ModelExSchema<'_>) -> Self {
        let entity_count = schema
            .fields
            .iter()
            .filter_map(|field| {
                if let ModelExFieldKind::Compound(compound) = &field.kind
                    && compound.relation.is_some()
                {
                    Some(&compound.compound_type.entity)
                } else {
                    None
                }
            })
            .fold(
                HashMap::<&TypePath, usize>::new(),
                |mut entity_count, entity| {
                    *entity_count.entry(entity).or_default() += 1;
                    entity_count
                },
            );

        let mut output = Self::default();
        for (field, relation) in schema.fields.iter().filter_map(|field| {
            if let ModelExFieldKind::Compound(compound) = &field.kind
                && let Some(relation) = &compound.relation
            {
                Some((compound, relation))
            } else {
                None
            }
        }) {
            let entity = &field.compound_type.entity;
            let is_unique_entity = entity_count.get(entity).is_some_and(|count| *count == 1);
            match relation {
                RelationAttr::BelongsTo(attr) => {
                    expand_belongs_to_into(&mut output, entity, attr, is_unique_entity);
                }
                RelationAttr::HasOne(attr) => {
                    expand_has_one_into(&mut output, entity, attr, is_unique_entity);
                }
                RelationAttr::HasMany(attr) => {
                    expand_has_many_into(&mut output, entity, attr, is_unique_entity);
                }
            }
        }
        output
    }
}

enum RelationAttr {
    BelongsTo(BelongsToAttr),
    HasOne(HasOneAttr),
    HasMany(HasManyAttr),
}

impl RelationAttr {
    fn from_attr(
        attrs: compound_attr::SeaOrm,
        field: &Field,
        compound_type: &CompoundType,
    ) -> syn::Result<Self> {
        match &attrs {
            compound_attr::SeaOrm {
                belongs_to: Some(_),
                ..
            }
            | compound_attr::SeaOrm {
                self_ref: Some(_),
                via: None,
                from: Some(_),
                to: Some(_),
                ..
            } => {
                let attr = BelongsToAttr::from_attr(attrs, field)?;
                if !matches!(
                    compound_type.kind,
                    CompoundKind::BelongsTo(_) | CompoundKind::HasOne
                ) {
                    return Err(syn::Error::new_spanned(
                        &field.ty,
                        "belongs_to must be paired with BelongsTo or HasOne",
                    ));
                }
                Ok(Self::BelongsTo(attr))
            }
            compound_attr::SeaOrm {
                has_one: Some(_), ..
            } => {
                let attr = HasOneAttr::from_attr(attrs, field)?;
                if compound_type.kind != CompoundKind::HasOne {
                    return Err(syn::Error::new_spanned(
                        &field.ty,
                        "has_one must be paired with HasOne",
                    ));
                }
                Ok(Self::HasOne(attr))
            }
            compound_attr::SeaOrm {
                has_many: Some(_), ..
            }
            | compound_attr::SeaOrm {
                self_ref: Some(_),
                via: Some(_),
                ..
            } => {
                let attr =
                    HasManyAttr::from_attr(attrs, field, is_self_entity(&compound_type.entity))?;
                if compound_type.kind != CompoundKind::HasMany {
                    return Err(syn::Error::new_spanned(
                        &field.ty,
                        "has_many must be paired with HasMany",
                    ));
                }
                Ok(Self::HasMany(attr))
            }
            _ => match compound_type.kind {
                CompoundKind::BelongsTo(_) => {
                    Ok(Self::BelongsTo(BelongsToAttr::from_attr(attrs, field)?))
                }
                CompoundKind::HasOne => Ok(Self::HasOne(HasOneAttr::from_attr(attrs, field)?)),
                CompoundKind::HasMany => Ok(Self::HasMany(HasManyAttr::from_attr(
                    attrs,
                    field,
                    is_self_entity(&compound_type.entity),
                )?)),
            },
        }
    }
}

struct CompoundField {
    compound_type: CompoundType,
    relation: Option<RelationAttr>,
}

impl CompoundField {
    fn from_field(field: &Field, compound: CompoundType, compact: bool) -> syn::Result<Self> {
        let attrs = compound_attr::SeaOrm::try_from_attributes(&field.attrs)?;
        if compact
            && attrs.as_ref().is_some_and(|attrs| {
                attrs.has_one.is_some() || attrs.has_many.is_some() || attrs.belongs_to.is_some()
            })
        {
            return Err(syn::Error::new_spanned(
                field,
                "You cannot use #[has_one / has_many / belongs_to] on #[sea_orm::compact_model], please use #[sea_orm::model] instead.",
            ));
        }

        let relation = attrs
            .map(|attrs| RelationAttr::from_attr(attrs, field, &compound))
            .transpose()?;
        Ok(Self {
            compound_type: compound,
            relation,
        })
    }
}

fn entity_loader_field(field: &Ident, compound: &CompoundField) -> EntityLoaderField {
    let entity = &compound.compound_type.entity;
    let self_entity = is_self_entity(entity);
    let (relation_enum, kind) = match &compound.relation {
        Some(RelationAttr::BelongsTo(attr)) => (
            attr.relation_variants.explicit_name().cloned(),
            if self_entity {
                EntityLoaderFieldKind::HasOneSelf
            } else {
                EntityLoaderFieldKind::HasOne
            },
        ),
        Some(RelationAttr::HasOne(attr)) => (
            attr.relation_variants.explicit_name().cloned(),
            if self_entity {
                EntityLoaderFieldKind::HasOneSelf
            } else {
                EntityLoaderFieldKind::HasOne
            },
        ),
        Some(RelationAttr::HasMany(attr)) => {
            let (relation_enum, junction_module, reverse) = match attr {
                HasManyAttr::Standard {
                    relation_variants,
                    via,
                    reverse,
                    ..
                } => (
                    relation_variants.explicit_name().cloned(),
                    via.as_ref().map(|junction| junction.module.clone()),
                    *reverse,
                ),
                HasManyAttr::ManyToManySelf {
                    relation_enum,
                    junction_module,
                    direction,
                } => (
                    relation_enum.clone(),
                    Some(junction_module.clone()),
                    matches!(direction, ManyToManySelfDirection::Reverse),
                ),
            };
            let kind = if self_entity {
                if let Some(junction_module) = junction_module {
                    EntityLoaderFieldKind::ManyToManySelf {
                        junction_module,
                        reverse,
                    }
                } else {
                    EntityLoaderFieldKind::HasManySelf
                }
            } else if junction_module.is_some() {
                EntityLoaderFieldKind::ManyToMany
            } else {
                EntityLoaderFieldKind::HasMany
            };
            (relation_enum, kind)
        }
        None => (
            None,
            match compound.compound_type.kind {
                CompoundKind::BelongsTo(_) | CompoundKind::HasOne if self_entity => {
                    EntityLoaderFieldKind::HasOneSelf
                }
                CompoundKind::BelongsTo(_) | CompoundKind::HasOne => EntityLoaderFieldKind::HasOne,
                CompoundKind::HasMany if self_entity => EntityLoaderFieldKind::HasManySelf,
                CompoundKind::HasMany => EntityLoaderFieldKind::HasMany,
            },
        ),
    };
    EntityLoaderField {
        field: field.clone(),
        entity: entity.clone(),
        relation_enum,
        kind,
    }
}

impl EntityLoaderSchema {
    fn from_model_ex_schema(schema: &ModelExSchema<'_>) -> Self {
        Self {
            fields: schema
                .fields
                .iter()
                .filter_map(|field| {
                    let ModelExFieldKind::Compound(compound) = &field.kind else {
                        return None;
                    };
                    Some(entity_loader_field(field.ident, compound))
                })
                .collect(),
        }
    }
}

fn expand_related_entity_variants_into(
    entity: &TypePath,
    relation_variants: &RelationVariants,
    output: &mut ModelExRelationOutput,
) {
    let relation_enum = relation_variants.forward_ident(entity);
    let related_entity_lit = entity
        .path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::");
    let extra = if relation_variants.explicit_name().is_some() {
        let relation_def = format!("Relation::{relation_enum}.def()");
        quote!(, def = #relation_def)
    } else {
        quote!()
    };
    output.related_entity_enum_variants.push(quote! {
        #[doc = " Generated by sea-orm-macros"]
        #[sea_orm(entity = #related_entity_lit #extra)]
        #relation_enum
    });

    if let Some(relation_enum_ref) = relation_variants.reverse_ident(&relation_enum) {
        let relation_def = format!("Relation::{relation_enum}.def().rev()");
        output.related_entity_enum_variants.push(quote! {
            #[doc = " Generated by sea-orm-macros"]
            #[sea_orm(entity = #related_entity_lit def = #relation_def)]
            #relation_enum_ref
        });
    }
}

fn expand_related_trait(entity: &TypePath, relation_variants: &RelationVariants) -> TokenStream {
    let relation_enum = relation_variants.forward_ident(entity);
    quote! {
        #[doc = " Generated by sea-orm-macros"]
        impl Related<#entity> for Entity {
            fn to() -> RelationDef {
                Relation::#relation_enum.def()
            }
        }
    }
}

fn expand_related_via_trait(
    entity: &TypePath,
    relation_variants: &RelationVariants,
    junction: &Junction,
) -> TokenStream {
    let relation_enum = relation_variants.forward_ident(entity);
    let module = &junction.module;
    let relation_def = quote!(super::#module::Relation::#relation_enum.def());
    let via_relation_def = if let Some(relation) = &junction.relation {
        quote!(super::#module::Relation::#relation.def().rev())
    } else {
        quote!(<super::#module::Entity as Related<Entity>>::to().rev())
    };
    quote! {
        #[doc = " Generated by sea-orm-macros"]
        impl Related<#entity> for Entity {
            fn to() -> RelationDef {
                #relation_def
            }
            fn via() -> Option<RelationDef> {
                Some(#via_relation_def)
            }
        }
    }
}

fn expand_related_into(
    output: &mut ModelExRelationOutput,
    entity: &TypePath,
    relation_variants: &RelationVariants,
    via: Option<&Junction>,
    impl_related: bool,
) {
    expand_related_entity_variants_into(entity, relation_variants, output);
    if impl_related {
        output.impl_related_trait.extend(if let Some(via) = via {
            expand_related_via_trait(entity, relation_variants, via)
        } else {
            expand_related_trait(entity, relation_variants)
        });
    }
}

struct BelongsToRelationAttr {
    declared: bool,
    via: Option<Junction>,
    from: RelationColumns,
    to: RelationColumns,
    on_update: Option<LitStr>,
    on_delete: Option<LitStr>,
    skip_fk: bool,
}

struct BelongsToAttr {
    relation_variants: RelationVariants,
    relation: Option<BelongsToRelationAttr>,
}

impl BelongsToAttr {
    fn from_attr(attrs: compound_attr::SeaOrm, field: &Field) -> syn::Result<Self> {
        let compound_attr::SeaOrm {
            has_one,
            has_many,
            belongs_to,
            self_ref,
            skip_fk,
            via,
            from,
            to,
            relation_enum,
            relation_reverse,
            on_update,
            on_delete,
            ..
        } = attrs;

        let is_self_ref = self_ref.is_some();
        let mut error = None;

        if has_one.is_some() {
            combine_error(
                &mut error,
                syn::Error::new_spanned(&field.ty, "has_one must be paired with HasOne"),
            );
        }
        if has_many.is_some() {
            combine_error(
                &mut error,
                syn::Error::new_spanned(&field.ty, "has_many must be paired with HasMany"),
            );
        }
        if is_self_ref && via.is_some() {
            combine_error(
                &mut error,
                syn::Error::new_spanned(
                    &field.ty,
                    "self_ref + via field type must be `HasMany<Entity>`",
                ),
            );
        }

        let relation_variants = if is_self_ref && via.is_some() {
            None
        } else {
            match RelationVariants::from_attr(relation_enum, relation_reverse, self_ref, field) {
                Ok(relation_variants) => Some(relation_variants),
                Err(err) => {
                    combine_error(&mut error, err);
                    None
                }
            }
        };

        let from = if belongs_to.is_some() && from.is_none() {
            combine_error(
                &mut error,
                syn::Error::new_spanned(
                    field,
                    "#[sea_orm(belongs_to)] must include `from = \"...\"` to name the local foreign key column",
                ),
            );
            None
        } else {
            from
        };
        let to = if belongs_to.is_some() && to.is_none() {
            combine_error(
                &mut error,
                syn::Error::new_spanned(
                    field,
                    "#[sea_orm(belongs_to)] must include `to = \"...\"` to name the related primary key column",
                ),
            );
            None
        } else {
            to
        };

        let from = match from.map(RelationColumns::from_lit).transpose() {
            Ok(from) => from,
            Err(err) => {
                combine_error(&mut error, err);
                None
            }
        };
        let to = match to.map(RelationColumns::from_lit).transpose() {
            Ok(to) => to,
            Err(err) => {
                combine_error(&mut error, err);
                None
            }
        };
        let via = match via.map(|via| Junction::from_lit(&via)).transpose() {
            Ok(via) => via,
            Err(err) => {
                combine_error(&mut error, err);
                None
            }
        };

        if let Some(err) = error {
            return Err(err);
        }

        let relation_variants = relation_variants.expect("validated");
        let relation = if belongs_to.is_some() {
            Some(BelongsToRelationAttr {
                declared: true,
                via,
                from: from.expect("validated"),
                to: to.expect("validated"),
                on_update,
                on_delete,
                skip_fk: skip_fk.is_some(),
            })
        } else if relation_variants.is_self_ref()
            && let (Some(from), Some(to)) = (from, to)
        {
            Some(BelongsToRelationAttr {
                declared: false,
                via: None,
                from,
                to,
                on_update,
                on_delete,
                skip_fk: skip_fk.is_some(),
            })
        } else {
            None
        };
        Ok(Self {
            relation_variants,
            relation,
        })
    }
}

fn expand_belongs_to_into(
    output: &mut ModelExRelationOutput,
    entity: &TypePath,
    attr: &BelongsToAttr,
    unique_entity: bool,
) {
    let relation_variants = &attr.relation_variants;
    if let Some(relation) = &attr.relation {
        let relation_enum = relation_variants.forward_ident(entity);
        let related_entity_lit = entity
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");
        let belongs_to = Ident::new("belongs_to", Span::call_site());
        let format_columns = |columns: &RelationColumns, prefix: &str| {
            let columns = columns
                .columns
                .iter()
                .map(|column| {
                    if prefix.is_empty() {
                        format!("Column::{column}")
                    } else {
                        format!("{prefix}::Column::{column}")
                    }
                })
                .collect::<Vec<_>>();
            if columns.len() > 1 {
                format!("({})", columns.join(", "))
            } else {
                columns[0].clone()
            }
        };
        let from = format_columns(&relation.from, "");
        let (related_entity, to) = if relation_variants.is_self_ref() {
            ("Entity", format_columns(&relation.to, ""))
        } else {
            (
                related_entity_lit.as_str(),
                format_columns(
                    &relation.to,
                    related_entity_lit.trim_end_matches("::Entity"),
                ),
            )
        };
        let mut extra: Punctuated<_, Comma> = Punctuated::new();
        if let Some(on_update) = &relation.on_update {
            let tag = Ident::new("on_update", on_update.span());
            extra.push(quote!(#tag = #on_update));
        }
        if let Some(on_delete) = &relation.on_delete {
            let tag = Ident::new("on_delete", on_delete.span());
            extra.push(quote!(#tag = #on_delete));
        }
        if relation.skip_fk {
            extra.push(quote!(skip_fk));
        }
        output.relation_enum_variants.push(quote! {
            #[doc = " Generated by sea-orm-macros"]
            #[sea_orm(#belongs_to = #related_entity, from = #from, to = #to, #extra)]
            #relation_enum
        });
    }

    let (declared, via) = attr
        .relation
        .as_ref()
        .map(|relation| (relation.declared, relation.via.as_ref()))
        .unwrap_or_default();
    expand_related_into(
        output,
        entity,
        relation_variants,
        via,
        unique_entity && declared,
    );
}

struct HasOneAttr {
    relation_variants: RelationVariants,
    /// Whether `#[sea_orm(has_one)]` is present.
    has_one: bool,
    via: Option<Junction>,
    via_rel: Option<LitStr>,
}

impl HasOneAttr {
    fn from_attr(attrs: compound_attr::SeaOrm, field: &Field) -> syn::Result<Self> {
        let compound_attr::SeaOrm {
            has_one,
            has_many,
            belongs_to,
            self_ref,
            via,
            via_rel,
            from,
            to,
            relation_enum,
            relation_reverse,
            ..
        } = attrs;

        let is_self_ref = self_ref.is_some();
        let mut error = None;

        if belongs_to.is_some() {
            combine_error(
                &mut error,
                syn::Error::new_spanned(&field.ty, "belongs_to must be paired with BelongsTo"),
            );
        }
        if has_many.is_some() {
            combine_error(
                &mut error,
                syn::Error::new_spanned(&field.ty, "has_many must be paired with HasMany"),
            );
        }
        if is_self_ref && via.is_some() {
            combine_error(
                &mut error,
                syn::Error::new_spanned(
                    &field.ty,
                    "self_ref + via field type must be `HasMany<Entity>`",
                ),
            );
        }

        let relation_variants = if is_self_ref && via.is_some() {
            None
        } else {
            match RelationVariants::from_attr(relation_enum, relation_reverse, self_ref, field) {
                Ok(relation_variants) => Some(relation_variants),
                Err(err) => {
                    combine_error(&mut error, err);
                    None
                }
            }
        };
        if is_self_ref && from.is_some() && to.is_some() {
            combine_error(
                &mut error,
                syn::Error::new_spanned(
                    &field.ty,
                    "self_ref belongs_to must be paired with BelongsTo",
                ),
            );
        }
        let via = match via.map(|via| Junction::from_lit(&via)).transpose() {
            Ok(via) => via,
            Err(err) => {
                combine_error(&mut error, err);
                None
            }
        };

        if let Some(err) = error {
            return Err(err);
        }

        let relation_variants = relation_variants.expect("validated relation variants");
        Ok(Self {
            relation_variants,
            has_one: has_one.is_some(),
            via,
            via_rel,
        })
    }
}

fn expand_has_one_into(
    output: &mut ModelExRelationOutput,
    entity: &TypePath,
    attr: &HasOneAttr,
    unique_entity: bool,
) {
    let relation_variants = &attr.relation_variants;
    if attr.has_one {
        let relation_enum = relation_variants.forward_ident(entity);
        let related_entity_lit = entity
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");
        let has_one = Ident::new("has_one", Span::call_site());
        let mut extra: Punctuated<_, Comma> = Punctuated::new();
        if let Some(via_rel) = &attr.via_rel {
            let tag = Ident::new("via_rel", via_rel.span());
            let via_rel = format!("Relation::{}", via_rel.value());
            extra.push(quote!(#tag = #via_rel));
        }
        output.relation_enum_variants.push(quote! {
            #[doc = " Generated by sea-orm-macros"]
            #[sea_orm(#has_one = #related_entity_lit, #extra)]
            #relation_enum
        });
    }

    expand_related_into(
        output,
        entity,
        relation_variants,
        attr.via.as_ref(),
        unique_entity && attr.has_one,
    );
}

enum HasManyAttr {
    Standard {
        relation_variants: RelationVariants,
        has_many: bool,
        via: Option<Junction>,
        via_rel: Option<LitStr>,
        reverse: bool,
    },
    ManyToManySelf {
        relation_enum: Option<LitStr>,
        junction_module: Ident,
        direction: ManyToManySelfDirection,
    },
}

enum ManyToManySelfDirection {
    Forward { from: LitStr, to: LitStr },
    Reverse,
    Manual,
}

impl HasManyAttr {
    fn from_attr(
        attrs: compound_attr::SeaOrm,
        field: &Field,
        is_self_entity: bool,
    ) -> syn::Result<Self> {
        let compound_attr::SeaOrm {
            has_one,
            has_many,
            belongs_to,
            self_ref,
            via,
            via_rel,
            from,
            to,
            relation_enum,
            relation_reverse,
            reverse,
            ..
        } = attrs;

        let is_self_ref = self_ref.is_some();
        let mut error = None;

        if belongs_to.is_some() {
            combine_error(
                &mut error,
                syn::Error::new_spanned(&field.ty, "belongs_to must be paired with BelongsTo"),
            );
        }
        if has_one.is_some() {
            combine_error(
                &mut error,
                syn::Error::new_spanned(&field.ty, "has_one must be paired with HasOne"),
            );
        }
        let junction = match via.as_ref().map(Junction::from_lit).transpose() {
            Ok(junction) => junction,
            Err(err) => {
                combine_error(&mut error, err);
                None
            }
        };
        if is_self_ref && let Some(via) = &via {
            if !is_self_entity {
                combine_error(
                    &mut error,
                    syn::Error::new_spanned(
                        &field.ty,
                        "self_ref + via field type must be `HasMany<Entity>`",
                    ),
                );
            }

            if junction
                .as_ref()
                .is_some_and(|junction| junction.relation.is_some())
            {
                combine_error(
                    &mut error,
                    syn::Error::new(via.span(), "`self_ref` via must name a junction entity"),
                );
            }
            if let Some(err) = error {
                return Err(err);
            }

            let direction = if reverse.is_some() {
                ManyToManySelfDirection::Reverse
            } else if let (Some(from), Some(to)) = (from, to) {
                ManyToManySelfDirection::Forward { from, to }
            } else {
                ManyToManySelfDirection::Manual
            };
            return Ok(Self::ManyToManySelf {
                relation_enum,
                junction_module: junction.expect("validated junction").module,
                direction,
            });
        }

        let relation_variants =
            match RelationVariants::from_attr(relation_enum, relation_reverse, self_ref, field) {
                Ok(relation_variants) => Some(relation_variants),
                Err(err) => {
                    combine_error(&mut error, err);
                    None
                }
            };
        if is_self_ref && from.is_some() && to.is_some() {
            combine_error(
                &mut error,
                syn::Error::new_spanned(
                    &field.ty,
                    "self_ref belongs_to must be paired with BelongsTo",
                ),
            );
        }

        if let Some(err) = error {
            return Err(err);
        }
        let relation_variants = relation_variants.expect("validated relation variants");
        Ok(Self::Standard {
            relation_variants,
            has_many: has_many.is_some(),
            via: junction,
            via_rel,
            reverse: has_many.is_none() && reverse.is_some(),
        })
    }
}

fn expand_has_many_into(
    output: &mut ModelExRelationOutput,
    entity: &TypePath,
    attr: &HasManyAttr,
    unique_entity: bool,
) {
    let (relation_variants, has_many, via, via_rel) = match attr {
        HasManyAttr::Standard {
            relation_variants,
            has_many,
            via,
            via_rel,
            ..
        } => (relation_variants, *has_many, via, via_rel),
        HasManyAttr::ManyToManySelf {
            junction_module,
            direction,
            ..
        } => {
            output
                .impl_related_trait
                .extend(expand_impl_related_many_to_many_self(
                    junction_module,
                    direction,
                ));
            return;
        }
    };
    let relation_enum = relation_variants.forward_ident(entity);
    if let RelationVariants::SelfRef {
        reverse_variant: Some(relation_reverse),
        ..
    } = relation_variants
    {
        let has_many = Ident::new("has_many", Span::call_site());
        let via_rel = format!("Relation::{}", relation_reverse.value());
        output.relation_enum_variants.push(quote! {
            #[doc = " Generated by sea-orm-macros"]
            #[sea_orm(#has_many = "Entity", via_rel = #via_rel)]
            #relation_enum
        });
    } else if has_many && via.is_none() {
        let related_entity_lit = entity
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>()
            .join("::");
        let has_many = Ident::new("has_many", Span::call_site());
        let mut extra: Punctuated<_, Comma> = Punctuated::new();
        if let Some(via_rel) = via_rel {
            let tag = Ident::new("via_rel", via_rel.span());
            let via_rel = format!("Relation::{}", via_rel.value());
            extra.push(quote!(#tag = #via_rel));
        }
        output.relation_enum_variants.push(quote! {
            #[doc = " Generated by sea-orm-macros"]
            #[sea_orm(#has_many = #related_entity_lit, #extra)]
            #relation_enum
        });
    }

    if relation_variants.is_self_ref() {
        return;
    }
    expand_related_into(
        output,
        entity,
        relation_variants,
        via.as_ref(),
        unique_entity && has_many,
    );
}

struct ModelExSchema<'a> {
    compact: bool,
    fields: Vec<ModelExField<'a>>,
}

impl<'a> ModelExSchema<'a> {
    fn from_data(data: &'a Data, attrs: &[Attribute], ident: &Ident) -> syn::Result<Self> {
        let mut compact = false;
        attrs
            .iter()
            .filter(|attr| attr.path().is_ident("sea_orm"))
            .try_for_each(|attr| {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("compact_model") {
                        compact = true;
                    } else {
                        consume_meta(meta);
                    }
                    Ok(())
                })
            })?;

        let fields = if let Data::Struct(r#struct) = data
            && let Fields::Named(fields) = &r#struct.fields
        {
            fields
                .named
                .iter()
                .map(|field| ModelExField::from_field(field, compact))
                .collect::<syn::Result<Vec<_>>>()?
        } else {
            return Err(syn::Error::new_spanned(
                ident,
                "You can only derive DeriveModelEx on structs",
            ));
        };

        Ok(Self { compact, fields })
    }
}

pub fn expand_derive_model_ex(
    vis: &Visibility,
    ident: Ident,
    data: Data,
    attrs: Vec<Attribute>,
) -> syn::Result<TokenStream> {
    let schema = ModelExSchema::from_data(&data, &attrs, &ident)?;
    let compact = schema.compact;
    let model_fields = schema
        .fields
        .iter()
        .filter_map(|field| {
            matches!(&field.kind, ModelExFieldKind::Scalar(_)).then_some(field.ident)
        })
        .collect::<Vec<_>>();
    let compound_fields = schema
        .fields
        .iter()
        .filter_map(|field| {
            matches!(&field.kind, ModelExFieldKind::Compound(_)).then_some(field.ident)
        })
        .collect::<Vec<_>>();

    let impl_model_trait = DeriveModel::new(&ident, &data, &attrs)?.impl_model_trait();

    let impl_from_model = quote! {
        impl Model {
            #[doc = " Generated by sea-orm-macros"]
            pub fn into_ex(self) -> ModelEx {
                self.into()
            }
        }

        #[automatically_derived]
        impl std::convert::From<Model> for ModelEx {
            fn from(m: Model) -> Self {
                Self {
                    #(#model_fields: m.#model_fields,)*
                    #(#compound_fields: Default::default(),)*
                }
            }
        }

        #[automatically_derived]
        impl std::convert::From<ModelEx> for Model {
            fn from(m: ModelEx) -> Self {
                Self {
                    #(#model_fields: m.#model_fields,)*
                }
            }
        }

        #[automatically_derived]
        impl PartialEq<ModelEx> for Model {
            fn eq(&self, other: &ModelEx) -> bool {
                true #(&& self.#model_fields == other.#model_fields)*
            }
        }

        #[automatically_derived]
        impl PartialEq<Model> for ModelEx {
            fn eq(&self, other: &Model) -> bool {
                true #(&& self.#model_fields == other.#model_fields)*
            }
        }
    };

    let ModelExRelationOutput {
        relation_enum_variants,
        related_entity_enum_variants,
        impl_related_trait,
    } = ModelExRelationOutput::from_schema(&schema);

    let relation_enum = if !compact {
        quote! {
            #[doc = " Generated by sea-orm-macros"]
            #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
            #vis enum Relation {
                #relation_enum_variants
            }
        }
    } else {
        // for backwards compatibility with compact models
        quote!()
    };

    let related_entity_enum = if !compact {
        quote! {
            #[doc = " Generated by sea-orm-macros"]
            #[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
            #vis enum RelatedEntity {
                #related_entity_enum_variants
            }
        }
    } else {
        // for backwards compatibility with compact models
        quote!()
    };

    let (typed_column, typed_column_const) = expand_typed_column(vis, &data)?;

    let (entity_find_by_key, loader_filter_by_key) = expand_find_by_unique_key(&schema);

    let entity_loader =
        expand_entity_loader(vis, EntityLoaderSchema::from_model_ex_schema(&schema));

    Ok(quote! {
        #typed_column

        #typed_column_const

        #impl_from_model

        #impl_model_trait

        #relation_enum

        #impl_related_trait

        #related_entity_enum

        #entity_loader

        impl Entity {
            #[doc = " Generated by sea-orm-macros"]
            pub const COLUMN: TypedColumn = COLUMN;

            #entity_find_by_key
        }

        impl EntityLoader {
            #loader_filter_by_key
        }
    })
}

fn expand_impl_related_many_to_many_self(
    junction_module: &Ident,
    direction: &ManyToManySelfDirection,
) -> TokenStream {
    match direction {
        ManyToManySelfDirection::Forward { from, to } => {
            let from = Ident::new(&from.value(), from.span());
            let to = Ident::new(&to.value(), to.span());

            quote! {
                #[doc = " Generated by sea-orm-macros"]
                impl RelatedSelfVia<super::#junction_module::Entity> for Entity {
                    fn to() -> RelationDef {
                        super::#junction_module::Relation::#to.def()
                    }
                    fn via() -> RelationDef {
                        super::#junction_module::Relation::#from.def().rev()
                    }
                }
            }

            // #[sea_orm(self_ref, via = "user_follower", from = "User", to = "Follower")]
            // impl RelatedSelfVia<super::user_follower::Entity> for Entity {
            //     fn to() -> RelationDef {
            //         super::user_follower::Relation::Follower.def()
            //     }

            //     fn via() -> RelationDef {
            //         super::user_follower::Relation::User.def().rev()
            //     }
            // }
        }
        ManyToManySelfDirection::Reverse | ManyToManySelfDirection::Manual => quote!(),
    }
}

pub(crate) fn infer_relation_name_from_entity(entity: &TypePath) -> String {
    let mut segments = entity.path.segments.iter().rev();
    let Some(last) = segments.next() else {
        return String::new();
    };
    segments
        .next()
        .map(|segment| segment.ident.to_string())
        .unwrap_or_else(|| last.ident.to_string())
}

fn expand_find_by_unique_key(schema: &ModelExSchema<'_>) -> (TokenStream, TokenStream) {
    let mut unique_keys = BTreeMap::<Ident, Vec<(Ident, Type)>>::new();
    for field in &schema.fields {
        let ModelExFieldKind::Scalar(scalar) = &field.kind else {
            continue;
        };
        if scalar.unique {
            unique_keys.insert(
                field.ident.clone(),
                vec![(field.ident.clone(), scalar.ty.clone())],
            );
        }
        for unique_key in &scalar.unique_keys {
            unique_keys
                .entry(unique_key.clone())
                .or_default()
                .push((field.ident.clone(), scalar.ty.clone()));
        }
    }

    let mut entity_find_by_key = TokenStream::new();
    let mut loader_filter_by_key = TokenStream::new();

    for (name, columns) in unique_keys {
        let find_method = format_ident!("find_by_{}", name);
        let filter_method = format_ident!("filter_by_{}", name);
        let delete_method = format_ident!("delete_by_{}", name);
        if columns.len() > 1 {
            let key_type = columns.iter().map(|(_, ty)| ty).collect::<Vec<_>>();

            let filters = columns
                .iter()
                .enumerate()
                .map(|(i, (col, _))| {
                    let i = syn::Index::from(i);
                    let col = Ident::new(&col.to_string().to_upper_camel_case(), col.span());
                    quote!(Column::#col.eq(v.#i))
                })
                .collect::<Vec<_>>();

            entity_find_by_key.extend(quote! {
                #[doc = " Generated by sea-orm-macros"]
                pub fn #find_method(v: (#(#key_type),*)) -> Select<Entity> {
                    Self::find()
                        #(.filter(#filters))*
                }

                #[doc = " Generated by sea-orm-macros"]
                pub fn #delete_method(v: (#(#key_type),*)) -> sea_orm::ValidatedDeleteOne<Entity> {
                    sea_orm::Delete::_one_only_for_use_by_model_ex(Entity)
                        #(.filter(#filters))*
                }
            });
            loader_filter_by_key.extend(quote! {
                #[doc = " Generated by sea-orm-macros"]
                pub fn #filter_method(mut self, v: (#(#key_type),*)) -> Self {
                    #(self.filter_mut(#filters);)*
                    self
                }
            });
        } else {
            let col = &columns[0].0;
            let col = Ident::new(&col.to_string().to_upper_camel_case(), col.span());
            let key_type = &columns[0].1;
            entity_find_by_key.extend(quote! {
                #[doc = " Generated by sea-orm-macros"]
                pub fn #find_method(v: impl Into<#key_type>) -> Select<Entity> {
                    Self::find().filter(Column::#col.eq(v.into()))
                }

                #[doc = " Generated by sea-orm-macros"]
                pub fn #delete_method(v: impl Into<#key_type>) -> sea_orm::ValidatedDeleteOne<Entity> {
                    sea_orm::Delete::_one_only_for_use_by_model_ex(Entity)
                        .filter(Column::#col.eq(v.into()))
                }
            });
            loader_filter_by_key.extend(quote! {
                #[doc = " Generated by sea-orm-macros"]
                pub fn #filter_method(mut self, v: impl Into<#key_type>) -> Self {
                    self.filter_mut(Column::#col.eq(v.into()));
                    self
                }
            });
        }
    }

    (entity_find_by_key, loader_filter_by_key)
}
