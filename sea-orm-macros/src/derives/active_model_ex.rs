use super::active_model::DeriveActiveModel;
use super::attributes::compound_attr;
use super::model_ex::infer_relation_name_from_entity;
use super::util::{
    CardinalityKind, CompoundKind, CompoundType, async_token, await_token, consume_meta,
    escape_rust_keyword, trim_starting_raw_identifier,
};
use heck::ToUpperCamelCase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    Attribute, Data, LitStr, Path, PathArguments, Type, TypePath, Visibility, parse::Parser,
    punctuated::Punctuated, token::Comma,
};

enum RelationAttr {
    BelongsTo {
        cardinality: CardinalityKind,
        from: LitStr,
        /// Explicit relation disambiguator, present when more than one relation
        /// targets the same entity. Routes the FK write through the
        /// relation-keyed `*_parent_key_for` helpers instead of the entity-keyed ones.
        relation_enum: Option<LitStr>,
    },
    BelongsToSelf {
        relation_enum: LitStr,
        cardinality: CardinalityKind,
        from: LitStr,
    },
    HasOne {
        cardinality: CardinalityKind,
    },
    HasMany,
    HasManySelf {
        relation_enum: LitStr,
    },
    HasManyVia {
        via: LitStr,
    },
    HasManyViaSelf {
        via: LitStr,
        reverse: bool,
    },
}

impl RelationAttr {
    fn from_attr(
        attrs: &compound_attr::SeaOrm,
        ident: &Ident,
        compound: &CompoundType,
    ) -> syn::Result<Option<Self>> {
        match attrs {
            compound_attr::SeaOrm {
                self_ref: Some(_),
                via: Some(via),
                ..
            } => {
                if compound.kind != CompoundKind::HasMany || !compound.is_self_entity() {
                    return Err(syn::Error::new_spanned(
                        via,
                        "self_ref + via field type must be `HasMany<Entity>`",
                    ));
                }
                Ok(Some(Self::HasManyViaSelf {
                    via: via.clone(),
                    reverse: attrs.reverse.is_some(),
                }))
            }
            compound_attr::SeaOrm {
                relation_enum: Some(relation_enum),
                self_ref: Some(_),
                via: None,
                from: Some(from),
                to: Some(_),
                ..
            } => Ok(Some(Self::BelongsToSelf {
                relation_enum: relation_enum.clone(),
                cardinality: compound.cardinality().ok_or_else(|| {
                    syn::Error::new_spanned(ident, "self_ref belongs_to must be paired with HasOne")
                })?,
                from: from.clone(),
            })),
            compound_attr::SeaOrm {
                relation_enum: Some(relation_enum),
                self_ref: Some(_),
                relation_reverse: Some(_),
                via: None,
                from: None,
                to: None,
                ..
            } => {
                if compound.kind != CompoundKind::HasMany || !compound.is_self_entity() {
                    return Err(syn::Error::new_spanned(
                        ident,
                        "self_ref has_many field type must be `HasMany<Entity>`",
                    ));
                }
                Ok(Some(Self::HasManySelf {
                    relation_enum: relation_enum.clone(),
                }))
            }
            compound_attr::SeaOrm {
                self_ref: Some(_),
                via: None,
                relation_enum: None,
                ..
            } => Err(syn::Error::new_spanned(
                ident,
                "Please specify `relation_enum` for `self_ref`",
            )),
            compound_attr::SeaOrm {
                self_ref: Some(_), ..
            } => Ok(None),
            compound_attr::SeaOrm {
                belongs_to: Some(_),
                ..
            } => Ok(Some(Self::BelongsTo {
                cardinality: compound.cardinality().ok_or_else(|| {
                    syn::Error::new_spanned(ident, "belongs_to must be paired with HasOne")
                })?,
                from: attrs.from.clone().ok_or_else(|| {
                    syn::Error::new_spanned(ident, "belongs_to must specify `from`")
                })?,
                // Captured so multiple belongs_to targeting the same entity can be
                // disambiguated via their relation enum on save.
                relation_enum: attrs.relation_enum.clone(),
            })),
            compound_attr::SeaOrm {
                relation_enum: Some(_),
                ..
            } => Ok(None),
            compound_attr::SeaOrm {
                has_one: Some(_), ..
            } => {
                let cardinality = match compound.kind {
                    CompoundKind::HasOne(CardinalityKind::Optional) => CardinalityKind::Optional,
                    CompoundKind::HasOne(CardinalityKind::Required) => {
                        return Err(syn::Error::new_spanned(
                            ident,
                            "#[sea_orm(has_one)] must be paired with HasOne<Option<Entity>>",
                        ));
                    }
                    CompoundKind::HasMany => {
                        return Err(syn::Error::new_spanned(
                            ident,
                            "#[sea_orm(has_one)] must be paired with HasOne<Option<Entity>>",
                        ));
                    }
                };
                Ok(Some(Self::HasOne { cardinality }))
            }
            compound_attr::SeaOrm {
                has_many: Some(_),
                via: None,
                ..
            } => {
                if compound.kind != CompoundKind::HasMany {
                    return Err(syn::Error::new_spanned(
                        ident,
                        "has_many must be paired with HasMany",
                    ));
                }
                Ok(Some(Self::HasMany))
            }
            compound_attr::SeaOrm {
                has_many: Some(_),
                via: Some(via),
                ..
            } => {
                if compound.kind != CompoundKind::HasMany {
                    return Err(syn::Error::new_spanned(
                        ident,
                        "has_many via must be paired with HasMany",
                    ));
                }
                Ok(Some(Self::HasManyVia { via: via.clone() }))
            }
            _ => Ok(None),
        }
    }
}

struct ScalarField<'a> {
    ty: &'a Type,
    column: Ident,
}

