 use darling::{FromAttributes, FromMeta, Result};
 use syn::{Attribute, Expr, Ident, Lit};

/// An optional `syn::Ident` attribute value.
///
/// Darling has no `from_word` for `syn::Ident`, so a bare-word usage such as
/// `#[sea_orm(model_ex)]` would be rejected for an `Option<Ident>` field. Several
/// `sea_orm` attributes are emitted both as bare words (e.g. `#[sea_orm(model_ex)]`)
/// and as name-value paths (e.g. `#[sea_orm(model_ex = ModelEx)]`); this wrapper
/// treats the bare-word form as absent so both usages parse cleanly.
#[derive(Debug, Clone, Default)]
pub struct OptionalIdent(pub Option<Ident>);

impl FromMeta for OptionalIdent {
    fn from_word() -> Result<Self> {
        Ok(Self(None))
    }
    fn from_value(value: &Lit) -> Result<Self> {
        Ident::from_value(value).map(|id| Self(Some(id)))
    }
    fn from_expr(expr: &Expr) -> Result<Self> {
        Ident::from_expr(expr).map(|id| Self(Some(id)))
    }
}

fn try_from_attributes<T: FromAttributes>(attrs: &[Attribute]) -> syn::Result<Option<T>> {
    if attrs.iter().any(|a| a.path().is_ident("sea_orm")) {
        T::from_attributes(attrs)
            .map(Some)
            .map_err(syn::Error::from)
    } else {
        Ok(None)
    }
}

fn from_attributes<T: FromAttributes>(attrs: &[Attribute]) -> syn::Result<T> {
    T::from_attributes(attrs).map_err(syn::Error::from)
}

pub mod derive_attr {
    use super::*;
    use syn::{Ident, LitStr};

    /// Attributes for Models and ActiveModels
    #[derive(Default, FromAttributes)]
    #[darling(attributes(sea_orm), allow_unknown_fields)]
    #[allow(dead_code)]
    pub struct SeaOrm {
        pub column: Option<Ident>,
        pub entity: Option<Ident>,
        pub model: Option<Ident>,
        #[darling(default)]
        pub model_ex: OptionalIdent,
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
            super::try_from_attributes(attrs)
        }
    }
}

pub mod relation_attr {
    use super::*;
    use syn::Lit;

    /// Attributes for Relation enum
    #[derive(Default, FromAttributes)]
    #[darling(attributes(sea_orm), allow_unknown_fields)]
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
            super::from_attributes(attrs)
        }
    }
}

pub mod compound_attr {
    use super::*;
    use syn::LitStr;

    /// Attributes for compound model fields
    #[derive(Default, FromAttributes)]
    #[darling(attributes(sea_orm), allow_unknown_fields)]
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
            super::try_from_attributes(attrs)
        }
    }
}

pub mod value_type_attr {
    use super::*;
    use syn::LitStr;

    /// Attributes for compound model fields
    #[derive(Default, FromAttributes)]
    #[darling(attributes(sea_orm), allow_unknown_fields)]
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
            super::try_from_attributes(attrs)
        }
    }
}

#[cfg(feature = "seaography")]
pub mod related_attr {
    use super::*;
    use syn::Lit;

    /// Attributes for RelatedEntity enum
    #[derive(Default, FromAttributes)]
    #[darling(attributes(sea_orm), allow_unknown_fields)]
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
            super::try_from_attributes(attrs)
        }
        pub fn from_attributes(attrs: &[Attribute]) -> syn::Result<Self> {
            super::from_attributes(attrs)
        }
    }
}
