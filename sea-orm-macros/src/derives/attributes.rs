use darling::FromMeta;
use darling::ast::NestedMeta;
use syn::punctuated::Punctuated;
use syn::{Attribute, Meta, MetaList, MetaNameValue, Token};

fn parse_sea_orm_attrs(attrs: &[Attribute]) -> syn::Result<Vec<NestedMeta>> {
    let parser = Punctuated::<NestedMeta, Token![,]>::parse_terminated;
    let mut metas = Vec::new();
    for attr in attrs {
        if attr.path().is_ident("sea_orm") {
            metas.extend(attr.parse_args_with(parser)?);
        }
    }
    Ok(metas)
}

fn meta_ident(meta: &NestedMeta) -> Option<String> {
    let path = match meta {
        NestedMeta::Meta(Meta::Path(path))
        | NestedMeta::Meta(Meta::List(MetaList { path, .. }))
        | NestedMeta::Meta(Meta::NameValue(MetaNameValue { path, .. })) => path,
        _ => return None,
    };
    path.get_ident().map(|ident| ident.to_string())
}

// Pre-filter the parsed metas before handing them to darling:
// drop any that aren't a known field, and drop bare words on value fields
// (only flag fields accept a bare word). Otherwise darling rejects the whole list.
fn filter_metas(metas: Vec<NestedMeta>, known: &[&str], flags: &[&str]) -> Vec<NestedMeta> {
    metas
        .into_iter()
        .filter(|meta| {
            let Some(name) = meta_ident(meta) else {
                return false;
            };
            if !known.contains(&name.as_str()) {
                return false;
            }
            let is_bare = matches!(meta, NestedMeta::Meta(Meta::Path(_)));
            if is_bare && !flags.contains(&name.as_str()) {
                return false;
            }
            true
        })
        .collect()
}

fn darling_err(e: darling::Error) -> syn::Error {
    syn::Error::new(e.span(), e)
}

fn try_from_attrs<T: FromMeta>(
    attrs: &[Attribute],
    known: &[&str],
    flags: &[&str],
) -> syn::Result<Option<T>> {
    let metas = filter_metas(parse_sea_orm_attrs(attrs)?, known, flags);
    if metas.is_empty() {
        Ok(None)
    } else {
        T::from_list(&metas).map(Some).map_err(darling_err)
    }
}

fn from_attrs<T: FromMeta>(attrs: &[Attribute], known: &[&str], flags: &[&str]) -> syn::Result<T> {
    let metas = filter_metas(parse_sea_orm_attrs(attrs)?, known, flags);
    T::from_list(&metas).map_err(darling_err)
}

pub mod derive_attr {
    use super::*;
    use syn::{Ident, LitStr};

    const KNOWN: &[&str] = &[
        "column",
        "entity",
        "model",
        "model_ex",
        "active_model",
        "active_model_ex",
        "primary_key",
        "relation",
        "schema_name",
        "table_name",
        "comment",
        "rename_all",
        "table_iden",
    ];
    const FLAGS: &[&str] = &["table_iden"];

    /// Attributes for Models and ActiveModels
    #[derive(Default, FromMeta)]
    #[allow(dead_code)]
    pub struct SeaOrm {
        pub column: Option<Ident>,
        pub entity: Option<Ident>,
        pub model: Option<Ident>,
        pub model_ex: Option<Ident>,
        pub active_model: Option<Ident>,
        pub active_model_ex: Option<Ident>,
        pub primary_key: Option<Ident>,
        pub relation: Option<Ident>,
        pub schema_name: Option<LitStr>,
        pub table_name: Option<LitStr>,
        pub comment: Option<LitStr>,
        pub table_iden: Option<()>,
        pub rename_all: Option<LitStr>,
    }

    impl SeaOrm {
        pub fn try_from_attributes(attrs: &[Attribute]) -> syn::Result<Option<Self>> {
            try_from_attrs::<Self>(attrs, KNOWN, FLAGS)
        }
    }
}

pub mod relation_attr {
    use super::*;
    use syn::Lit;

    const KNOWN: &[&str] = &[
        "belongs_to",
        "has_one",
        "has_many",
        "via_rel",
        "on_update",
        "on_delete",
        "on_condition",
        "from",
        "to",
        "fk_name",
        "skip_fk",
        "condition_type",
    ];
    const FLAGS: &[&str] = &["skip_fk"];

