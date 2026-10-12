use serde_derive_internals::{Ctxt, Derive, ast};
use syn::DeriveInput;

pub(crate) struct SerdeMeta<'a> {
    pub(super) container: ast::Container<'a>,
}

impl<'a> SerdeMeta<'a> {
    pub(super) fn new(input: &'a DeriveInput) -> syn::Result<Self> {
        let context = Ctxt::new();
        let container = ast::Container::from_ast(&context, input, Derive::Deserialize);
        context.check()?;

        let container = container
            .ok_or_else(|| syn::Error::new_spanned(input, "failed to parse Serde attributes"))?;
        Ok(Self { container })
    }
}
