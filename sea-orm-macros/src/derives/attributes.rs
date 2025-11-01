pub mod derive_attr {
    use bae::FromAttributes;

    /// Attributes for Models and ActiveModels
    #[derive(Default, FromAttributes)]
    #[allow(dead_code)]
    pub struct SeaOrm {
        pub column: Option<syn::Ident>,
        pub entity: Option<syn::Ident>,
        pub model: Option<syn::Ident>,
        pub model_ex: Option<syn::Ident>,
        pub active_model: Option<syn::Ident>,
        pub primary_key: Option<syn::Ident>,
        pub relation: Option<syn::Ident>,
        pub schema_name: Option<syn::LitStr>,
        pub table_name: Option<syn::LitStr>,
        pub comment: Option<syn::LitStr>,
        pub table_iden: Option<()>,
        pub rename_all: Option<syn::LitStr>,
    }
}

pub mod relation_attr {
    use bae::FromAttributes;

    /// Attributes for Relation enum
    #[derive(Default, FromAttributes)]
    pub struct SeaOrm {
        pub belongs_to: Option<syn::Lit>,
        pub has_one: Option<syn::Lit>,
        pub has_many: Option<syn::Lit>,
        pub via_rel: Option<syn::Lit>,
        pub on_update: Option<syn::Lit>,
        pub on_delete: Option<syn::Lit>,
        pub on_condition: Option<syn::Lit>,
        pub from: Option<syn::Lit>,
        pub to: Option<syn::Lit>,
        pub fk_name: Option<syn::Lit>,
        pub skip_fk: Option<()>,
        pub condition_type: Option<syn::Lit>,
    }
}

pub mod compound_attr {
    use bae::FromAttributes;

    /// Attributes for compound model fields
    #[derive(Default, FromAttributes)]
    pub struct SeaOrm {
        pub has_one: Option<()>,
        pub has_many: Option<()>,
        pub belongs_to: Option<()>,
        pub self_ref: Option<()>,
        pub skip_fk: Option<()>,
        pub via: Option<syn::LitStr>,
        pub via_rel: Option<syn::LitStr>,
        pub from: Option<syn::LitStr>,
        pub to: Option<syn::LitStr>,
        pub relation_enum: Option<syn::LitStr>,
        pub relation_reverse: Option<syn::LitStr>,
        pub on_update: Option<syn::LitStr>,
        pub on_delete: Option<syn::LitStr>,
    }
}

pub mod column_attr {
    use bae::FromAttributes;

    /// Attributes for compound model fields
    #[derive(Default, FromAttributes)]
    pub struct SeaOrm {
        pub unique: Option<()>,
        pub unique_key: Option<syn::LitStr>,
    }
}

#[cfg(feature = "seaography")]
pub mod related_attr {
    use bae::FromAttributes;

    /// Attributes for RelatedEntity enum
    #[derive(Default, FromAttributes)]
    pub struct SeaOrm {
        ///
        /// Allows to modify target entity
        ///
        /// Required on enumeration variants
        ///
        /// If used on enumeration attributes
        /// it allows to specify different
        /// Entity ident
        pub entity: Option<syn::Lit>,
        ///
        /// Allows to specify RelationDef
        ///
        /// Optional
        ///
        /// If not supplied the generated code
        /// will utilize `impl Related` trait
        pub def: Option<syn::Lit>,
    }
}