    /// Attributes for Relation enum
    #[derive(Default, FromMeta)]
    pub struct SeaOrm {
        pub belongs_to: Option<Lit>,
        pub has_one: Option<Lit>,
        pub has_many: Option<Lit>,
        pub via_rel: Option<Lit>,
        pub on_update: Option<Lit>,
        pub on_delete: Option<Lit>,
        pub on_condition: Option<Lit>,
        pub from: Option<Lit>,
        pub to: Option<Lit>,
        pub fk_name: Option<Lit>,
        pub skip_fk: Option<()>,
        pub condition_type: Option<Lit>,
    }

    impl SeaOrm {
        pub fn from_attributes(attrs: &[Attribute]) -> syn::Result<Self> {
            from_attrs::<Self>(attrs, KNOWN, FLAGS)
        }
    }
}

pub mod compound_attr {
    use super::*;
    use syn::LitStr;

    const KNOWN: &[&str] = &[
        "has_one",
        "has_many",
        "belongs_to",
        "self_ref",
        "skip_fk",
        "via",
        "via_rel",
        "from",
        "to",
        "relation_enum",
        "relation_reverse",
        "reverse",
        "on_update",
        "on_delete",
    ];
    const FLAGS: &[&str] = &[
        "has_one",
        "has_many",
        "belongs_to",
        "self_ref",
        "skip_fk",
        "reverse",
    ];

    /// Attributes for compound model fields
    #[derive(Default, FromMeta)]
    pub struct SeaOrm {
        pub has_one: Option<()>,
        pub has_many: Option<()>,
        pub belongs_to: Option<()>,
        pub self_ref: Option<()>,
        pub skip_fk: Option<()>,
        pub via: Option<LitStr>,
        pub via_rel: Option<LitStr>,
        pub from: Option<LitStr>,
        pub to: Option<LitStr>,
        pub relation_enum: Option<LitStr>,
        pub relation_reverse: Option<LitStr>,
        pub reverse: Option<()>,
        pub on_update: Option<LitStr>,
        pub on_delete: Option<LitStr>,
    }

    impl SeaOrm {
        pub fn try_from_attributes(attrs: &[Attribute]) -> syn::Result<Option<Self>> {
            try_from_attrs::<Self>(attrs, KNOWN, FLAGS)
        }
    }
}

pub mod value_type_attr {
    use super::*;
    use syn::LitStr;

    const KNOWN: &[&str] = &[
        "column_type",
        "array_type",
        "value_type",
        "from_str",
        "to_str",
        "try_from_u64",
        "try_getable_array",
    ];
    const FLAGS: &[&str] = &["try_from_u64", "try_getable_array"];

    /// Attributes for compound model fields
    #[derive(Default, FromMeta)]
    pub struct SeaOrm {
        pub column_type: Option<LitStr>,
        pub array_type: Option<LitStr>,
        pub value_type: Option<LitStr>,
        pub from_str: Option<LitStr>,
        pub to_str: Option<LitStr>,
        pub try_from_u64: Option<()>,
        pub try_getable_array: Option<()>,
    }

    impl SeaOrm {
        pub fn try_from_attributes(attrs: &[Attribute]) -> syn::Result<Option<Self>> {
            try_from_attrs::<Self>(attrs, KNOWN, FLAGS)
        }
    }
}

#[cfg(feature = "seaography")]
pub mod related_attr {
    use super::*;
    use syn::Lit;

    const KNOWN: &[&str] = &["entity", "def"];
    const FLAGS: &[&str] = &[];

    /// Attributes for RelatedEntity enum
    #[derive(Default, FromMeta)]
    pub struct SeaOrm {
        ///
        /// Allows to modify target entity
        ///
        /// Required on enumeration variants
        ///
        /// If used on enumeration attributes
        /// it allows to specify different
        /// Entity ident
        pub entity: Option<Lit>,
        ///
        /// Allows to specify RelationDef
        ///
        /// Optional
        ///
        /// If not supplied the generated code
        /// will utilize `impl Related` trait
        pub def: Option<Lit>,
    }

    impl SeaOrm {
        pub fn try_from_attributes(attrs: &[Attribute]) -> syn::Result<Option<Self>> {
            try_from_attrs::<Self>(attrs, KNOWN, FLAGS)
        }
        pub fn from_attributes(attrs: &[Attribute]) -> syn::Result<Self> {
            from_attrs::<Self>(attrs, KNOWN, FLAGS)
        }
    }
}
