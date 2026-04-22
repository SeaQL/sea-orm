use sea_query::ColumnType;
use std::collections::HashSet;

use super::*;

// seperate enum so `ColumnType` doesnt need to derive `Hash` or `Eq`
#[derive(Hash, PartialEq, Eq)]
enum ExternalTypes {
    JsonOrJsonBinary,
    Date,
    Time,
    DateTime,
    Timestamp,
    TimestampWithTimeZone,
    DecimalOrMoney,
    Uuid,
    Vector,
    CidrOrInet,
}

impl ExternalTypes {
    fn from_column_type(col_type: &ColumnType) -> Option<Self> {
        Some(match col_type {
            ColumnType::Json | ColumnType::JsonBinary => Self::JsonOrJsonBinary,
            ColumnType::Date => Self::Date,
            ColumnType::Time => Self::Time,
            ColumnType::DateTime => Self::DateTime,
            ColumnType::Timestamp => Self::Timestamp,
            ColumnType::TimestampWithTimeZone => Self::TimestampWithTimeZone,
            ColumnType::Decimal(..) | ColumnType::Money(..) => Self::DecimalOrMoney,
            ColumnType::Uuid => Self::Uuid,
            ColumnType::Vector(..) => Self::Vector,
            ColumnType::Cidr | ColumnType::Inet => Self::CidrOrInet,
            _ => return None,
        })
    }
}

impl EntityWriter {
    #[allow(clippy::too_many_arguments)]
    pub fn gen_frontend_code_blocks(
        entity: &Entity,
        with_serde: &WithSerde,
        column_option: &ColumnOption,
        schema_name: &Option<String>,
        serde_skip_deserializing_primary_key: bool,
        serde_skip_hidden_column: bool,
        model_extra_derives: &TokenStream,
        model_extra_attributes: &TokenStream,
        _column_extra_derives: &TokenStream,
        _seaography: bool,
        _impl_active_model_behavior: bool,
    ) -> Vec<TokenStream> {
        let mut imports = Self::gen_import_serde(with_serde);
        imports.extend(Self::gen_import_active_enum(entity));
        imports.extend(Self::gen_import_frontend(entity, column_option));
        let code_blocks = vec![
            imports,
            Self::gen_frontend_model_struct(
                entity,
                with_serde,
                column_option,
                schema_name,
                serde_skip_deserializing_primary_key,
                serde_skip_hidden_column,
                model_extra_derives,
                model_extra_attributes,
            ),
        ];
        code_blocks
    }

