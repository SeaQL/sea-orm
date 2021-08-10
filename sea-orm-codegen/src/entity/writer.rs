use crate::Entity;
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Clone, Debug)]
pub struct EntityWriter {
    pub(crate) entities: Vec<Entity>,
}

pub struct WriterOutput {
    pub files: Vec<OutputFile>,
}

pub struct OutputFile {
    pub name: String,
    pub content: String,
}

impl EntityWriter {
    pub fn generate(self) -> WriterOutput {
        let mut files = Vec::new();
        files.extend(self.write_entities());
        files.push(self.write_mod());
        files.push(self.write_prelude());
        WriterOutput { files }
    }

    pub fn write_entities(&self) -> Vec<OutputFile> {
        self.entities
            .iter()
            .map(|entity| {
                let mut lines = Vec::new();
                Self::write_doc_comment(&mut lines);
                let code_blocks = Self::gen_code_blocks(entity);
                Self::write(&mut lines, code_blocks);
                OutputFile {
                    name: format!("{}.rs", entity.get_table_name_snake_case()),
                    content: lines.join("\n\n"),
                }
            })
            .collect()
    }

    pub fn write_mod(&self) -> OutputFile {
        let mut lines = Vec::new();
        Self::write_doc_comment(&mut lines);
        let code_blocks = self
            .entities
            .iter()
            .map(|entity| Self::gen_mod(entity))
            .collect();
        Self::write(&mut lines, code_blocks);
        OutputFile {
            name: "mod.rs".to_owned(),
            content: lines.join("\n"),
        }
    }

    pub fn write_prelude(&self) -> OutputFile {
        let mut lines = Vec::new();
        Self::write_doc_comment(&mut lines);
        let code_blocks = self
            .entities
            .iter()
            .map(|entity| Self::gen_prelude_use(entity))
            .collect();
        Self::write(&mut lines, code_blocks);
        OutputFile {
            name: "prelude.rs".to_owned(),
            content: lines.join("\n"),
        }
    }

    pub fn write(lines: &mut Vec<String>, code_blocks: Vec<TokenStream>) {
        lines.extend(
            code_blocks
                .into_iter()
                .map(|code_block| code_block.to_string())
                .collect::<Vec<_>>(),
        );
    }

    pub fn write_doc_comment(lines: &mut Vec<String>) {
        let ver = env!("CARGO_PKG_VERSION");
        let comments = vec![format!(
            "//! SeaORM Entity. Generated by sea-orm-codegen {}",
            ver
        )];
        lines.extend(comments);
        lines.push("".to_owned());
    }

    pub fn gen_code_blocks(entity: &Entity) -> Vec<TokenStream> {
        let mut code_blocks = vec![
            Self::gen_import(),
            Self::gen_entity_struct(),
            Self::gen_impl_entity_name(entity),
            Self::gen_model_struct(entity),
            Self::gen_column_enum(entity),
            Self::gen_primary_key_enum(entity),
            Self::gen_impl_primary_key(entity),
            Self::gen_relation_enum(entity),
            Self::gen_impl_column_trait(entity),
            Self::gen_impl_relation_trait(entity),
        ];
        code_blocks.extend(Self::gen_impl_related(entity));
        code_blocks.extend(Self::gen_impl_conjunct_related(entity));
        code_blocks.extend(vec![Self::gen_impl_active_model_behavior()]);
        code_blocks
    }

    pub fn gen_import() -> TokenStream {
        quote! {
            use sea_orm::entity::prelude::*;
        }
    }

    pub fn gen_entity_struct() -> TokenStream {
        quote! {
            #[derive(Copy, Clone, Default, Debug, DeriveEntity)]
            pub struct Entity;
        }
    }

    pub fn gen_impl_entity_name(entity: &Entity) -> TokenStream {
        let table_name = entity.table_name.as_str();
        quote! {
            impl EntityName for Entity {
                fn table_name(&self) -> &str {
                    #table_name
                }
            }
        }
    }

    pub fn gen_model_struct(entity: &Entity) -> TokenStream {
        let column_names_snake_case = entity.get_column_names_snake_case();
        let column_rs_types = entity.get_column_rs_types();
        quote! {
            #[derive(Clone, Debug, PartialEq, DeriveModel, DeriveActiveModel)]
            pub struct Model {
                #(pub #column_names_snake_case: #column_rs_types,)*
            }
        }
    }

    pub fn gen_column_enum(entity: &Entity) -> TokenStream {
        let column_names_camel_case = entity.get_column_names_camel_case();
        quote! {
            #[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
            pub enum Column {
                #(#column_names_camel_case,)*
            }
        }
    }

    pub fn gen_primary_key_enum(entity: &Entity) -> TokenStream {
        let primary_key_names_camel_case = entity.get_primary_key_names_camel_case();
        quote! {
            #[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
            pub enum PrimaryKey {
                #(#primary_key_names_camel_case,)*
            }
        }
    }