impl<'a> ScalarField<'a> {
    fn from_field(field: &'a syn::Field, ident: &Ident) -> syn::Result<Self> {
        let column = trim_starting_raw_identifier(ident).to_upper_camel_case();
        let mut column = Ident::new(&escape_rust_keyword(column), ident.span());

        for attr in &field.attrs {
            if !attr.path().is_ident("sea_orm") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("enum_name") {
                    column = syn::parse_str(&meta.value()?.parse::<LitStr>()?.value())?;
                } else {
                    consume_meta(meta);
                }
                Ok(())
            })?;
        }

        Ok(Self {
            ty: &field.ty,
            column,
        })
    }

    fn is_optional(&self) -> bool {
        let Type::Path(type_path) = self.ty else {
            return false;
        };
        type_path
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "Option")
    }
}

#[derive(Clone, Copy)]
enum FieldParseMode {
    Compact,
    Dense,
}

struct CompoundField {
    compound: CompoundType,
}

struct RelationField {
    compound: CompoundType,
    relation: RelationAttr,
}

enum FieldKind<'a> {
    Ignored,
    Scalar(ScalarField<'a>),
    Compound(CompoundField),
    Relation(RelationField),
}

struct Field<'a> {
    ident: &'a Ident,
    kind: FieldKind<'a>,
}

