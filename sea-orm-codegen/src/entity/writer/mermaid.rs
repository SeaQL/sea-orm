use std::collections::{BTreeSet, HashSet};
use std::fmt::Write;

use sea_query::ColumnType;

use crate::{Entity, RelationType};

use super::EntityWriter;

impl EntityWriter {
    pub fn generate_er_diagram(&self) -> String {
        let mut out = String::from("erDiagram\n");

        let pk_sets: Vec<HashSet<&str>> = self
            .entities
            .iter()
            .map(|e| e.primary_keys.iter().map(|pk| pk.name.as_str()).collect())
            .collect();

        let fk_sets: Vec<HashSet<&str>> = self
            .entities
            .iter()
            .map(|e| {
                e.relations
                    .iter()
                    .filter(|r| matches!(r.rel_type, RelationType::BelongsTo))
                    .flat_map(|r| r.columns.iter().map(String::as_str))
                    .collect()
            })
            .collect();

        for (i, entity) in self.entities.iter().enumerate() {
            write_entity_block(&mut out, entity, &pk_sets[i], &fk_sets[i]);
        }

        let mut emitted: BTreeSet<String> = BTreeSet::new();

        for entity in &self.entities {
            write_relations(&mut out, entity, &mut emitted);
        }

        out
    }
}

fn write_entity_block(out: &mut String, entity: &Entity, pks: &HashSet<&str>, fks: &HashSet<&str>) {
    let _ = writeln!(out, "    {} {{", entity.table_name);

    for col in &entity.columns {
        let type_name = col_type_name(&col.col_type);
        let is_pk = pks.contains(col.name.as_str());
        let is_fk = fks.contains(col.name.as_str());
        let is_uk = col.unique || col.unique_key.is_some();

        let constraint = match (is_pk, is_fk, is_uk) {
            (true, true, _) => " PK,FK",
            (true, false, _) => " PK",
            (false, true, true) => " FK,UK",
            (false, true, false) => " FK",
            (false, false, true) => " UK",
            (false, false, false) => "",
        };

        let _ = writeln!(out, "        {} {}{}", type_name, col.name, constraint);
    }

    let _ = writeln!(out, "    }}");
}

fn write_relations(out: &mut String, entity: &Entity, emitted: &mut BTreeSet<String>) {
    for rel in &entity.relations {
        let (left, right, cardinality, label) = match rel.rel_type {
            RelationType::BelongsTo => (
                &entity.table_name,
                &rel.ref_table,
                "}o--||",
                rel.columns.join(", "),
            ),
            RelationType::HasOne => continue,
            RelationType::HasMany => continue,
        };

        let key = format!("{left} {cardinality} {right} : \"{label}\"");
        if emitted.insert(key.clone()) {
            let _ = writeln!(out, "    {key}");
        }
    }

    for conj in &entity.conjunct_relations {
        let left = &entity.table_name;
        let right = &conj.to;
        let label = format!("[{}]", conj.via);

        let key = if left <= right {
            format!("{left} }}o--o{{ {right} : \"{label}\"")
        } else {
            format!("{right} }}o--o{{ {left} : \"{label}\"")
        };

        if emitted.insert(key.clone()) {
            let _ = writeln!(out, "    {key}");
        }
    }
}

