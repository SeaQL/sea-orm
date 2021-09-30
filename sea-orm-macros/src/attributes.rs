pub mod r#type;

pub mod derive_attr {
    use bae::FromAttributes;

    #[derive(Default, FromAttributes)]
    pub struct SeaOrm {
        pub column: Option<syn::Ident>,
        pub entity: Option<syn::Ident>,
        pub model: Option<syn::Ident>,
        pub primary_key: Option<syn::Ident>,
        pub relation: Option<syn::Ident>,
        pub schema_name: Option<syn::Lit>,
        pub table_name: Option<syn::Lit>,
    }
}

pub mod field_attr {
    use bae::FromAttributes;

    #[derive(Default, FromAttributes)]
    pub struct SeaOrm {
        pub belongs_to: Option<syn::Lit>,
        pub has_one: Option<syn::Lit>,
        pub has_many: Option<syn::Lit>,
        pub on_update: Option<syn::Lit>,
        pub on_delete: Option<syn::Lit>,
        pub from: Option<syn::Lit>,
        pub to: Option<syn::Lit>,
    }
}