impl<'a> Field<'a> {
    fn from_field(field: &'a syn::Field, mode: FieldParseMode) -> syn::Result<Self> {
        let Some(ident) = &field.ident else {
            return Err(syn::Error::new_spanned(field, "expected named field"));
        };
        let mut ignored = false;
        for attr in &field.attrs {
            if !attr.path().is_ident("sea_orm") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("ignore") {
                    ignored = true;
                } else {
                    consume_meta(meta);
                }
                Ok(())
            })?;
        }

        let kind = if ignored {
            FieldKind::Ignored
        } else if let Type::Path(type_path) = &field.ty
            && let Some(compound) = CompoundType::from_type(type_path)?
        {
            let relation = match mode {
                FieldParseMode::Compact => None,
                FieldParseMode::Dense => {
                    let attrs = compound_attr::SeaOrm::try_from_attributes(&field.attrs)?
                        .unwrap_or_default();
                    RelationAttr::from_attr(&attrs, ident, &compound)?
                }
            };
            if let Some(relation) = relation {
                FieldKind::Relation(RelationField { compound, relation })
            } else {
                FieldKind::Compound(CompoundField { compound })
            }
        } else {
            FieldKind::Scalar(ScalarField::from_field(field, ident)?)
        };
        Ok(Self { ident, kind })
    }

    fn expand_into(&'a self, fields: &'a Fields<'a>, output: &mut Output<'a>) -> syn::Result<()> {
        let ident = self.ident;
        match &self.kind {
            FieldKind::Ignored => {
                output.ignored_model_fields.push(ident);
            }
            FieldKind::Scalar(scalar) => {
                let field_type = scalar.ty;
                output.model_field_defs.push(quote! {
                    #[doc = " Generated by sea-orm-macros"]
                    pub #ident: sea_orm::ActiveValue<#field_type>
                });
                output.active_model_setters.extend(
                    ActiveModelSetter {
                        field: self,
                        fields,
                    }
                    .expand()?,
                );
                output.scalar_fields.push(ident);
            }
            FieldKind::Compound(CompoundField { compound })
            | FieldKind::Relation(RelationField { compound, .. }) => {
                self.expand_compound_into(fields, output, compound)?;
            }
        }

        Ok(())
    }

    fn expand_compound_into(
        &'a self,
        fields: &'a Fields<'a>,
        output: &mut Output<'a>,
        compound: &CompoundType,
    ) -> syn::Result<()> {
        let ident = self.ident;
        let entity_type = &compound.entity;
        let field_type = match compound.kind {
            CompoundKind::HasOne(cardinality) => {
                let target_type = cardinality.has_one_target_type(quote!(#entity_type));
                quote!(ActiveHasOne<#target_type>)
            }
            CompoundKind::HasMany => quote!(ActiveHasMany<#entity_type>),
        };
        output.model_field_defs.push(quote! {
            #[doc = " Generated by sea-orm-macros"]
            pub #ident: #field_type
        });
        output.active_model_setters.extend(
            ActiveModelSetter {
                field: self,
                fields,
            }
            .expand()?,
        );
        output.compound_fields.push(ident);

        Ok(())
    }
}

struct Fields<'a>(Vec<Field<'a>>);

impl<'a> Fields<'a> {
    fn from_data(data: &'a Data, ident: &Ident, mode: FieldParseMode) -> syn::Result<Self> {
        let fields = if let Data::Struct(r#struct) = data
            && let syn::Fields::Named(fields) = &r#struct.fields
        {
            fields
                .named
                .iter()
                .map(|field| Field::from_field(field, mode))
                .collect::<syn::Result<Vec<_>>>()?
        } else {
            return Err(syn::Error::new_spanned(
                ident,
                "You can only derive DeriveActiveModelEx on structs",
            ));
        };

        Ok(Self(fields))
    }

    /// Return the fields corresponding to the `from` columns of a relation.
    /// For example:
    /// #[sea_orm(belogns_to, from = "cake_id")], #[sea_orm(belongs_to, from = "Column::CakeId")] will return the field corresponding to `cake_id`.
    /// #[sea_orm(belongs_to, from = "(user_id, post_id)")] will return the fields corresponding to `user_id` and `post_id`.
    fn get_nullable_from_fields(&'a self, columns_lit: &LitStr) -> syn::Result<Vec<&'a Ident>> {
        let columns = if columns_lit.value().starts_with('(') {
            let parser = Punctuated::<Path, Comma>::parse_terminated;
            columns_lit.parse_with(|input: syn::parse::ParseStream<'_>| {
                let content;
                syn::parenthesized!(content in input);
                parser.parse2(content.parse()?)
            })?
        } else {
            let mut columns = Punctuated::new();
            columns.push(columns_lit.parse()?);
            columns
        };

        columns.iter().try_fold(Vec::new(), |mut fields, column| {
            let Some(column) = column.segments.last() else {
                return Err(syn::Error::new_spanned(column, "expected column path"));
            };
            let column = Ident::new(
                &escape_rust_keyword(column.ident.to_string().to_upper_camel_case()),
                column.ident.span(),
            );
            let (field, scalar) = self
                .0
                .iter()
                .find_map(|field| {
                    if let FieldKind::Scalar(scalar) = &field.kind
                        && scalar.column == column
                    {
                        Some((field.ident, scalar))
                    } else {
                        None
                    }
                })
                .ok_or_else(|| {
                    syn::Error::new(
                        columns_lit.span(),
                        format!("unknown `from` column `{column}`"),
                    )
                })?;
            if scalar.is_optional() {
                fields.push(field);
            }
            Ok(fields)
        })
    }

    fn entity_count(&self, entity: &TypePath) -> usize {
        self.0
            .iter()
            .filter(|field| match &field.kind {
                FieldKind::Compound(compound_field) => &compound_field.compound.entity == entity,
                FieldKind::Relation(relation_field) => &relation_field.compound.entity == entity,
                _ => false,
            })
            .count()
    }
}

struct ActiveModelSetter<'a> {
    field: &'a Field<'a>,
    fields: &'a Fields<'a>,
}

impl<'a> ActiveModelSetter<'a> {
    fn expand(&self) -> syn::Result<TokenStream> {
        let field_ident = self.field.ident;
        match &self.field.kind {
            FieldKind::Ignored => Ok(quote!()),
            FieldKind::Scalar(scalar) => {
                let field_type = scalar.ty;
                let setter = format_ident!("set_{}", field_ident);

                Ok(quote! {
                    #[doc = " Generated by sea-orm-macros"]
                    pub fn #setter(mut self, v: impl Into<#field_type>) -> Self {
                        self.#field_ident = sea_orm::Set(v.into());
                        self
                    }
                })
            }
            FieldKind::Compound(compound_field) => {
                self.expand_compound(&compound_field.compound, None)
            }
            FieldKind::Relation(relation_field) => {
                self.expand_compound(&relation_field.compound, Some(&relation_field.relation))
            }
        }
    }

    fn expand_compound(
        &self,
        compound: &CompoundType,
        relation: Option<&RelationAttr>,
    ) -> syn::Result<TokenStream> {
        let field_ident = self.field.ident;
        let entity_path = &compound.entity;
        let mut active_model_path = entity_path.path.clone();
        let Some(segment) = active_model_path.segments.last_mut() else {
            return Err(syn::Error::new_spanned(entity_path, "expected entity path"));
        };
        segment.ident = format_ident!("ActiveModelEx");
        segment.arguments = PathArguments::None;

        match compound.kind {
            CompoundKind::HasOne(cardinality) => {
                let has_one_target = cardinality.has_one_target_type(quote!(#entity_path));
                let setter = format_ident!("set_{}", field_ident);
                let optional = if cardinality.is_optional() {
                    quote!(Some)
                } else {
                    quote!()
                };
                let optional_setters = if cardinality.is_optional() {
                    let optional_setter = format_ident!("set_{}_option", field_ident);
                    let clear_method = format_ident!("clear_{}", field_ident);
                    let clear_parent_keys = self.clear_parent_key_setters(relation)?;

                    quote! {
                        #[doc = " Generated by sea-orm-macros"]
                        pub fn #optional_setter(mut self, v: Option<impl Into<#active_model_path>>) -> Self {
                            if v.is_none() {
                                #clear_parent_keys
                            }
                            self.#field_ident = sea_orm::ActiveHasOne::<#has_one_target>::set(v);
                            self
                        }

                        #[doc = " Generated by sea-orm-macros"]
                        pub fn #clear_method(mut self) -> Self {
                            #clear_parent_keys
                            self.#field_ident = sea_orm::ActiveHasOne::Set(None);
                            self
                        }
                    }
                } else {
                    quote!()
                };

                Ok(quote! {
                    #[doc = " Generated by sea-orm-macros"]
                    pub fn #setter(mut self, v: impl Into<#active_model_path>) -> Self {
                        self.#field_ident = sea_orm::ActiveHasOne::<#has_one_target>::set(#optional(v.into()));
                        self
                    }

                    #optional_setters
                })
            }
            CompoundKind::HasMany => {
                let setter = format_ident!(
                    "add_{}",
                    pluralizer::pluralize(&field_ident.to_string(), 1, false)
                );

                Ok(quote! {
                    #[doc = " Generated by sea-orm-macros"]
                    pub fn #setter(mut self, v: impl Into<#active_model_path>) -> Self {
                        self.#field_ident.push(v.into());
                        self
                    }
                })
            }
        }
    }

    fn clear_parent_key_setters(
        &self,
        relation: Option<&RelationAttr>,
    ) -> syn::Result<TokenStream> {
        let from = match relation {
            Some(
                RelationAttr::BelongsTo { from, .. } | RelationAttr::BelongsToSelf { from, .. },
            ) => from,
            _ => return Ok(quote!()),
        };
        let nullable_from_fields = self.fields.get_nullable_from_fields(from)?;

        Ok(quote! {
            #(self.#nullable_from_fields = sea_orm::Set(None);)*
        })
    }
}

