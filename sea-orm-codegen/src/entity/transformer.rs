use crate::{
    ActiveEnum, Column, ConjunctRelation, Entity, EntityWriter, Error, PrimaryKey, Relation,
    RelationType,
};
use sea_query::TableCreateStatement;
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Clone, Debug)]
pub struct EntityTransformer;

impl EntityTransformer {
    pub fn transform(table_create_stmts: Vec<TableCreateStatement>) -> Result<EntityWriter, Error> {
        let mut enums: BTreeMap<String, ActiveEnum> = BTreeMap::new();
        let mut inverse_relations: BTreeMap<String, Vec<Relation>> = BTreeMap::new();
        let mut entities = BTreeMap::new();
        for table_create in table_create_stmts.into_iter() {
            let table_name = match table_create.get_table_name() {
                Some(table_ref) => table_ref.sea_orm_table().to_string(),
                None => {
                    return Err(Error::TransformError(
                        "Table name should not be empty".into(),
                    ));
                }
            };
            let mut primary_keys: Vec<PrimaryKey> = Vec::new();
            let mut columns: Vec<Column> = table_create
                .get_columns()
                .iter()
                .map(|col_def| {
                    let primary_key = col_def.get_column_spec().primary_key;
                    if primary_key {
                        primary_keys.push(PrimaryKey {
                            name: col_def.get_column_name(),
                        });
                    }
                    col_def.into()
                })
                .map(|mut col: Column| {
                    col.unique |= table_create
                        .get_indexes()
                        .iter()
                        .filter(|index| index.is_unique_key())
                        .map(|index| index.get_index_spec().get_column_names())
                        .filter(|col_names| col_names.len() == 1 && col_names[0] == col.name)
                        .count()
                        > 0;
                    col
                })
                .inspect(|col| {
                    if let sea_query::ColumnType::Enum { name, variants } = col.get_inner_col_type()
                    {
                        enums.insert(
                            name.to_string(),
                            ActiveEnum {
                                enum_name: name.clone(),
                                values: variants.clone(),
                            },
                        );
                    }
                })
                .collect();
            for index in table_create.get_indexes().iter() {
                if index.is_unique_key() {
                    let col_names = index.get_index_spec().get_column_names();
                    if col_names.len() > 1 {
                        if let Some(mut key_name) = index.get_index_spec().get_name() {
                            if let Some((_, suffix)) = key_name.rsplit_once('-') {
                                key_name = suffix;
                            }
                            for col_name in col_names {
                                for column in columns.iter_mut() {
                                    if column.name == col_name {
                                        column.unique_key = Some(key_name.to_owned());
                                    }
                                }
                            }
                        }
                    }
                }
            }
            let mut ref_table_counts: BTreeMap<String, usize> = BTreeMap::new();
            let relations: Vec<Relation> = table_create
                .get_foreign_key_create_stmts()
                .iter()
                .map(|fk_create_stmt| fk_create_stmt.get_foreign_key())
                .map(|tbl_fk| {
                    let ref_tbl = tbl_fk.get_ref_table().unwrap().sea_orm_table().to_string();
                    if let Some(count) = ref_table_counts.get_mut(&ref_tbl) {
                        if *count == 0 {
                            *count = 1;
                        }
                        *count += 1;
                    } else {
                        ref_table_counts.insert(ref_tbl, 0);
                    };
                    tbl_fk.into()
                })
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .map(|mut rel: Relation| {
                    rel.self_referencing = rel.ref_table == table_name;
                    if let Some(count) = ref_table_counts.get_mut(&rel.ref_table) {
                        rel.num_suffix = *count;
                        if *count > 0 {
                            *count -= 1;
                        }
                    }
                    rel
                })
                .rev()
                .collect();
            primary_keys.extend(
                table_create
                    .get_indexes()
                    .iter()
                    .filter(|index| index.is_primary_key())
                    .flat_map(|index| {
                        index
                            .get_index_spec()
                            .get_column_names()
                            .into_iter()
                            .map(|name| PrimaryKey { name })
                            .collect::<Vec<_>>()
                    }),
            );
            let entity = Entity {
                table_name: table_name.clone(),
                columns,
                relations: relations.clone(),
                conjunct_relations: vec![],
                primary_keys,
            };
            entities.insert(table_name.clone(), entity.clone());
            for mut rel in relations.into_iter() {
                // This will produce a duplicated relation
                if rel.self_referencing {
                    continue;
                }
                // This will cause compile error on the many side,
                // got relation variant but without Related<T> implemented
                if rel.num_suffix > 0 {
                    continue;
                }
                let ref_table = rel.ref_table;
                let mut unique = true;
                for column in rel.columns.iter() {
                    if !entity
                        .columns
                        .iter()
                        .filter(|col| col.unique)
                        .any(|col| col.name.as_str() == column)
                    {
                        unique = false;
                        break;
                    }
                }
                if rel.columns.len() == entity.primary_keys.len() {
                    let mut count_pk = 0;
                    for primary_key in entity.primary_keys.iter() {
                        if rel.columns.contains(&primary_key.name) {
                            count_pk += 1;
                        }
                    }
                    if count_pk == entity.primary_keys.len() {
                        unique = true;
                    }
                }
                let rel_type = if unique {
                    RelationType::HasOne
                } else {
                    RelationType::HasMany
                };
                rel.rel_type = rel_type;
                rel.ref_table = table_name.to_string();
                rel.columns = Vec::new();
                rel.ref_columns = Vec::new();
                if let Some(vec) = inverse_relations.get_mut(&ref_table) {
                    vec.push(rel);
                } else {
                    inverse_relations.insert(ref_table, vec![rel]);
                }
            }
        }
        for (tbl_name, relations) in inverse_relations.into_iter() {
            if let Some(entity) = entities.get_mut(&tbl_name) {
                for relation in relations.into_iter() {
                    let duplicate_relation = entity
                        .relations
                        .iter()
                        .any(|rel| rel.ref_table == relation.ref_table);
                    if !duplicate_relation {
                        entity.relations.push(relation);
                    }
                }
            }
        }

        // When codegen is fed with a subset of tables (e.g. via `sea-orm-cli generate entity --tables`),
        // we must not generate relations that point to entities outside this set, otherwise it will
        // produce invalid paths like `super::<missing_table>::Entity`.
        let table_names: HashSet<String> = entities.keys().cloned().collect();
        for entity in entities.values_mut() {
            entity
                .relations
                .retain(|rel| rel.self_referencing || table_names.contains(&rel.ref_table));
        }

        for table_name in entities.clone().keys() {
            let relations = match entities.get(table_name) {
                Some(entity) => {
                    let is_conjunct_relation =
                        entity.relations.len() == 2 && entity.primary_keys.len() == 2;
                    if !is_conjunct_relation {
                        continue;
                    }
                    entity.relations.clone()
                }
                None => unreachable!(),
            };
            for (i, rel) in relations.iter().enumerate() {
                let another_rel = relations.get((i == 0) as usize).unwrap();
                if let Some(entity) = entities.get_mut(&rel.ref_table) {
                    let conjunct_relation = ConjunctRelation {
                        via: table_name.clone(),
                        to: another_rel.ref_table.clone(),
                    };
                    entity.conjunct_relations.push(conjunct_relation);
                }
            }
        }
        Ok(EntityWriter {
            entities: entities
                .into_values()
                .map(|mut v| {
                    // Filter duplicated conjunct relations
                    let duplicated_to: Vec<_> = v
                        .conjunct_relations
                        .iter()
                        .fold(HashMap::new(), |mut acc, conjunct_relation| {
                            acc.entry(conjunct_relation.to.clone())
                                .and_modify(|c| *c += 1)
                                .or_insert(1);
                            acc
                        })
                        .into_iter()
                        .filter(|(_, v)| v > &1)
                        .map(|(k, _)| k)
                        .collect();
                    v.conjunct_relations
                        .retain(|conjunct_relation| !duplicated_to.contains(&conjunct_relation.to));

                    // Skip `impl Related ... { fn to() ... }` implementation block,
                    // if the same related entity is being referenced by a conjunct relation
                    v.relations.iter_mut().for_each(|relation| {
                        if v.conjunct_relations
                            .iter()
                            .any(|conjunct_relation| conjunct_relation.to == relation.ref_table)
                        {
                            relation.impl_related = false;
                        }
                    });

                    // Sort relation vectors
                    v.relations.sort_by(|a, b| a.ref_table.cmp(&b.ref_table));
                    v.conjunct_relations.sort_by(|a, b| a.to.cmp(&b.to));
                    v
                })
                .collect(),
            enums,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use proc_macro2::TokenStream;
    use sea_orm::{DbBackend, Schema};
    use sea_query::{ColumnDef, ForeignKey, Table};
    use std::{
        error::Error,
        io::{self, BufRead, BufReader},
    };

    #[test]
    fn duplicated_many_to_many_paths() -> Result<(), Box<dyn Error>> {
        use crate::tests_cfg::duplicated_many_to_many_paths::*;
        let schema = Schema::new(DbBackend::Postgres);

        validate_compact_entities(
            vec![
                schema.create_table_from_entity(bills::Entity),
                schema.create_table_from_entity(users::Entity),
                schema.create_table_from_entity(users_saved_bills::Entity),
                schema.create_table_from_entity(users_votes::Entity),
            ],
            vec![
                (
                    "bills",
                    include_str!("../tests_cfg/duplicated_many_to_many_paths/bills.rs"),
                ),
                (
                    "users",
                    include_str!("../tests_cfg/duplicated_many_to_many_paths/users.rs"),
                ),
                (
                    "users_saved_bills",
                    include_str!("../tests_cfg/duplicated_many_to_many_paths/users_saved_bills.rs"),
                ),
                (
                    "users_votes",
                    include_str!("../tests_cfg/duplicated_many_to_many_paths/users_votes.rs"),
                ),
            ],
        )
    }

    #[test]
    fn many_to_many() -> Result<(), Box<dyn Error>> {
        use crate::tests_cfg::many_to_many::*;
        let schema = Schema::new(DbBackend::Postgres);

        validate_compact_entities(
            vec![
                schema.create_table_from_entity(bills::Entity),
                schema.create_table_from_entity(users::Entity),
                schema.create_table_from_entity(users_votes::Entity),
            ],
            vec![
                ("bills", include_str!("../tests_cfg/many_to_many/bills.rs")),
                ("users", include_str!("../tests_cfg/many_to_many/users.rs")),
                (
                    "users_votes",
                    include_str!("../tests_cfg/many_to_many/users_votes.rs"),
                ),
            ],
        )
    }

    #[test]
    fn many_to_many_multiple() -> Result<(), Box<dyn Error>> {
        use crate::tests_cfg::many_to_many_multiple::*;
        let schema = Schema::new(DbBackend::Postgres);

        validate_compact_entities(
            vec![
                schema.create_table_from_entity(bills::Entity),
                schema.create_table_from_entity(users::Entity),
                schema.create_table_from_entity(users_votes::Entity),
            ],
            vec![
                (
                    "bills",
                    include_str!("../tests_cfg/many_to_many_multiple/bills.rs"),
                ),
                (
                    "users",
                    include_str!("../tests_cfg/many_to_many_multiple/users.rs"),
                ),
                (
                    "users_votes",
                    include_str!("../tests_cfg/many_to_many_multiple/users_votes.rs"),
                ),
            ],
        )
    }

    #[test]
    fn self_referencing() -> Result<(), Box<dyn Error>> {
        use crate::tests_cfg::self_referencing::*;
        let schema = Schema::new(DbBackend::Postgres);

        validate_compact_entities(
            vec![
                schema.create_table_from_entity(bills::Entity),
                schema.create_table_from_entity(users::Entity),
            ],
            vec![
                (
                    "bills",
                    include_str!("../tests_cfg/self_referencing/bills.rs"),
                ),
                (
                    "users",
                    include_str!("../tests_cfg/self_referencing/users.rs"),
                ),
            ],
        )
    }

    #[test]
    fn test_indexes_transform() -> Result<(), Box<dyn Error>> {
        let schema = Schema::new(DbBackend::Postgres);

        validate_compact_entities(
            vec![
                schema.create_table_with_index_from_entity(
                    crate::tests_cfg::compact::indexes::Entity,
                ),
            ],
            vec![("indexes", include_str!("../tests_cfg/compact/indexes.rs"))],
        )?;

        validate_dense_entities(
            vec![
                schema
                    .create_table_with_index_from_entity(crate::tests_cfg::dense::indexes::Entity),
            ],
            vec![("indexes", include_str!("../tests_cfg/dense/indexes.rs"))],
        )?;

        Ok(())
    }

    #[test]
    fn filter_relations_to_missing_entities() -> Result<(), Box<dyn Error>> {
        let parent_stmt = || {
            Table::create()
                .table("parent")
                .col(
                    ColumnDef::new("id")
                        .integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .to_owned()
        };

        let child_stmt = || {
            Table::create()
                .table("child")
                .col(
                    ColumnDef::new("id")
                        .integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(ColumnDef::new("parent_id").integer().not_null())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk-child-parent_id")
                        .from("child", "parent_id")
                        .to("parent", "id"),
                )
                .to_owned()
        };

        let entities: HashMap<_, _> =
            EntityTransformer::transform(vec![parent_stmt(), child_stmt()])?
                .entities
                .into_iter()
                .map(|entity| (entity.table_name.clone(), entity))
                .collect();

        let child = entities.get("child").expect("missing entity `child`");
        assert_eq!(child.relations.len(), 1);
        assert_eq!(child.relations[0].ref_table, "parent");

        let entities: HashMap<_, _> = EntityTransformer::transform(vec![child_stmt()])?
            .entities
            .into_iter()
            .map(|entity| (entity.table_name.clone(), entity))
            .collect();

        let child = entities.get("child").expect("missing entity `child`");
        assert!(child.relations.is_empty());

        Ok(())
    }

    #[test]
    fn filter_conjunct_relations_to_missing_entities() -> Result<(), Box<dyn Error>> {
        let user_stmt = || {
            Table::create()
                .table("user")
                .col(
                    ColumnDef::new("id")
                        .integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .to_owned()
        };

        let role_stmt = || {
            Table::create()
                .table("role")
                .col(
                    ColumnDef::new("id")
                        .integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .to_owned()
        };

        let user_role_stmt = || {
            Table::create()
                .table("user_role")
                .col(ColumnDef::new("user_id").integer().not_null().primary_key())
                .col(ColumnDef::new("role_id").integer().not_null().primary_key())
                .foreign_key(
                    ForeignKey::create()
                        .name("fk-user_role-user_id")
                        .from("user_role", "user_id")
                        .to("user", "id"),
                )
                .foreign_key(
                    ForeignKey::create()
                        .name("fk-user_role-role_id")
                        .from("user_role", "role_id")
                        .to("role", "id"),
                )
                .to_owned()
        };

        let entities: HashMap<_, _> =
            EntityTransformer::transform(vec![user_stmt(), role_stmt(), user_role_stmt()])?
                .entities
                .into_iter()
                .map(|entity| (entity.table_name.clone(), entity))
                .collect();

        let user = entities.get("user").expect("missing entity `user`");
        assert!(user.conjunct_relations.iter().any(|conjunct_relation| {
            conjunct_relation.via == "user_role" && conjunct_relation.to == "role"
        }));

        let entities: HashMap<_, _> =
            EntityTransformer::transform(vec![user_stmt(), user_role_stmt()])?
                .entities
                .into_iter()
                .map(|entity| (entity.table_name.clone(), entity))
                .collect();

        let user = entities.get("user").expect("missing entity `user`");
        assert!(user.conjunct_relations.is_empty());

        let user_role = entities
            .get("user_role")
            .expect("missing entity `user_role`");
        assert_eq!(user_role.relations.len(), 1);
        assert_eq!(user_role.relations[0].ref_table, "user");

        Ok(())
    }

    macro_rules! validate_entities_fn {
        ($fn_name: ident, $method: ident) => {
            fn $fn_name(
                table_create_stmts: Vec<TableCreateStatement>,
                files: Vec<(&str, &str)>,
            ) -> Result<(), Box<dyn Error>> {
                let entities: HashMap<_, _> = EntityTransformer::transform(table_create_stmts)?
                    .entities
                    .into_iter()
                    .map(|entity| (entity.table_name.clone(), entity))
                    .collect();

                for (entity_name, file_content) in files {
                    let entity = entities
                        .get(entity_name)
                        .expect("Forget to add entity to the list");

                    assert_eq!(
                        parse_from_file(file_content.as_bytes())?.to_string(),
                        EntityWriter::$method(
                            entity,
                            &crate::WithSerde::None,
                            &Default::default(),
                            &None,
                            false,
                            false,
                            &Default::default(),
                            &Default::default(),
                            &Default::default(),
                            false,
                            true,
                        )
                        .into_iter()
                        .skip(1)
                        .fold(TokenStream::new(), |mut acc, tok| {
                            acc.extend(tok);
                            acc
                        })
                        .to_string()
                    );
                }

                Ok(())
            }
        };
    }

    validate_entities_fn!(validate_compact_entities, gen_compact_code_blocks);
    validate_entities_fn!(validate_dense_entities, gen_dense_code_blocks);

    fn parse_from_file<R>(inner: R) -> io::Result<TokenStream>
    where
        R: io::Read,
    {
        let mut reader = BufReader::new(inner);
        let mut lines: Vec<String> = Vec::new();

        reader.read_until(b';', &mut Vec::new())?;

        let mut line = String::new();
        while reader.read_line(&mut line)? > 0 {
            lines.push(line.to_owned());
            line.clear();
        }
        let content = lines.join("");
        Ok(content.parse().unwrap())
    }
}
