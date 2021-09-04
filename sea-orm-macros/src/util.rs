use syn::{Attribute, Path, PathSegment, Type};

pub(crate) fn has_attribute(name: &str, attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path
            .segments
            .iter()
            .any(|segment| segment.ident == name)
    })
}

/// Returns in inner type of an `Option` `syn::Type`.
/// `None` is returned if the type provided is not wrapped in an `Option`, otherwise `Some` along with the inner type is returned.
pub(crate) fn option_type_to_inner_type(ty: &Type) -> Option<&Type> {
    fn extract_type_path(ty: &Type) -> Option<&Path> {
        match *ty {
            Type::Path(ref typepath) if typepath.qself.is_none() => Some(&typepath.path),
            _ => None,
        }
    }

    fn extract_option_segment(path: &Path) -> Option<&PathSegment> {
        let idents_of_path = path
            .segments
            .iter()
            .into_iter()
            .fold(String::new(), |mut acc, v| {
                acc.push_str(&v.ident.to_string());
                acc.push('|');
                acc
            });
        vec!["Option|", "std|option|Option|", "core|option|Option|"]
            .into_iter()
            .find(|s| idents_of_path == *s)
            .and_then(|_| path.segments.last())
    }

    extract_type_path(ty)
        .and_then(|path| extract_option_segment(path))
        .and_then(|path_seg| {
            let type_params = &path_seg.arguments;
            // It should have only on angle-bracketed param ("<String>"):
            match *type_params {
                syn::PathArguments::AngleBracketed(ref params) => params.args.first(),
                _ => None,
            }
        })
        .and_then(|generic_arg| match *generic_arg {
            syn::GenericArgument::Type(ref ty) => Some(ty),
            _ => None,
        })
}