    pub fn gen_impl_primary_key(entity: &Entity) -> TokenStream {
        let primary_key_auto_increment = entity.get_primary_key_auto_increment();
        quote! {
            impl PrimaryKeyTrait for PrimaryKey {
                fn auto_increment() -> bool {
                    #primary_key_auto_increment
                }
            }
        }
    }

    pub fn gen_relation_enum(entity: &Entity) -> TokenStream {
        let relation_ref_tables_camel_case = entity.get_relation_ref_tables_camel_case();
        quote! {
            #[derive(Copy, Clone, Debug, EnumIter)]
            pub enum Relation {
                #(#relation_ref_tables_camel_case,)*
            }
        }
    }

    pub fn gen_impl_column_trait(entity: &Entity) -> TokenStream {
        let column_names_camel_case = entity.get_column_names_camel_case();
        let column_defs = entity.get_column_defs();
        quote! {
            impl ColumnTrait for Column {
                type EntityName = Entity;

                fn def(&self) -> ColumnDef {
                    match self {
                        #(Self::#column_names_camel_case => #column_defs,)*
                    }
                }
            }
        }
    }

    pub fn gen_impl_relation_trait(entity: &Entity) -> TokenStream {
        let relation_ref_tables_camel_case = entity.get_relation_ref_tables_camel_case();
        let relation_defs = entity.get_relation_defs();
        let quoted = if relation_ref_tables_camel_case.is_empty() {
            quote! {
                _ => panic!("No RelationDef"),
            }
        } else {
            quote! {
                #(Self::#relation_ref_tables_camel_case => #relation_defs,)*
            }
        };
        quote! {
            impl RelationTrait for Relation {
                fn def(&self) -> RelationDef {
                    match self {
                        #quoted
                    }
                }
            }
        }
    }

