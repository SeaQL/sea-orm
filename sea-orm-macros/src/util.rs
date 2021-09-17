use syn::{punctuated::Punctuated, token::Comma, Field, Meta};

pub(crate) fn field_not_ignored(field: &Field) -> bool {
    for attr in field.attrs.iter() {
        if let Some(ident) = attr.path.get_ident() {
            if ident != "sea_orm" {
                continue;
            }
        } else {
            continue;
        }

        if let Ok(list) = attr.parse_args_with(Punctuated::<Meta, Comma>::parse_terminated) {
            for meta in list.iter() {
                if let Meta::Path(path) = meta {
                    if let Some(name) = path.get_ident() {
                        if name == "ignore" {
                            return false;
                        }
                    }
                }
            }
        }
    }
    true
}