enum Relation<'a> {
    BelongsTo(BelongsToField<'a>),
    BelongsToSelf(SelfHasOneField<'a>),
    HasOne(HasOneField<'a>),
    HasMany(HasManyField<'a>),
    HasManySelf(SelfRelationField<'a>),
    HasManyVia(ViaField<'a>),
    HasManyViaSelf(SelfViaField<'a>),
}

#[derive(Default)]
struct ActiveModelActionTokens {
    belongs_to_action: TokenStream,
    belongs_to_after_action: TokenStream,
    has_one_before_action: TokenStream,
    has_one_action: TokenStream,
    has_one_delete: TokenStream,
    has_many_before_action: TokenStream,
    has_many_action: TokenStream,
    has_many_delete: TokenStream,
    has_many_via_before_action: TokenStream,
    has_many_via_action: TokenStream,
    has_many_via_delete: TokenStream,
}

impl ActiveModelActionTokens {
    fn from_fields(fields: &Fields<'_>) -> syn::Result<Self> {
        let relations = fields
            .0
            .iter()
            .filter_map(|field| {
                let FieldKind::Relation(relation_field) = &field.kind else {
                    return None;
                };
                Some((field.ident, relation_field))
            })
            .try_fold(Vec::new(), |mut relations, (ident, relation_field)| {
                let compound = &relation_field.compound;
                let relation_attr = &relation_field.relation;
                let is_unique_entity = fields.entity_count(&compound.entity) == 1;

                let relation = match relation_attr {
                    RelationAttr::BelongsToSelf {
                        relation_enum,
                        cardinality,
                        from,
                    } => Some(Relation::BelongsToSelf(SelfHasOneField {
                        ident,
                        relation_enum: relation_enum.clone(),
                        cardinality: *cardinality,
                        nullable_from_fields: fields.get_nullable_from_fields(from)?,
                    })),
                    RelationAttr::HasManySelf { relation_enum } => {
                        Some(Relation::HasManySelf(SelfRelationField {
                            ident,
                            relation_enum: relation_enum.clone(),
                        }))
                    }
                    RelationAttr::BelongsTo {
                        cardinality,
                        relation_enum,
                        from,
                    } => {
                        // Always generate. Pick the FK-write path: an explicit
                        // relation_enum, or a non-unique target, is keyed by relation
                        // variant (there is no canonical `Related<E>` to key by);
                        // a unique target with no explicit enum uses the entity-keyed path.
                        let relation_variant = match (relation_enum.as_ref(), is_unique_entity) {
                            (Some(relation_enum), _) => Some(Ident::new(
                                &relation_enum.value().to_upper_camel_case(),
                                relation_enum.span(),
                            )),
                            (None, false) => Some(Ident::new(
                                &infer_relation_name_from_entity(&compound.entity)
                                    .to_upper_camel_case(),
                                Span::call_site(),
                            )),
                            (None, true) => None,
                        };
                        Some(Relation::BelongsTo(BelongsToField {
                            ident,
                            entity: &compound.entity,
                            cardinality: *cardinality,
                            relation_enum: relation_variant,
                            nullable_from_fields: fields.get_nullable_from_fields(from)?,
                        }))
                    }
                    RelationAttr::HasOne { cardinality } if is_unique_entity => {
                        Some(Relation::HasOne(HasOneField {
                            ident,
                            entity: &compound.entity,
                            cardinality: *cardinality,
                        }))
                    }
                    RelationAttr::HasMany if is_unique_entity => {
                        Some(Relation::HasMany(HasManyField {
                            ident,
                            entity: &compound.entity,
                        }))
                    }
                    RelationAttr::HasManyVia { via } if is_unique_entity => {
                        Some(Relation::HasManyVia(ViaField {
                            ident,
                            entity: via.value(),
                        }))
                    }
                    RelationAttr::HasManyViaSelf { via, reverse } => {
                        Some(Relation::HasManyViaSelf(SelfViaField {
                            ident,
                            entity: via.value(),
                            reverse: *reverse,
                        }))
                    }
                    _ => None,
                };

                relations.extend(relation);
                Ok::<_, syn::Error>(relations)
            })?;

        let mut this = Self::default();
        for relation in &relations {
            relation.expand_into(&mut this);
        }

        Ok(this)
    }

    fn expand(self) -> TokenStream {
        let async_ = async_token();
        let await_ = await_token();
        let Self {
            belongs_to_action,
            belongs_to_after_action,
            has_one_before_action,
            has_one_action,
            has_one_delete,
            has_many_before_action,
            has_many_action,
            has_many_delete,
            has_many_via_before_action,
            has_many_via_action,
            has_many_via_delete,
        } = self;

        quote! {
            #[doc = " Generated by sea-orm-macros"]
            pub #async_ fn insert<'a, C>(self, db: &'a C) -> Result<ModelEx, sea_orm::DbErr>
            where
                C: sea_orm::TransactionTrait,
            {
                let active_model = self.action(sea_orm::ActiveModelAction::Insert, db)#await_?;
                active_model.try_into()
            }

            #[doc = " Generated by sea-orm-macros"]
            pub #async_ fn update<'a, C>(self, db: &'a C) -> Result<ModelEx, sea_orm::DbErr>
            where
                C: sea_orm::TransactionTrait,
            {
                let active_model = self.action(sea_orm::ActiveModelAction::Update, db)#await_?;
                active_model.try_into()
            }

            #[doc = " Generated by sea-orm-macros"]
            pub #async_ fn save<'a, C>(self, db: &'a C) -> Result<Self, sea_orm::DbErr>
            where
                C: sea_orm::TransactionTrait,
            {
                self.action(sea_orm::ActiveModelAction::Save, db)#await_
            }

            #[doc = " Generated by sea-orm-macros"]
            pub #async_ fn delete<'a, C>(self, db: &'a C) -> Result<sea_orm::DeleteResult, sea_orm::DbErr>
            where
                C: sea_orm::TransactionTrait,
            {
                use sea_orm::{IntoActiveModel, TransactionSession};

                let txn = db.begin()#await_?;
                let db = &txn;
                let mut deleted = sea_orm::DeleteResult::empty();

                #has_one_delete
                #has_many_delete
                #has_many_via_delete

                let model: ActiveModel = self.into();
                deleted.merge(model.delete(db)#await_?);

                txn.commit()#await_?;

                Ok(deleted)
            }

            #[doc = " Generated by sea-orm-macros"]
            pub #async_ fn action<'a, C>(mut self, action: sea_orm::ActiveModelAction, db: &'a C) -> Result<Self, sea_orm::DbErr>
            where
                C: sea_orm::TransactionTrait,
            {
                use sea_orm::{ActiveHasOne, ActiveHasMany, IntoActiveModel, TransactionSession};
                let txn = db.begin()#await_?;
                let db = &txn;
                let mut deleted = sea_orm::DeleteResult::empty();

                #belongs_to_action
                #has_one_before_action
                #has_many_before_action
                #has_many_via_before_action

                let model: ActiveModel = self.into();

                let mut model: Self = if model.is_changed() {
                    match action {
                        sea_orm::ActiveModelAction::Insert => model.insert(db)#await_,
                        sea_orm::ActiveModelAction::Update => model.update(db)#await_,
                        sea_orm::ActiveModelAction::Save => if !model.is_update() {
                            model.insert(db)#await_
                        } else {
                            model.update(db)#await_
                        },
                    }?.into_ex().into()
                } else {
                    model.into()
                };

                #belongs_to_after_action
                #has_one_action
                #has_many_action
                #has_many_via_action

                txn.commit()#await_?;

                Ok(model)
            }
        }
    }
}

#[derive(Default)]
struct Output<'a> {
    model_field_defs: Vec<TokenStream>,
    active_model_setters: TokenStream,
    ignored_model_fields: Vec<&'a Ident>,
    scalar_fields: Vec<&'a Ident>,
    compound_fields: Vec<&'a Ident>,
}

impl<'a> Output<'a> {
    fn from_fields(fields: &'a Fields<'a>) -> syn::Result<Self> {
        let mut this = Self::default();

        for field in &fields.0 {
            field.expand_into(fields, &mut this)?;
        }

        Ok(this)
    }
}

fn expand_active_model_ex<'a>(
    vis: &Visibility,
    ident: &Ident,
    data: &Data,
    fields: &'a Fields<'a>,
    active_model_action: TokenStream,
) -> syn::Result<TokenStream> {
    let async_ = async_token();
    let await_ = await_token();
    let active_model_trait_methods =
        DeriveActiveModel::new(vis, ident, data)?.impl_active_model_trait_methods();
    let Output {
        model_field_defs,
        active_model_setters,
        ignored_model_fields,
        scalar_fields,
        compound_fields,
    } = Output::from_fields(fields)?;

    Ok(quote! {
        #[doc = " Generated by sea-orm-macros"]
        #[derive(Clone, Debug, PartialEq)]
        #vis struct ActiveModelEx {
            #(#model_field_defs),*
        }

        impl ActiveModel {
            #[doc = " Generated by sea-orm-macros"]
            pub fn into_ex(self) -> ActiveModelEx {
                self.into()
            }
        }

        #[automatically_derived]
        impl sea_orm::ActiveModelTrait for ActiveModelEx {
            type Entity = Entity;

            #active_model_trait_methods

            /// Returns true if any field is set or changed. This is recursive.
            fn is_changed(&self) -> bool {
                false #(|| self.#scalar_fields.is_set())* #(|| self.#compound_fields.is_changed())*
            }

            fn default() -> Self {
                <ActiveModel as sea_orm::ActiveModelBehavior>::new().into()
            }
        }

        impl ActiveModelEx {
            #[doc = " Generated by sea-orm-macros"]
            pub fn new() -> Self {
                <Self as sea_orm::ActiveModelTrait>::default()
            }

            #active_model_action

            #active_model_setters
        }

        #[automatically_derived]
        impl std::default::Default for ActiveModelEx {
            fn default() -> Self {
                <Self as sea_orm::ActiveModelTrait>::default()
            }
        }

        #[automatically_derived]
        impl std::convert::From<ActiveModel> for ActiveModelEx {
            fn from(m: ActiveModel) -> Self {
                Self {
                    #(#scalar_fields: m.#scalar_fields,)*
                    #(#compound_fields: Default::default(),)*
                }
            }
        }

        #[automatically_derived]
        impl std::convert::From<ActiveModelEx> for ActiveModel {
            fn from(m: ActiveModelEx) -> Self {
                Self {
                    #(#scalar_fields: m.#scalar_fields,)*
                }
            }
        }

        #[automatically_derived]
        impl std::convert::From<ModelEx> for ActiveModelEx {
            fn from(m: ModelEx) -> Self {
                Self {
                    #(#scalar_fields: sea_orm::ActiveValue::Unchanged(m.#scalar_fields),)*
                    #(#compound_fields: m.#compound_fields.into_active_model(),)*
                }
            }
        }

        #[automatically_derived]
        impl std::convert::TryFrom<ActiveModelEx> for ModelEx {
            type Error = sea_orm::DbErr;
            fn try_from(a: ActiveModelEx) -> Result<Self, sea_orm::DbErr> {
                #(if a.#scalar_fields.is_not_set() {
                    return Err(sea_orm::DbErr::AttrNotSet(stringify!(#scalar_fields).to_owned()));
                })*
                Ok(
                    Self {
                        #(#scalar_fields: a.#scalar_fields.unwrap(),)*
                        #(#compound_fields: a.#compound_fields.try_into_model()?,)*
                        #(#ignored_model_fields: Default::default(),)*
                    }
                )
            }
        }

        #[automatically_derived]
        impl sea_orm::IntoActiveModel<ActiveModelEx> for ModelEx {
            fn into_active_model(self) -> ActiveModelEx {
                self.into()
            }
        }

        #[automatically_derived]
        impl sea_orm::TryIntoModel<ModelEx> for ActiveModelEx {
            fn try_into_model(self) -> Result<ModelEx, sea_orm::DbErr> {
                self.try_into()
            }
        }

        impl Model {
            #[doc = " Generated by sea-orm-macros"]
            pub #async_ fn cascade_delete<'a, C>(self, db: &'a C) -> Result<sea_orm::DeleteResult, sea_orm::DbErr>
            where
                C: sea_orm::TransactionTrait,
            {
                self.into_ex().delete(db)#await_
            }
        }

        impl ModelEx {
            #[doc = " Generated by sea-orm-macros"]
            pub #async_ fn delete<'a, C>(self, db: &'a C) -> Result<sea_orm::DeleteResult, sea_orm::DbErr>
            where
                C: sea_orm::TransactionTrait,
            {
                let active_model: ActiveModelEx = self.into();
                active_model.delete(db)#await_
            }
        }

        impl ActiveModel {
            #[doc = " Generated by sea-orm-macros"]
            pub fn builder() -> ActiveModelEx {
                ActiveModelEx::new()
            }
        }
    })
}

struct BelongsToField<'a> {
    ident: &'a Ident,
    entity: &'a TypePath,
    cardinality: CardinalityKind,
    nullable_from_fields: Vec<&'a Ident>,
    /// The relation-enum variant to key FK writes by (`*_parent_key_for`) when this
    /// relation shares its target entity with another. `None` uses the entity-keyed
    /// path (`*_parent_key`), valid only when the target has a unique `Related<E>`.
    relation_enum: Option<Ident>,
}

impl BelongsToField<'_> {
    fn belongs_to_action(&self) -> TokenStream {
        let await_ = await_token();
        let box_pin = if cfg!(feature = "async") {
            quote!(Box::pin)
        } else {
            quote!()
        };
        let ident = self.ident;
        let related_entity = self.entity;
        let has_one_target = self
            .cardinality
            .has_one_target_type(quote!(#related_entity));
        let optional = if self.cardinality.is_optional() {
            quote!(Some)
        } else {
            quote!()
        };
        let nullable_from_fields = &self.nullable_from_fields;
        // Disambiguate the FK write: a relation that shares its target entity with
        // another relation is keyed by its relation enum; otherwise by entity (the
        // canonical `Related<E>`).
        let (set_parent_key, clear_parent_key) = match &self.relation_enum {
            Some(relation_enum) => (
                quote!(self.set_parent_key_for(&model, Relation::#relation_enum)?),
                quote!(self.clear_parent_key_for(Relation::#relation_enum)?),
            ),
            None => (
                quote!(self.set_parent_key(&model)?),
                quote!(self.clear_parent_key::<#related_entity>()?),
            ),
        };
        let replace_model = quote! {
            ActiveHasOne::Set(#optional(model)) => {
                let mut model = *model;
                if model.is_update() {
                    #set_parent_key;
                    if model.is_changed() {
                        model = #box_pin(model.action(action, db))#await_?;
                    }
                } else {
                    model = #box_pin(model.action(action, db))#await_?;
                    #set_parent_key;
                }
                ActiveHasOne::<#has_one_target>::set(#optional(model))
            }
        };
        let clear_optional = if self.cardinality.is_optional() {
            quote! {
                ActiveHasOne::Set(None) => {
                    #(self.#nullable_from_fields = sea_orm::Set(None);)*
                    if !#clear_parent_key {
                        return Err(sea_orm::DbErr::Type(format!(
                            "Relation {} cannot be cleared",
                            stringify!(#ident)
                        )));
                    }
                    ActiveHasOne::Set(None)
                }
            }
        } else {
            quote!()
        };

        quote! {
            let #ident = match self.#ident.take() {
                ActiveHasOne::NotSet => ActiveHasOne::NotSet,
                #replace_model
                #clear_optional
            };
        }
    }

    fn belongs_to_after_action(&self) -> TokenStream {
        let ident = self.ident;
        quote! {
            if #ident.is_set() {
                model.#ident = #ident;
            }
        }
    }

    fn expand_into(&self, output: &mut ActiveModelActionTokens) {
        output.belongs_to_action.extend(self.belongs_to_action());
        output
            .belongs_to_after_action
            .extend(self.belongs_to_after_action());
    }
}

struct HasOneField<'a> {
    ident: &'a Ident,
    entity: &'a TypePath,
    cardinality: CardinalityKind,
}

impl HasOneField<'_> {
    fn has_one_before_action(&self) -> TokenStream {
        let ident = self.ident;
        quote! {
            let #ident = std::mem::take(&mut self.#ident);
        }
    }

    fn has_one_action(&self) -> TokenStream {
        let await_ = await_token();
        let box_pin = if cfg!(feature = "async") {
            quote!(Box::pin)
        } else {
            quote!()
        };
        let delete_associated_model = quote! {
            let mut item = item.into_active_model();
            if item.clear_parent_key::<Entity>()? {
                item.update(db)#await_?;
            } else {
                deleted.merge(item.into_ex().delete(db)#await_?); // deep delete
            }
        };
        let ident = self.ident;
        let related_entity = self.entity;
        let has_one_target = self
            .cardinality
            .has_one_target_type(quote!(#related_entity));

        let optional = if self.cardinality.is_optional() {
            quote!(Some)
        } else {
            quote!()
        };

        let delete_existing_child = quote! {
            if let Some(item) = model.find_related(#related_entity).one(db)#await_? {
                let item_pk = sea_orm::ModelTrait::get_primary_key_value(&item);
                let child_pk = sea_orm::ActiveModelTrait::get_primary_key_value(&child);
                if child_pk.as_ref() != Some(&item_pk) {
                    #delete_associated_model
                }
            }
        };
        let set_child_action = quote! {
            #delete_existing_child
            child.set_parent_key(&model)?;
            let child = if child.is_changed() {
                #box_pin(child.action(action, db))#await_?
            } else {
                child
            };
            model.#ident = ActiveHasOne::<#has_one_target>::set(#optional(child));
        };
        let set_relation_action = if self.cardinality.is_optional() {
            quote! {
                ActiveHasOne::Set(Some(child)) => {
                    let mut child = *child;
                    #set_child_action
                }
                ActiveHasOne::Set(None) => {
                    if let Some(item) = model.find_related(#related_entity).one(db)#await_? {
                        #delete_associated_model
                    }
                    model.#ident = ActiveHasOne::Set(None);
                }
            }
        } else {
            quote! {
                ActiveHasOne::Set(child) => {
                    let mut child = *child;
                    #set_child_action
                }
            }
        };

        quote! {
            match #ident {
                ActiveHasOne::NotSet => {}
                #set_relation_action
            }
        }
    }

    fn has_one_delete(&self) -> TokenStream {
        let await_ = await_token();
        let related_entity = self.entity;
        let delete_associated_model = quote! {
            let mut item = item.into_active_model();
            if item.clear_parent_key::<Entity>()? {
                item.update(db)#await_?;
            } else {
                deleted.merge(item.into_ex().delete(db)#await_?); // deep delete
            }
        };

        quote! {
            if let Some(item) = self.find_related(#related_entity).one(db)#await_? {
                #delete_associated_model
            }
        }
    }

    fn expand_into(&self, output: &mut ActiveModelActionTokens) {
        output
            .has_one_before_action
            .extend(self.has_one_before_action());
        output.has_one_action.extend(self.has_one_action());
        output.has_one_delete.extend(self.has_one_delete());
    }
}

struct SelfHasOneField<'a> {
    ident: &'a Ident,
    relation_enum: LitStr,
    cardinality: CardinalityKind,
    nullable_from_fields: Vec<&'a Ident>,
}

impl SelfHasOneField<'_> {
    fn belongs_to_action(&self) -> TokenStream {
        let await_ = await_token();
        let box_pin = if cfg!(feature = "async") {
            quote!(Box::pin)
        } else {
            quote!()
        };
        let ident = self.ident;
        let relation_enum = Ident::new(&self.relation_enum.value(), self.relation_enum.span());
        let relation_enum = quote!(Relation::#relation_enum);
        let has_one_target = self.cardinality.has_one_target_type(quote!(Entity));
        let optional = if self.cardinality.is_optional() {
            quote!(Some)
        } else {
            quote!()
        };
        let nullable_from_fields = &self.nullable_from_fields;
        let clear_parent_key_fields = quote! {
            #(self.#nullable_from_fields = sea_orm::Set(None);)*
        };
        let replace_model = quote! {
            ActiveHasOne::Set(#optional(model)) => {
                let mut model = *model;
                if model.is_update() {
                    self.set_parent_key_for(&model, #relation_enum)?;
                    if model.is_changed() {
                        model = #box_pin(model.action(action, db))#await_?;
                    }
                } else {
                    model = #box_pin(model.action(action, db))#await_?;
                    self.set_parent_key_for(&model, #relation_enum)?;
                }
                ActiveHasOne::<#has_one_target>::set(#optional(model))
            }
        };
        let clear_optional = if self.cardinality.is_optional() {
            quote! {
                ActiveHasOne::Set(None) => {
                    #clear_parent_key_fields
                    if !self.clear_parent_key_for(#relation_enum)? {
                        return Err(sea_orm::DbErr::Type(format!(
                            "Relation {} cannot be cleared",
                            stringify!(#ident)
                        )));
                    }
                    ActiveHasOne::Set(None)
                }
            }
        } else {
            quote!()
        };

        quote! {
            let #ident = match self.#ident.take() {
                ActiveHasOne::NotSet => ActiveHasOne::NotSet,
                #replace_model
                #clear_optional
            };
        }
    }

    fn belongs_to_after_action(&self) -> TokenStream {
        let ident = self.ident;
        quote! {
            if #ident.is_set() {
                model.#ident = #ident;
            }
        }
    }

    fn expand_into(&self, output: &mut ActiveModelActionTokens) {
        output.belongs_to_action.extend(self.belongs_to_action());
        output
            .belongs_to_after_action
            .extend(self.belongs_to_after_action());
    }
}

struct SelfRelationField<'a> {
    ident: &'a Ident,
    relation_enum: LitStr,
}

impl SelfRelationField<'_> {
    fn expand_into(&self, output: &mut ActiveModelActionTokens) {
        let await_ = await_token();
        let box_pin = if cfg!(feature = "async") {
            quote!(Box::pin)
        } else {
            quote!()
        };
        let ident = self.ident;
        let relation_enum = Ident::new(&self.relation_enum.value(), self.relation_enum.span());
        let relation_enum = quote!(Relation::#relation_enum);

        let delete_associated_model = quote! {
            let mut item = item.into_active_model();
            if item.clear_parent_key_for_self_rev(#relation_enum)? {
                item.update(db)#await_?;
            } else {
                // attempt to cascade delete may lead to infinite recursion
                return Err(sea_orm::DbErr::RecordNotUpdated);
            }
        };

        let has_many_before_action = quote! {
            let #ident = self.#ident.take();
        };

        let has_many_action = quote! {
            if #ident.is_replace() {
                for item in model.find_belongs_to_self(#relation_enum, db.get_database_backend())?.all(db)#await_? {
                    if !#ident.find(&item) {
                        #delete_associated_model
                    }
                }
            }
            model.#ident = #ident.empty_holder();
            for mut #ident in #ident.into_vec() {
                #ident.set_parent_key_for_self_rev(&model, #relation_enum)?;
                let #ident = if #ident.is_changed() {
                    #box_pin(#ident.action(action, db))#await_?
                } else {
                    #ident
                };
                model.#ident.push(#ident);
            }
        };

        let has_many_delete = quote! {
            for item in self.find_belongs_to_self(#relation_enum, db.get_database_backend())?.all(db)#await_? {
                #delete_associated_model
            }
        };

        output.has_many_before_action.extend(has_many_before_action);
        output.has_many_action.extend(has_many_action);
        output.has_many_delete.extend(has_many_delete);
    }
}

struct ViaField<'a> {
    ident: &'a Ident,
    entity: String,
}

impl ViaField<'_> {
    fn expand_into(&self, output: &mut ActiveModelActionTokens) {
        let await_ = await_token();
        let box_pin = if cfg!(feature = "async") {
            quote!(Box::pin)
        } else {
            quote!()
        };
        let ident = self.ident;
        let mut via_entity = self.entity.as_str();
        if let Some((prefix, _)) = via_entity.split_once("::") {
            via_entity = prefix;
        }

        let related_entity: TokenStream = format!("super::{via_entity}::Entity").parse().unwrap();

        let has_many_via_before_action = quote! {
            let #ident = self.#ident.take();
        };

        let has_many_via_action = quote! {
            model.#ident = #ident.empty_holder();
            for item in #ident.into_vec() {
                let item = if item.is_update() && !item.is_changed() {
                    item
                } else {
                    #box_pin(item.action(action, db))#await_?
                };
                model.#ident.push(item);
            }
            model.establish_links(
                #related_entity,
                model.#ident.as_slice(),
                model.#ident.is_replace(),
                db
            )#await_?;
        };

        let has_many_via_delete = quote! {
            deleted.merge(self.delete_links(#related_entity, db)#await_?);
        };

        output
            .has_many_via_before_action
            .extend(has_many_via_before_action);
        output.has_many_via_action.extend(has_many_via_action);
        output.has_many_via_delete.extend(has_many_via_delete);
    }
}

struct SelfViaField<'a> {
    ident: &'a Ident,
    entity: String,
    reverse: bool,
}

impl SelfViaField<'_> {
    fn expand_into(&self, output: &mut ActiveModelActionTokens) {
        let await_ = await_token();
        let box_pin = if cfg!(feature = "async") {
            quote!(Box::pin)
        } else {
            quote!()
        };
        let ident = self.ident;
        let related_entity: TokenStream =
            format!("super::{}::Entity", self.entity).parse().unwrap();
        let establish_links = Ident::new(
            if self.reverse {
                "establish_links_self_rev"
            } else {
                "establish_links_self"
            },
            ident.span(),
        );

        let has_many_via_before_action = quote! {
            let #ident = self.#ident.take();
        };

        let has_many_via_action = quote! {
            model.#ident = #ident.empty_holder();
            for item in #ident.into_vec() {
                let item = if item.is_update() && !item.is_changed() {
                    item
                } else {
                    #box_pin(item.action(action, db))#await_?
                };
                model.#ident.push(item);
            }
            model.#establish_links(
                #related_entity,
                model.#ident.as_slice(),
                model.#ident.is_replace(),
                db
            )#await_?;
        };

        let has_many_via_delete = quote! {
            deleted.merge(self.delete_links_self(#related_entity, db)#await_?);
        };

        output
            .has_many_via_before_action
            .extend(has_many_via_before_action);
        output.has_many_via_action.extend(has_many_via_action);
        output.has_many_via_delete.extend(has_many_via_delete);
    }
}

struct HasManyField<'a> {
    ident: &'a Ident,
    entity: &'a TypePath,
}