fn col_type_name(col_type: &ColumnType) -> &str {
    #[allow(unreachable_patterns)]
    match col_type {
        ColumnType::Char(_) => "char",
        ColumnType::String(_) => "varchar",
        ColumnType::Text => "text",
        ColumnType::TinyInteger => "tinyint",
        ColumnType::SmallInteger => "smallint",
        ColumnType::Integer => "int",
        ColumnType::BigInteger => "bigint",
        ColumnType::TinyUnsigned => "tinyint_unsigned",
        ColumnType::SmallUnsigned => "smallint_unsigned",
        ColumnType::Unsigned => "int_unsigned",
        ColumnType::BigUnsigned => "bigint_unsigned",
        ColumnType::Float => "float",
        ColumnType::Double => "double",
        ColumnType::Decimal(_) => "decimal",
        ColumnType::Money(_) => "money",
        ColumnType::DateTime => "datetime",
        ColumnType::Timestamp => "timestamp",
        ColumnType::TimestampWithTimeZone => "timestamptz",
        ColumnType::Time => "time",
        ColumnType::Date => "date",
        ColumnType::Year => "year",
        ColumnType::Binary(_) | ColumnType::VarBinary(_) | ColumnType::Blob => "blob",
        ColumnType::Boolean => "bool",
        ColumnType::Json | ColumnType::JsonBinary => "json",
        ColumnType::Uuid => "uuid",
        ColumnType::Enum { .. } => "enum",
        ColumnType::Array(_) => "array",
        ColumnType::Vector(_) => "vector",
        ColumnType::Bit(_) | ColumnType::VarBit(_) => "bit",
        ColumnType::Cidr => "cidr",
        ColumnType::Inet => "inet",
        ColumnType::MacAddr => "macaddr",
        ColumnType::LTree => "ltree",
        ColumnType::Interval(_, _) => "interval",
        ColumnType::Custom(_) => "custom",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use sea_query::{ColumnType, StringLen};

    use crate::{
        Column, ConjunctRelation, Entity, EntityWriter, PrimaryKey, Relation, RelationType,
    };

    fn setup_blog_schema() -> EntityWriter {
        EntityWriter {
            entities: vec![
                Entity {
                    table_name: "user".to_owned(),
                    columns: vec![
                        Column {
                            name: "id".to_owned(),
                            col_type: ColumnType::Integer,
                            auto_increment: true,
                            not_null: true,
                            unique: false,
                            unique_key: None,
                        },
                        Column {
                            name: "name".to_owned(),
                            col_type: ColumnType::String(StringLen::N(255)),
                            auto_increment: false,
                            not_null: true,
                            unique: false,
                            unique_key: None,
                        },
                        Column {
                            name: "email".to_owned(),
                            col_type: ColumnType::String(StringLen::N(255)),
                            auto_increment: false,
                            not_null: true,
                            unique: true,
                            unique_key: None,
                        },
                        Column {
                            name: "parent_id".to_owned(),
                            col_type: ColumnType::Integer,
                            auto_increment: false,
                            not_null: false,
                            unique: false,
                            unique_key: None,
                        },
                    ],
                    relations: vec![
                        Relation {
                            ref_table: "post".to_owned(),
                            columns: vec![],
                            ref_columns: vec![],
                            rel_type: RelationType::HasMany,
                            on_delete: None,
                            on_update: None,
                            self_referencing: false,
                            num_suffix: 0,
                            impl_related: true,
                        },
                        Relation {
                            ref_table: "user".to_owned(),
                            columns: vec!["parent_id".to_owned()],
                            ref_columns: vec!["id".to_owned()],
                            rel_type: RelationType::BelongsTo,
                            on_delete: None,
                            on_update: None,
                            self_referencing: true,
                            num_suffix: 0,
                            impl_related: true,
                        },
                    ],
                    conjunct_relations: vec![],
                    primary_keys: vec![PrimaryKey {
                        name: "id".to_owned(),
                    }],
                },
                Entity {
                    table_name: "post".to_owned(),
                    columns: vec![
                        Column {
                            name: "id".to_owned(),
                            col_type: ColumnType::Integer,
                            auto_increment: true,
                            not_null: true,
                            unique: false,
                            unique_key: None,
                        },
                        Column {
                            name: "title".to_owned(),
                            col_type: ColumnType::Text,
                            auto_increment: false,
                            not_null: true,
                            unique: false,
                            unique_key: None,
                        },
                        Column {
                            name: "user_id".to_owned(),
                            col_type: ColumnType::Integer,
                            auto_increment: false,
                            not_null: true,
                            unique: false,
                            unique_key: None,
                        },
                    ],
                    relations: vec![Relation {
                        ref_table: "user".to_owned(),
                        columns: vec!["user_id".to_owned()],
                        ref_columns: vec!["id".to_owned()],
                        rel_type: RelationType::BelongsTo,
                        on_delete: None,
                        on_update: None,
                        self_referencing: false,
                        num_suffix: 0,
                        impl_related: true,
                    }],
                    conjunct_relations: vec![ConjunctRelation {
                        via: "post_tag".to_owned(),
                        to: "tag".to_owned(),
                    }],
                    primary_keys: vec![PrimaryKey {
                        name: "id".to_owned(),
                    }],
                },
                Entity {
                    table_name: "tag".to_owned(),
                    columns: vec![
                        Column {
                            name: "id".to_owned(),
                            col_type: ColumnType::Integer,
                            auto_increment: true,
                            not_null: true,
                            unique: false,
                            unique_key: None,
                        },
                        Column {
                            name: "name".to_owned(),
                            col_type: ColumnType::String(StringLen::N(100)),
                            auto_increment: false,
                            not_null: true,
                            unique: true,
                            unique_key: None,
                        },
                    ],
                    relations: vec![],
                    conjunct_relations: vec![ConjunctRelation {
                        via: "post_tag".to_owned(),
                        to: "post".to_owned(),
                    }],
                    primary_keys: vec![PrimaryKey {
                        name: "id".to_owned(),
                    }],
                },
                Entity {
                    table_name: "post_tag".to_owned(),
                    columns: vec![
                        Column {
                            name: "post_id".to_owned(),
                            col_type: ColumnType::Integer,
                            auto_increment: false,
                            not_null: true,
                            unique: false,
                            unique_key: None,
                        },
                        Column {
                            name: "tag_id".to_owned(),
                            col_type: ColumnType::Integer,
                            auto_increment: false,
                            not_null: true,
                            unique: false,
                            unique_key: None,
                        },
                    ],
                    relations: vec![
                        Relation {
                            ref_table: "post".to_owned(),
                            columns: vec!["post_id".to_owned()],
                            ref_columns: vec!["id".to_owned()],
                            rel_type: RelationType::BelongsTo,
                            on_delete: None,
                            on_update: None,
                            self_referencing: false,
                            num_suffix: 0,
                            impl_related: true,
                        },
                        Relation {
                            ref_table: "tag".to_owned(),
                            columns: vec!["tag_id".to_owned()],
                            ref_columns: vec!["id".to_owned()],
                            rel_type: RelationType::BelongsTo,
                            on_delete: None,
                            on_update: None,
                            self_referencing: false,
                            num_suffix: 0,
                            impl_related: true,
                        },
                    ],
                    conjunct_relations: vec![],
                    primary_keys: vec![
                        PrimaryKey {
                            name: "post_id".to_owned(),
                        },
                        PrimaryKey {
                            name: "tag_id".to_owned(),
                        },
                    ],
                },
            ],
            enums: BTreeMap::new(),
        }
    }

    #[test]
    fn test_generate_er_diagram() {
        let writer = setup_blog_schema();
        let diagram = writer.generate_er_diagram();

        let expected = r#"erDiagram
    user {
        int id PK
        varchar name
        varchar email UK
        int parent_id FK
    }
    post {
        int id PK
        text title
        int user_id FK
    }
    tag {
        int id PK
        varchar name UK
    }
    post_tag {
        int post_id PK,FK
        int tag_id PK,FK
    }
    user }o--|| user : "parent_id"
    post }o--|| user : "user_id"
    post }o--o{ tag : "[post_tag]"
    post_tag }o--|| post : "post_id"
    post_tag }o--|| tag : "tag_id"
"#;

        assert_eq!(diagram, expected);
    }

    #[test]
    fn test_er_diagram_deduplicates_m2m() {
        let writer = setup_blog_schema();
        let diagram = writer.generate_er_diagram();

        let m2m_count = diagram.matches("}o--o{").count();
        assert_eq!(m2m_count, 1, "M-N relation should appear only once");
    }
}
