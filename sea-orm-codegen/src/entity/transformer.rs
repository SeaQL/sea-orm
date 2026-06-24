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
                    if col_names.len() > 1
                        && let Some(mut key_name) = index.get_index_spec().get_name()
                    {
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
    use sea_query::{ColumnDef, ForeignKey, ForeignKeyAction, Table};
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

    /// Render an entity's compact code blocks (imports and body) into a single
    /// string for substring assertions, using the same `gen_compact_code_blocks`
    /// arguments as the `validate_compact_entities` macro.
    fn render_compact(entity: &Entity) -> String {
        EntityWriter::gen_compact_code_blocks(
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
        .fold(TokenStream::new(), |mut acc, tok| {
            acc.extend(tok);
            acc
        })
        .to_string()
    }

    /// End-to-end regression for issue #2662: a foreign key referencing a bare
    /// unique *index* makes schema discovery emit the cartesian product of the
    /// referenced columns (e.g. `[a, b, a, b]`). The transformer must collapse
    /// both sides to their distinct columns so the generated `belongs_to` keeps
    /// `from.len() == to.len()`.
    #[test]
    fn multi_column_foreign_key_relation_to_unique_index() -> Result<(), Box<dyn Error>> {
        let first = Table::create()
            .table("first")
            .col(
                ColumnDef::new("id")
                    .big_integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new("a").string().not_null())
            .col(ColumnDef::new("b").string().not_null())
            .to_owned();

        let second = Table::create()
            .table("second")
            .col(
                ColumnDef::new("id")
                    .big_integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new("f_a").string().not_null())
            .col(ColumnDef::new("f_b").string().not_null())
            .foreign_key(
                ForeignKey::create()
                    .name("fk_ab")
                    .from("second", "f_a")
                    .from("second", "f_b")
                    .to("first", "a")
                    .to("first", "b")
                    // The referenced side arrives as a cartesian product because the
                    // FK targets a bare unique index (issue #2662).
                    .to("first", "a")
                    .to("first", "b")
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .to_owned();

        let entities: HashMap<_, _> = EntityTransformer::transform(vec![first, second])?
            .entities
            .into_iter()
            .map(|entity| (entity.table_name.clone(), entity))
            .collect();

        let second = entities.get("second").expect("missing entity `second`");
        let relation = second
            .relations
            .iter()
            .find(|rel| rel.ref_table == "first")
            .expect("missing belongs-to relation to `first`");
        assert!(matches!(relation.rel_type, RelationType::BelongsTo));
        assert_eq!(relation.columns, ["f_a", "f_b"]);
        assert_eq!(relation.ref_columns, ["a", "b"]);
        assert_eq!(relation.columns.len(), relation.ref_columns.len());

        // The rendered `belongs_to` must list two columns on each side -- never the
        // pre-fix four-column `to` from the cartesian product.
        let rendered = render_compact(second);
        assert!(
            rendered.contains(r#"from = "(Column::FA, Column::FB)""#),
            "unexpected `from`: {rendered}"
        );
        assert!(
            rendered.contains(r#"to = "(super::first::Column::A, super::first::Column::B)""#),
            "unexpected `to`: {rendered}"
        );
        assert!(
            !rendered.contains(
                "super::first::Column::A, super::first::Column::B, super::first::Column::A"
            ),
            "`to` still contains the four-column cartesian product: {rendered}"
        );

        Ok(())
    }

    /// The cartesian product also threatens the inverse-relation classification,
    /// which keys off the *local* column count (`rel.columns.len() ==
    /// entity.primary_keys.len()`). Deduping the local side to the right length
    /// must keep that decision correct.
    #[test]
    fn inverse_relation_classification_under_cartesian_product() -> Result<(), Box<dyn Error>> {
        // Build a `parent(a, b)` with composite PK `(a, b)` and a `child` whose FK
        // references it with a cartesian-product referenced side. `pk_is_fk`
        // toggles whether the child's own PK is the FK columns or a separate `id`.
        let build = |pk_is_fk: bool| {
            let parent = Table::create()
                .table("parent")
                .col(ColumnDef::new("a").string().not_null().primary_key())
                .col(ColumnDef::new("b").string().not_null().primary_key())
                .to_owned();

            let mut child = Table::create();
            child.table("child");
            if pk_is_fk {
                child
                    .col(ColumnDef::new("f_a").string().not_null().primary_key())
                    .col(ColumnDef::new("f_b").string().not_null().primary_key());
            } else {
                child
                    .col(
                        ColumnDef::new("id")
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new("f_a").string().not_null())
                    .col(ColumnDef::new("f_b").string().not_null());
            }
            child.foreign_key(
                ForeignKey::create()
                    .name("fk_ab")
                    // The raw cartesian product duplicates *both* sides in lockstep
                    // (`[f_a, f_a, f_b, f_b]` / `[a, b, a, b]`). The local side must be
                    // deduped back to `[f_a, f_b]`, otherwise the length-based inverse
                    // classification below sees four columns and misfires (HasMany
                    // instead of HasOne when the child's PK *is* the FK).
                    .from("child", "f_a")
                    .from("child", "f_a")
                    .from("child", "f_b")
                    .from("child", "f_b")
                    .to("parent", "a")
                    .to("parent", "b")
                    .to("parent", "a")
                    .to("parent", "b"),
            );
            vec![parent, child.to_owned()]
        };

        let inverse_rel_type = |pk_is_fk: bool| -> Result<RelationType, Box<dyn Error>> {
            let entities: HashMap<_, _> = EntityTransformer::transform(build(pk_is_fk))?
                .entities
                .into_iter()
                .map(|entity| (entity.table_name.clone(), entity))
                .collect();
            let parent = entities.get("parent").expect("missing entity `parent`");
            let rel = parent
                .relations
                .iter()
                .find(|rel| rel.ref_table == "child")
                .expect("missing inverse relation to `child`");
            Ok(rel.rel_type)
        };

        // FK is not a key on the child -> the inverse is a HasMany.
        assert!(matches!(inverse_rel_type(false)?, RelationType::HasMany));
        // The child's composite PK *is* the FK -> the inverse is a HasOne.
        assert!(matches!(inverse_rel_type(true)?, RelationType::HasOne));

        Ok(())
    }

    /// A three-column composite key is the smallest width at which the cartesian
    /// product has non-consecutive duplicates with a period > 2 on the referenced
    /// side (`[a, b, c, a, b, c, a, b, c]`); confirm the fix collapses it and the
    /// generated `to` lists exactly the three columns in order.
    #[test]
    fn three_column_foreign_key_relation_to_unique_index() -> Result<(), Box<dyn Error>> {
        let first = Table::create()
            .table("first")
            .col(
                ColumnDef::new("id")
                    .big_integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new("a").string().not_null())
            .col(ColumnDef::new("b").string().not_null())
            .col(ColumnDef::new("c").string().not_null())
            .to_owned();

        let mut fk = ForeignKey::create();
        fk.name("fk_abc")
            .from("second", "f_a")
            .from("second", "f_b")
            .from("second", "f_c");
        // The referenced side is the 3x3 cartesian product `[a, b, c] x 3`.
        for _ in 0..3 {
            fk.to("first", "a").to("first", "b").to("first", "c");
        }

        let second = Table::create()
            .table("second")
            .col(
                ColumnDef::new("id")
                    .big_integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new("f_a").string().not_null())
            .col(ColumnDef::new("f_b").string().not_null())
            .col(ColumnDef::new("f_c").string().not_null())
            .foreign_key(&mut fk)
            .to_owned();

        let entities: HashMap<_, _> = EntityTransformer::transform(vec![first, second])?
            .entities
            .into_iter()
            .map(|entity| (entity.table_name.clone(), entity))
            .collect();

        let second = entities.get("second").expect("missing entity `second`");
        let relation = second
            .relations
            .iter()
            .find(|rel| rel.ref_table == "first")
            .expect("missing belongs-to relation to `first`");
        assert_eq!(relation.columns, ["f_a", "f_b", "f_c"]);
        assert_eq!(relation.ref_columns, ["a", "b", "c"]);
        assert_eq!(relation.columns.len(), relation.ref_columns.len());

        let rendered = render_compact(second);
        assert!(
            rendered.contains(r#"from = "(Column::FA, Column::FB, Column::FC)""#),
            "unexpected `from`: {rendered}"
        );
        assert!(
            rendered.contains(
                r#"to = "(super::first::Column::A, super::first::Column::B, super::first::Column::C)""#
            ),
            "unexpected `to`: {rendered}"
        );

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