impl HasManyField<'_> {
    fn has_many_before_action(&self) -> TokenStream {
        let ident = self.ident;
        quote! {
            let #ident = self.#ident.take();
        }
    }

    fn has_many_action(&self) -> TokenStream {
        let await_ = await_token();
        let box_pin = if cfg!(feature = "async") {
            quote!(Box::pin)
        } else {
            quote!()
        };
        let ident = self.ident;
        let related_entity = self.entity;
        let delete_associated_model = quote! {
            let mut item = item.into_active_model();
            if item.clear_parent_key::<Entity>()? {
                item.update(db)#await_?;
            } else {
                deleted.merge(item.into_ex().delete(db)#await_?); // deep delete
            }
        };
        quote! {
            if #ident.is_replace() {
                for item in model.find_related(#related_entity).all(db)#await_? {
                    if !#ident.find(&item) {
                        #delete_associated_model
                    }
                }
            }
            model.#ident = #ident.empty_holder();
            for mut #ident in #ident.into_vec() {
                #ident.set_parent_key(&model)?;
                let #ident = if #ident.is_changed() {
                    #box_pin(#ident.action(action, db))#await_?
                } else {
                    #ident
                };
                model.#ident.push(#ident);
            }
        }
    }

    fn has_many_delete(&self) -> TokenStream {
        let await_ = await_token();
        let related_entity = self.entity;
        let delete_associated_model = quote! {
            let mut item = item.into_active_model();
            if item.clear_parent_key::<Entity>()? {
                item.update(db)#await_?;
            } else {
                deleted.merge(item.into_ex().delete(db)#await_?); // deep delete
            }
        };
        quote! {
            for item in self.find_related(#related_entity).all(db)#await_? {
                #delete_associated_model
            }
        }
    }

    fn expand_into(&self, output: &mut ActiveModelActionTokens) {
        output
            .has_many_before_action
            .extend(self.has_many_before_action());
        output.has_many_action.extend(self.has_many_action());
        output.has_many_delete.extend(self.has_many_delete());
    }
}

impl Relation<'_> {
    fn expand_into(&self, output: &mut ActiveModelActionTokens) {
        match self {
            Self::BelongsTo(field) => field.expand_into(output),
            Self::BelongsToSelf(field) => field.expand_into(output),
            Self::HasOne(field) => field.expand_into(output),
            Self::HasMany(field) => field.expand_into(output),
            Self::HasManySelf(field) => field.expand_into(output),
            Self::HasManyVia(field) => field.expand_into(output),
            Self::HasManyViaSelf(field) => field.expand_into(output),
        }
    }
}

pub fn expand_derive_active_model_ex(
    vis: &Visibility,
    ident: &Ident,
    data: &Data,
    attrs: &[Attribute],
) -> syn::Result<TokenStream> {
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

    let mode = if compact {
        FieldParseMode::Compact
    } else {
        FieldParseMode::Dense
    };
    let fields = Fields::from_data(data, ident, mode)?;
    let active_model_action_tokens = if compact {
        ActiveModelActionTokens::default()
    } else {
        ActiveModelActionTokens::from_fields(&fields)?
    };
    let active_model_action = active_model_action_tokens.expand();

    expand_active_model_ex(vis, ident, data, &fields, active_model_action)
}