    #[allow(clippy::too_many_arguments)]
    pub fn gen_frontend_model_struct(
        entity: &Entity,
        with_serde: &WithSerde,
        column_option: &ColumnOption,
        _schema_name: &Option<String>,
        serde_skip_deserializing_primary_key: bool,
        serde_skip_hidden_column: bool,
        model_extra_derives: &TokenStream,
        model_extra_attributes: &TokenStream,
    ) -> TokenStream {
        let column_names_snake_case = entity.get_column_names_snake_case();
        let column_rs_types = entity.get_column_rs_types(column_option);
        let if_eq_needed = entity.get_eq_needed();
        let primary_keys: Vec<String> = entity
            .primary_keys
            .iter()
            .map(|pk| pk.name.clone())
            .collect();
        let attrs: Vec<TokenStream> = entity
            .columns
            .iter()
            .map(|col| {
                let is_primary_key = primary_keys.contains(&col.name);
                let ts_type_attribute = col.get_ts_type_attrs(
                    model_extra_derives,
                    model_extra_attributes,
                );
                let serde_attribute = col.get_serde_attribute(
                    is_primary_key,
                    serde_skip_deserializing_primary_key,
                    serde_skip_hidden_column,
                );
                quote! {
                    #ts_type_attribute
                    #serde_attribute
                }
            })
            .collect();
        let extra_derive = with_serde.extra_derive();

        quote! {
            #[derive(Clone, Debug, PartialEq #if_eq_needed #extra_derive #model_extra_derives)]
            #model_extra_attributes
            pub struct Model {
                #(
                    #attrs
                    pub #column_names_snake_case: #column_rs_types,
                )*
            }
        }
    }

    pub fn gen_import_frontend(entity: &Entity, opt: &ColumnOption) -> TokenStream {
        fn collect(
            col_type: &ColumnType,
            opt: &ColumnOption,
            date_time: &mut Vec<TokenStream>,
            aliases: &mut Vec<TokenStream>,
            plain_uses: &mut Vec<TokenStream>,
            encountered: &mut HashSet<ExternalTypes>,
        ) {
            // skip column types we have already generated imports for
            if let Some(ty) = ExternalTypes::from_column_type(col_type) {
                if !encountered.insert(ty) {
                    return;
                }
            }

            match col_type {
                ColumnType::Json | ColumnType::JsonBinary => {
                    plain_uses.push(quote! { use serde_json::Value as Json; });
                }
                ColumnType::Date => match opt.date_time_crate {
                    DateTimeCrate::Chrono => {
                        date_time.push(quote! { NaiveDate as Date });
                    }
                    DateTimeCrate::Time => {
                        date_time.push(quote! { Date as TimeDate });
                    }
                },
                ColumnType::Time => match opt.date_time_crate {
                    DateTimeCrate::Chrono => {
                        date_time.push(quote! { NaiveTime as Time });
                    }
                    DateTimeCrate::Time => {
                        date_time.push(quote! { Time as TimeTime });
                    }
                },
                ColumnType::DateTime => match opt.date_time_crate {
                    DateTimeCrate::Chrono => {
                        date_time.push(quote! { NaiveDateTime as DateTime });
                    }
                    DateTimeCrate::Time => {
                        date_time.push(quote! { PrimitiveDateTime as TimeDateTime });
                    }
                },
                ColumnType::Timestamp => match opt.date_time_crate {
                    DateTimeCrate::Chrono => {
                        aliases.push(quote! {
                            type DateTimeUtc = chrono::DateTime<chrono::Utc>;
                        });
                    }
                    DateTimeCrate::Time => {
                        date_time.push(quote! { PrimitiveDateTime as TimeDateTime });
                    }
                },
                ColumnType::TimestampWithTimeZone => match opt.date_time_crate {
                    DateTimeCrate::Chrono => {
                        aliases.push(quote! {
                            type DateTimeWithTimeZone = chrono::DateTime<chrono::FixedOffset>;
                        });
                    }
                    DateTimeCrate::Time => {
                        date_time.push(quote! { OffsetDateTime as TimeDateTimeWithTimeZone });
                    }
                },
                ColumnType::Decimal(_) | ColumnType::Money(_) => {
                    plain_uses.push(quote! { use rust_decimal::Decimal; })
                }
                ColumnType::Uuid => {
                    plain_uses.push(quote! { use uuid::Uuid; });
                }
                ColumnType::Vector(_) => {
                    plain_uses.push(quote! { use pgvector::Vector as PgVector; });
                }
                ColumnType::Cidr | ColumnType::Inet => {
                    plain_uses.push(quote! { use ipnetwork::IpNetwork; });
                }
                ColumnType::Array(inner) => {
                    collect(
                        inner.as_ref(),
                        opt,
                        date_time,
                        aliases,
                        plain_uses,
                        encountered,
                    );
                }
                _ => {}
            }
        }

        let mut date_time_uses = Vec::new();
        let mut aliases = Vec::new();
        let mut plain_uses = Vec::new();
        let mut encountered = HashSet::new();

        for col in &entity.columns {
            collect(
                &col.col_type,
                opt,
                &mut date_time_uses,
                &mut aliases,
                &mut plain_uses,
                &mut encountered,
            );
        }

        let time_use = if date_time_uses.is_empty() {
            quote! {}
        } else {
            match opt.date_time_crate {
                DateTimeCrate::Chrono => quote! { use chrono::{ #(#date_time_uses),* }; },
                DateTimeCrate::Time => quote! { use time::{ #(#date_time_uses),* }; },
            }
        };

        quote! {
            #time_use
            #(#plain_uses)*
            #(#aliases)*
        }
    }
}
