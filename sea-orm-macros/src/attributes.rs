pub mod derive_attr {
    use bae::FromAttributes;

    #[derive(Default, FromAttributes)]
    pub struct Sea {
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
    pub struct Sea {
        pub auto_increment: Option<syn::Lit>,
        pub belongs_to: Option<syn::Ident>,
        pub column_type: Option<syn::Lit>,
        pub column_type_raw: Option<syn::Lit>,
        pub from: Option<syn::Ident>,
        pub indexed: Option<()>,
        pub null: Option<()>,
        pub primary_key: Option<()>,
        pub to: Option<syn::Ident>,
        pub unique: Option<()>,
    }
}