    pub fn gen_impl_related(entity: &Entity) -> Vec<TokenStream> {
        let camel = entity.get_relation_ref_tables_camel_case();
        let snake = entity.get_relation_ref_tables_snake_case();
        camel
            .into_iter()
            .zip(snake)
            .map(|(c, s)| {
                quote! {
                    impl Related<super::#s::Entity> for Entity {
                        fn to() -> RelationDef {
                            Relation::#c.def()
                        }
                    }
                }
            })
            .collect()
    }

    pub fn gen_impl_conjunct_related(entity: &Entity) -> Vec<TokenStream> {
        let table_name_camel_case = entity.get_table_name_camel_case_ident();
        let via_snake_case = entity.get_conjunct_relations_via_snake_case();
        let to_snake_case = entity.get_conjunct_relations_to_snake_case();
        let to_camel_case = entity.get_conjunct_relations_to_camel_case();
        via_snake_case
            .into_iter()
            .zip(to_snake_case)
            .zip(to_camel_case)
            .map(|((via_snake_case, to_snake_case), to_camel_case)| {
                quote! {
                    impl Related<super::#to_snake_case::Entity> for Entity {
                        fn to() -> RelationDef {
                            super::#via_snake_case::Relation::#to_camel_case.def()
                        }

                        fn via() -> Option<RelationDef> {
                            Some(super::#via_snake_case::Relation::#table_name_camel_case.def().rev())
                        }
                    }
                }
            })
            .collect()
    }

    pub fn gen_impl_active_model_behavior() -> TokenStream {
        quote! {
            impl ActiveModelBehavior for ActiveModel {}
        }
    }

    pub fn gen_mod(entity: &Entity) -> TokenStream {
        let table_name_snake_case_ident = entity.get_table_name_snake_case_ident();
        quote! {
            pub mod #table_name_snake_case_ident;
        }
    }

    pub fn gen_prelude_use(entity: &Entity) -> TokenStream {
        let table_name_snake_case_ident = entity.get_table_name_snake_case_ident();
        let table_name_camel_case_ident = entity.get_table_name_camel_case_ident();
        quote! {
            pub use super::#table_name_snake_case_ident::Entity as #table_name_camel_case_ident;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Column, ConjunctRelation, Entity, EntityWriter, PrimaryKey, Relation, RelationType,
    };
    use proc_macro2::TokenStream;
    use sea_query::ColumnType;
    use std::io::{self, BufRead, BufReader};

    const ENTITY_FILES: [&'static str; 5] = [
        include_str!("../../tests/entity/cake.rs"),
        include_str!("../../tests/entity/cake_filling.rs"),
        include_str!("../../tests/entity/filling.rs"),
        include_str!("../../tests/entity/fruit.rs"),
        include_str!("../../tests/entity/vendor.rs"),
    ];

    fn setup() -> Vec<Entity> {
        vec![
            Entity {
                table_name: "cake".to_owned(),
                columns: vec![
                    Column {
                        name: "id".to_owned(),
                        col_type: ColumnType::Integer(Some(11)),
                        auto_increment: true,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "name".to_owned(),
                        col_type: ColumnType::String(Some(255)),
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                ],
                relations: vec![Relation {
                    ref_table: "fruit".to_owned(),
                    columns: vec![],
                    ref_columns: vec![],
                    rel_type: RelationType::HasMany,
                }],
                conjunct_relations: vec![ConjunctRelation {
                    via: "cake_filling".to_owned(),
                    to: "filling".to_owned(),
                }],
                primary_keys: vec![PrimaryKey {
                    name: "id".to_owned(),
                }],
            },
            Entity {
                table_name: "_cake_filling_".to_owned(),
                columns: vec![
                    Column {
                        name: "cake_id".to_owned(),
                        col_type: ColumnType::Integer(Some(11)),
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "filling_id".to_owned(),
                        col_type: ColumnType::Integer(Some(11)),
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                ],
                relations: vec![
                    Relation {
                        ref_table: "cake".to_owned(),
                        columns: vec!["cake_id".to_owned()],
                        ref_columns: vec!["id".to_owned()],
                        rel_type: RelationType::BelongsTo,
                    },
                    Relation {
                        ref_table: "filling".to_owned(),
                        columns: vec!["filling_id".to_owned()],
                        ref_columns: vec!["id".to_owned()],
                        rel_type: RelationType::BelongsTo,
                    },
                ],
                conjunct_relations: vec![],
                primary_keys: vec![
                    PrimaryKey {
                        name: "cake_id".to_owned(),
                    },
                    PrimaryKey {
                        name: "filling_id".to_owned(),
                    },
                ],
            },
            Entity {
                table_name: "filling".to_owned(),
                columns: vec![
                    Column {
                        name: "id".to_owned(),
                        col_type: ColumnType::Integer(Some(11)),
                        auto_increment: true,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "name".to_owned(),
                        col_type: ColumnType::String(Some(255)),
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                ],
                relations: vec![],
                conjunct_relations: vec![ConjunctRelation {
                    via: "cake_filling".to_owned(),
                    to: "cake".to_owned(),
                }],
                primary_keys: vec![PrimaryKey {
                    name: "id".to_owned(),
                }],
            },
            Entity {
                table_name: "fruit".to_owned(),
                columns: vec![
                    Column {
                        name: "id".to_owned(),
                        col_type: ColumnType::Integer(Some(11)),
                        auto_increment: true,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "name".to_owned(),
                        col_type: ColumnType::String(Some(255)),
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "cake_id".to_owned(),
                        col_type: ColumnType::Integer(Some(11)),
                        auto_increment: false,
                        not_null: false,
                        unique: false,
                    },
                ],
                relations: vec![
                    Relation {
                        ref_table: "cake".to_owned(),
                        columns: vec!["cake_id".to_owned()],
                        ref_columns: vec!["id".to_owned()],
                        rel_type: RelationType::BelongsTo,
                    },
                    Relation {
                        ref_table: "vendor".to_owned(),
                        columns: vec![],
                        ref_columns: vec![],
                        rel_type: RelationType::HasMany,
                    },
                ],
                conjunct_relations: vec![],
                primary_keys: vec![PrimaryKey {
                    name: "id".to_owned(),
                }],
            },
            Entity {
                table_name: "vendor".to_owned(),
                columns: vec![
                    Column {
                        name: "id".to_owned(),
                        col_type: ColumnType::Integer(Some(11)),
                        auto_increment: true,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "name".to_owned(),
                        col_type: ColumnType::String(Some(255)),
                        auto_increment: false,
                        not_null: true,
                        unique: false,
                    },
                    Column {
                        name: "fruit_id".to_owned(),
                        col_type: ColumnType::Integer(Some(11)),
                        auto_increment: false,
                        not_null: false,
                        unique: false,
                    },
                ],
                relations: vec![Relation {
                    ref_table: "fruit".to_owned(),
                    columns: vec!["fruit_id".to_owned()],
                    ref_columns: vec!["id".to_owned()],
                    rel_type: RelationType::BelongsTo,
                }],
                conjunct_relations: vec![],
                primary_keys: vec![PrimaryKey {
                    name: "id".to_owned(),
                }],
            },
        ]
    }

    #[test]
    fn test_gen_code_blocks() -> io::Result<()> {
        let entities = setup();

        assert_eq!(entities.len(), ENTITY_FILES.len());

        for (i, entity) in entities.iter().enumerate() {
            let mut reader = BufReader::new(ENTITY_FILES[i].as_bytes());
            let mut lines: Vec<String> = Vec::new();

            reader.read_until(b';', &mut Vec::new())?;

            let mut line = String::new();
            while reader.read_line(&mut line)? > 0 {
                lines.push(line.to_owned());
                line.clear();
            }
            let content = lines.join("");
            let expected: TokenStream = content.parse().unwrap();
            let generated = EntityWriter::gen_code_blocks(entity)
                .into_iter()
                .skip(1)
                .fold(TokenStream::new(), |mut acc, tok| {
                    acc.extend(tok);
                    acc
                });
            assert_eq!(expected.to_string(), generated.to_string());
        }

        Ok(())
    }
}
