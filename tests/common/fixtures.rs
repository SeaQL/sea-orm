// ---------------------------------------------------------------------------
// Enum fixtures
// ---------------------------------------------------------------------------

pub mod enum_v1 {
    use sea_orm::entity::prelude::*;

    #[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
    #[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "disc_status")]
    pub enum Status {
        #[sea_orm(string_value = "active")]
        Active,
        #[sea_orm(string_value = "inactive")]
        Inactive,
    }

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "disc_enum_table")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub status: Status,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod enum_v2 {
    use sea_orm::entity::prelude::*;

    #[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
    #[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "disc_status")]
    pub enum Status {
        #[sea_orm(string_value = "active")]
        Active,
        #[sea_orm(string_value = "inactive")]
        Inactive,
        #[sea_orm(string_value = "pending")]
        Pending,
    }

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "disc_enum_table")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub status: Status,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod enum_renamed {
    use sea_orm::entity::prelude::*;

    #[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
    #[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "disc_state")]
    pub enum State {
        #[sea_orm(string_value = "active")]
        Active,
        #[sea_orm(string_value = "inactive")]
        Inactive,
    }

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "disc_enum_table")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub status: State,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

// ---------------------------------------------------------------------------
// Widget fixtures (column drop / no-drop)
// ---------------------------------------------------------------------------

pub mod widget_v1 {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "sync_widget")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub label: String,
        pub weight: i32,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod widget_v2 {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "sync_widget")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub label: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

// ---------------------------------------------------------------------------
// Combo fixtures (rename detection)
// ---------------------------------------------------------------------------

pub mod combo_v1 {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "sync_combo")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub old_field: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod combo_v2 {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "sync_combo")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub new_field: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

// ---------------------------------------------------------------------------
// Category / Article fixtures (FK drop)
// ---------------------------------------------------------------------------

pub mod category_v1 {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "sync_category")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod article_v1 {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "sync_article")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub title: String,
        #[sea_orm(belongs_to, from = "category_id", to = "id")]
        pub category: HasOne<super::category_v1::Entity>,
        pub category_id: i32,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod article_v2 {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "sync_article")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub title: String,
        pub category_id: i32,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

// ---------------------------------------------------------------------------
// Tag fixture (orphan table)
// ---------------------------------------------------------------------------

pub mod tag_v1 {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "sync_tag")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub slug: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

// ---------------------------------------------------------------------------
// Column addition fixtures
// ---------------------------------------------------------------------------

pub mod coltest_v1 {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "disc_coltest")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod coltest_v2_nullable {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "disc_coltest")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
        pub bio: Option<String>,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod coltest_v2_notnull {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "disc_coltest")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
        pub age: i32,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod coltest_v2_notnull_default {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "disc_coltest")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
        #[sea_orm(default_value = 0)]
        pub score: i32,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod coltest_v2_multi {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "disc_coltest")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
        pub bio: Option<String>,
        pub age: i32,
        #[sea_orm(default_value = 0)]
        pub score: i32,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

// ---------------------------------------------------------------------------
// Parent / Child fixtures (FK ordering in drops)
// ---------------------------------------------------------------------------

/// Simple parent table with no FK dependencies.
pub mod fk_parent_v1 {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "sync_fk_parent")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

/// Grandparent table — the root of a three-level FK chain used in complex drop-sequence tests.
pub mod fk_grandparent_v1 {
    use sea_orm::entity::prelude::*;

    #[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
    #[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "drop_seq_status")]
    pub enum Status {
        #[sea_orm(string_value = "active")]
        Active,
        #[sea_orm(string_value = "inactive")]
        Inactive,
    }

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "drop_seq_gp")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub status: Status,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

/// Middle table — has an FK to `drop_seq_gp` and is itself referenced by `drop_seq_child`.
pub mod fk_mid_v1 {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "drop_seq_mid")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(belongs_to, from = "gp_id", to = "id")]
        pub grandparent: HasOne<super::fk_grandparent_v1::Entity>,
        pub gp_id: i32,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

/// Child table with an FK pointing to `sync_fk_parent`.
pub mod fk_child_v1 {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "sync_fk_child")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(belongs_to, from = "parent_id", to = "id")]
        pub parent: HasOne<super::fk_parent_v1::Entity>,
        pub parent_id: i32,
    }

    impl ActiveModelBehavior for ActiveModel {}
}

/// Leaf table — deepest in the three-level chain, references `drop_seq_mid`.
pub mod fk_leaf_v1 {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "drop_seq_leaf")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        #[sea_orm(belongs_to, from = "mid_id", to = "id")]
        pub mid: HasOne<super::fk_mid_v1::Entity>,
        pub mid_id: i32,
    }

    impl ActiveModelBehavior for ActiveModel {}
}
