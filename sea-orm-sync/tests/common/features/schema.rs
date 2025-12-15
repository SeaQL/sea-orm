use super::*;
use crate::common::setup::{
    create_enum, create_table, create_table_from_entity, create_table_without_asserts,
};
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DbBackend, DbConn, EntityName, ExecResult, Schema,
    error::*, sea_query,
};
use sea_query::{
    Alias, ColumnDef, ColumnType, ForeignKeyCreateStatement, IntoIden, IntoTableRef, StringLen,
    extension::postgres::Type,
};

pub fn create_tables(db: &DatabaseConnection) -> Result<(), DbErr> {
    let db_backend = db.get_database_backend();

    create_log_table(db)?;
    create_metadata_table(db)?;
    create_repository_table(db)?;
    create_self_join_table(db)?;
    create_byte_primary_key_table(db)?;
    create_satellites_table(db)?;
    create_transaction_log_table(db)?;

    let create_enum_stmts = match db_backend {
        DbBackend::MySql | DbBackend::Sqlite => Vec::new(),
        DbBackend::Postgres => {
            let schema = Schema::new(db_backend);
            let enum_create_stmt = Type::create()
                .as_enum("tea")
                .values(["EverydayTea", "BreakfastTea", "AfternoonTea"])
                .to_owned();
            assert_eq!(
                db_backend.build(&enum_create_stmt),
                db_backend.build(&schema.create_enum_from_active_enum::<Tea>().unwrap())
            );
            vec![enum_create_stmt]
        }
        db => {
            return Err(DbErr::BackendNotSupported {
                db: db.as_str(),
                ctx: "create_byte_primary_key_table",
            });
        }
    };
    create_enum(db, &create_enum_stmts, ActiveEnum)?;

    create_active_enum_table(db)?;
    create_active_enum_child_table(db)?;
    create_insert_default_table(db)?;
    create_pi_table(db)?;
    create_uuid_fmt_table(db)?;
    create_edit_log_table(db)?;
    create_teas_table(db)?;
    create_binary_table(db)?;
    if matches!(db_backend, DbBackend::Postgres) {
        create_bits_table(db)?;
    }
    create_dyn_table_name_lazy_static_table(db)?;
    create_value_type_table(db)?;

    create_json_vec_table(db)?;
    create_json_struct_table(db)?;
    create_json_string_vec_table(db)?;
    create_json_struct_vec_table(db)?;

    if DbBackend::Postgres == db_backend {
        create_value_type_postgres_table(db)?;
        create_collection_table(db)?;
        create_event_trigger_table(db)?;
        create_categories_table(db)?;
        #[cfg(feature = "postgres-vector")]
        create_embedding_table(db)?;
        create_host_network_table(db)?;
    }

    Ok(())
}

pub fn create_log_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let stmt = sea_query::Table::create()
        .table(applog::Entity)
        .comment("app logs")
        .col(
            ColumnDef::new(applog::Column::Id)
                .integer()
                .not_null()
                .comment("ID")
                .auto_increment()
                .primary_key(),
        )
        .col(
            ColumnDef::new(applog::Column::Action)
                .string()
                .not_null()
                .comment("action"),
        )
        .col(
            ColumnDef::new(applog::Column::Json)
                .json()
                .not_null()
                .comment("action data"),
        )
        .col(
            ColumnDef::new(applog::Column::CreatedAt)
                .timestamp_with_time_zone()
                .not_null()
                .comment("create time"),
        )
        .to_owned();

    create_table(db, &stmt, Applog)
}

pub fn create_metadata_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let stmt = sea_query::Table::create()
        .table(metadata::Entity)
        .col(
            ColumnDef::new(metadata::Column::Uuid)
                .uuid()
                .not_null()
                .primary_key(),
        )
        .col(ColumnDef::new(metadata::Column::Type).string().not_null())
        .col(ColumnDef::new(metadata::Column::Key).string().not_null())
        .col(ColumnDef::new(metadata::Column::Value).string().not_null())
        .col(
            ColumnDef::new_with_type(
                metadata::Column::Bytes,
                ColumnType::VarBinary(StringLen::N(32)),
            )
            .not_null(),
        )
        .col(ColumnDef::new(metadata::Column::Date).date())
        .col(ColumnDef::new(metadata::Column::Time).time())
        .to_owned();

    create_table(db, &stmt, Metadata)
}

pub fn create_repository_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let stmt = sea_query::Table::create()
        .table(repository::Entity)
        .col(
            ColumnDef::new(repository::Column::Id)
                .string()
                .not_null()
                .primary_key(),
        )
        .col(
            ColumnDef::new(repository::Column::Owner)
                .string()
                .not_null(),
        )
        .col(ColumnDef::new(repository::Column::Name).string().not_null())
        .col(ColumnDef::new(repository::Column::Description).string())
        .to_owned();

    create_table(db, &stmt, Repository)
}

pub fn create_self_join_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let stmt = sea_query::Table::create()
        .table(self_join::Entity)
        .col(
            ColumnDef::new(self_join::Column::Uuid)
                .uuid()
                .not_null()
                .primary_key(),
        )
        .col(ColumnDef::new(self_join::Column::UuidRef).uuid())
        .col(ColumnDef::new(self_join::Column::Time).time())
        .foreign_key(
            ForeignKeyCreateStatement::new()
                .name("fk-self_join-uuid_ref")
                .from_tbl(SelfJoin)
                .from_col(self_join::Column::UuidRef)
                .to_tbl(SelfJoin)
                .to_col(self_join::Column::Uuid),
        )
        .to_owned();

    create_table(db, &stmt, SelfJoin)
}

pub fn create_byte_primary_key_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let mut primary_key_col = ColumnDef::new(byte_primary_key::Column::Id);
    match db.get_database_backend() {
        DbBackend::MySql => primary_key_col.binary_len(3),
        DbBackend::Sqlite | DbBackend::Postgres => primary_key_col.binary(),
        db => {
            return Err(DbErr::BackendNotSupported {
                db: db.as_str(),
                ctx: "create_byte_primary_key_table",
            });
        }
    };

    let stmt = sea_query::Table::create()
        .table(byte_primary_key::Entity)
        .col(primary_key_col.not_null().primary_key())
        .col(
            ColumnDef::new(byte_primary_key::Column::Value)
                .string()
                .not_null(),
        )
        .to_owned();

    create_table_without_asserts(db, &stmt)
}

pub fn create_active_enum_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let create_table_stmt = sea_query::Table::create()
        .table(active_enum::Entity.table_ref())
        .col(
            ColumnDef::new(active_enum::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(active_enum::Column::Category).string_len(1))
        .col(ColumnDef::new(active_enum::Column::Color).integer())
        .col(ColumnDef::new(active_enum::Column::Tea).enumeration(
            TeaEnum,
            [
                TeaVariant::EverydayTea,
                TeaVariant::BreakfastTea,
                TeaVariant::AfternoonTea,
            ],
        ))
        .to_owned();

    create_table(db, &create_table_stmt, ActiveEnum)
}

pub fn create_active_enum_child_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let create_table_stmt = sea_query::Table::create()
        .table(active_enum_child::Entity.table_ref())
        .col(
            ColumnDef::new(active_enum_child::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(
            ColumnDef::new(active_enum_child::Column::ParentId)
                .integer()
                .not_null(),
        )
        .col(ColumnDef::new(active_enum_child::Column::Category).string_len(1))
        .col(ColumnDef::new(active_enum_child::Column::Color).integer())
        .col(ColumnDef::new(active_enum_child::Column::Tea).enumeration(
            TeaEnum,
            [
                TeaVariant::EverydayTea,
                TeaVariant::BreakfastTea,
                TeaVariant::AfternoonTea,
            ],
        ))
        .foreign_key(
            ForeignKeyCreateStatement::new()
                .name("fk-active_enum_child-active_enum")
                .from_tbl(ActiveEnumChild)
                .from_col(active_enum_child::Column::ParentId)
                .to_tbl(if cfg!(feature = "sqlx-postgres") {
                    ("public", ActiveEnum).into_table_ref()
                } else {
                    ActiveEnum.into_table_ref()
                })
                .to_col(active_enum::Column::Id),
        )
        .to_owned();

    create_table(db, &create_table_stmt, ActiveEnumChild)
}

pub fn create_satellites_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let stmt = sea_query::Table::create()
        .table(satellite::Entity)
        .col(
            ColumnDef::new(satellite::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(
            ColumnDef::new(satellite::Column::SatelliteName)
                .string()
                .not_null(),
        )
        .col(
            ColumnDef::new(satellite::Column::LaunchDate)
                .timestamp_with_time_zone()
                .not_null()
                .default("2022-01-26 16:24:00"),
        )
        .col(
            ColumnDef::new(satellite::Column::DeploymentDate)
                .timestamp_with_time_zone()
                .not_null()
                .default("2022-01-26 16:24:00"),
        )
        .to_owned();

    create_table(db, &stmt, Satellite)
}

pub fn create_transaction_log_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let stmt = sea_query::Table::create()
        .table(transaction_log::Entity)
        .col(
            ColumnDef::new(transaction_log::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(
            ColumnDef::new(transaction_log::Column::Date)
                .date()
                .not_null(),
        )
        .col(
            ColumnDef::new(transaction_log::Column::Time)
                .time()
                .not_null(),
        )
        .col(
            ColumnDef::new(transaction_log::Column::DateTime)
                .date_time()
                .not_null(),
        )
        .col(
            ColumnDef::new(transaction_log::Column::DateTimeTz)
                .timestamp_with_time_zone()
                .not_null(),
        )
        .to_owned();

    create_table(db, &stmt, TransactionLog)
}

pub fn create_insert_default_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let create_table_stmt = sea_query::Table::create()
        .table(insert_default::Entity.table_ref())
        .col(
            ColumnDef::new(insert_default::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .to_owned();

    create_table(db, &create_table_stmt, InsertDefault)
}

pub fn create_json_vec_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let create_table_stmt = sea_query::Table::create()
        .table(json_vec::Entity.table_ref())
        .col(
            ColumnDef::new(json_vec::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(json_vec::Column::StrVec).json())
        .to_owned();

    create_table(db, &create_table_stmt, JsonVec)
}

pub fn create_json_struct_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let stmt = sea_query::Table::create()
        .table(json_struct::Entity)
        .col(
            ColumnDef::new(json_struct::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(json_struct::Column::Json).json().not_null())
        .col(
            ColumnDef::new(json_struct::Column::JsonValue)
                .json()
                .not_null(),
        )
        .col(ColumnDef::new(json_struct::Column::JsonValueOpt).json())
        .col(ColumnDef::new(json_struct::Column::JsonNonSerializable).json())
        .to_owned();

    create_table(db, &stmt, JsonStruct)
}

pub fn create_json_string_vec_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let create_table_stmt = sea_query::Table::create()
        .table(JsonStringVec.table_ref())
        .col(
            ColumnDef::new(json_vec_derive::json_string_vec::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(json_vec_derive::json_string_vec::Column::StrVec).json())
        .to_owned();

    create_table(db, &create_table_stmt, JsonStringVec)
}

pub fn create_json_struct_vec_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let create_table_stmt = sea_query::Table::create()
        .table(JsonStructVec.table_ref())
        .col(
            ColumnDef::new(json_vec_derive::json_struct_vec::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(
            ColumnDef::new(json_vec_derive::json_struct_vec::Column::StructVec)
                .json_binary()
                .not_null(),
        )
        .to_owned();

    create_table(db, &create_table_stmt, JsonStructVec)
}

pub fn create_collection_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    db.execute_raw(sea_orm::Statement::from_string(
        db.get_database_backend(),
        "CREATE EXTENSION IF NOT EXISTS citext",
    ))?;

    let stmt = sea_query::Table::create()
        .table(collection::Entity)
        .col(
            ColumnDef::new(collection::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(
            ColumnDef::new(collection::Column::Name)
                .custom("citext")
                .not_null(),
        )
        .col(
            ColumnDef::new(collection::Column::Integers)
                .array(sea_query::ColumnType::Integer)
                .not_null(),
        )
        .col(ColumnDef::new(collection::Column::IntegersOpt).array(sea_query::ColumnType::Integer))
        .col(
            ColumnDef::new(collection::Column::Teas)
                .array(sea_query::ColumnType::Enum {
                    name: TeaEnum.into_iden(),
                    variants: vec![
                        TeaVariant::EverydayTea.into_iden(),
                        TeaVariant::BreakfastTea.into_iden(),
                        TeaVariant::AfternoonTea.into_iden(),
                    ],
                })
                .not_null(),
        )
        .col(
            ColumnDef::new(collection::Column::TeasOpt).array(sea_query::ColumnType::Enum {
                name: TeaEnum.into_iden(),
                variants: vec![
                    TeaVariant::EverydayTea.into_iden(),
                    TeaVariant::BreakfastTea.into_iden(),
                    TeaVariant::AfternoonTea.into_iden(),
                ],
            }),
        )
        .col(
            ColumnDef::new(collection::Column::Colors)
                .array(sea_query::ColumnType::Integer)
                .not_null(),
        )
        .col(ColumnDef::new(collection::Column::ColorsOpt).array(sea_query::ColumnType::Integer))
        .col(
            ColumnDef::new(collection::Column::Uuid)
                .array(sea_query::ColumnType::Uuid)
                .not_null(),
        )
        .col(
            ColumnDef::new(collection::Column::UuidHyphenated)
                .array(sea_query::ColumnType::Uuid)
                .not_null(),
        )
        .to_owned();

    create_table(db, &stmt, Collection)
}

pub fn create_host_network_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let stmt = sea_query::Table::create()
        .table(host_network::Entity)
        .col(
            ColumnDef::new(host_network::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(
            ColumnDef::new(host_network::Column::Hostname)
                .string()
                .not_null(),
        )
        .col(
            ColumnDef::new(host_network::Column::Ipaddress)
                .inet()
                .not_null(),
        )
        .col(
            ColumnDef::new(host_network::Column::Network)
                .cidr()
                .not_null(),
        )
        .to_owned();

    create_table(db, &stmt, HostNetwork)
}

pub fn create_pi_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let stmt = sea_query::Table::create()
        .table(pi::Entity)
        .col(
            ColumnDef::new(pi::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(
            ColumnDef::new(pi::Column::Decimal)
                .decimal_len(11, 10)
                .not_null(),
        )
        .col(
            ColumnDef::new(pi::Column::BigDecimal)
                .decimal_len(11, 10)
                .not_null(),
        )
        .col(ColumnDef::new(pi::Column::DecimalOpt).decimal_len(11, 10))
        .col(ColumnDef::new(pi::Column::BigDecimalOpt).decimal_len(11, 10))
        .to_owned();

    create_table(db, &stmt, Pi)
}

pub fn create_event_trigger_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let stmt = sea_query::Table::create()
        .table(event_trigger::Entity)
        .col(
            ColumnDef::new(event_trigger::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(
            ColumnDef::new(event_trigger::Column::Events)
                .array(sea_query::ColumnType::String(StringLen::None))
                .not_null(),
        )
        .to_owned();

    create_table(db, &stmt, EventTrigger)
}

pub fn create_uuid_fmt_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let stmt = sea_query::Table::create()
        .table(uuid_fmt::Entity)
        .col(
            ColumnDef::new(uuid_fmt::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(uuid_fmt::Column::Uuid).uuid().not_null())
        .col(
            ColumnDef::new(uuid_fmt::Column::UuidBraced)
                .uuid()
                .not_null(),
        )
        .col(
            ColumnDef::new(uuid_fmt::Column::UuidHyphenated)
                .uuid()
                .not_null(),
        )
        .col(
            ColumnDef::new(uuid_fmt::Column::UuidSimple)
                .uuid()
                .not_null(),
        )
        .col(ColumnDef::new(uuid_fmt::Column::UuidUrn).uuid().not_null())
        .to_owned();

    create_table(db, &stmt, UuidFmt)
}

pub fn create_edit_log_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let stmt = sea_query::Table::create()
        .table(edit_log::Entity)
        .col(
            ColumnDef::new(edit_log::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(edit_log::Column::Action).string().not_null())
        .col(ColumnDef::new(edit_log::Column::Values).json().not_null())
        .to_owned();

    create_table(db, &stmt, EditLog)
}

pub fn create_teas_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let create_table_stmt = sea_query::Table::create()
        .table(teas::Entity.table_ref())
        .col(
            ColumnDef::new(teas::Column::Id)
                .enumeration(
                    TeaEnum,
                    [
                        TeaVariant::EverydayTea,
                        TeaVariant::BreakfastTea,
                        TeaVariant::AfternoonTea,
                    ],
                )
                .not_null()
                .primary_key(),
        )
        .col(ColumnDef::new(teas::Column::Category).string_len(1))
        .col(ColumnDef::new(teas::Column::Color).integer())
        .to_owned();

    create_table(db, &create_table_stmt, Teas)
}

pub fn create_categories_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let create_table_stmt = sea_query::Table::create()
        .table(categories::Entity.table_ref())
        .col(
            ColumnDef::new(categories::Column::Id)
                .integer()
                .not_null()
                .primary_key(),
        )
        .col(
            ColumnDef::new(categories::Column::Categories)
                .array(ColumnType::String(StringLen::N(1))),
        )
        .to_owned();

    create_table(db, &create_table_stmt, Categories)
}

#[cfg(feature = "postgres-vector")]
pub fn create_embedding_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    db.execute(sea_orm::Statement::from_string(
        db.get_database_backend(),
        "CREATE EXTENSION IF NOT EXISTS vector",
    ))?;

    let create_table_stmt = sea_query::Table::create()
        .table(embedding::Entity.table_ref())
        .col(
            ColumnDef::new(embedding::Column::Id)
                .integer()
                .not_null()
                .primary_key(),
        )
        .col(
            ColumnDef::new(embedding::Column::Embedding)
                .vector(None)
                .not_null(),
        )
        .to_owned();

    create_table(db, &create_table_stmt, Embedding)
}

pub fn create_binary_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let create_table_stmt = sea_query::Table::create()
        .table(binary::Entity.table_ref())
        .col(
            ColumnDef::new(binary::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(binary::Column::Binary).binary().not_null())
        .col(
            ColumnDef::new(binary::Column::Binary10)
                .binary_len(10)
                .not_null(),
        )
        .col(
            ColumnDef::new(binary::Column::VarBinary16)
                .var_binary(16)
                .not_null(),
        )
        .to_owned();

    create_table(db, &create_table_stmt, Binary)
}

pub fn create_bits_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let create_table_stmt = sea_query::Table::create()
        .table(bits::Entity.table_ref())
        .col(
            ColumnDef::new(bits::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(bits::Column::Bit0).custom("BIT").not_null())
        .col(
            ColumnDef::new(bits::Column::Bit1)
                .custom("BIT(1)")
                .not_null(),
        )
        .col(
            ColumnDef::new(bits::Column::Bit8)
                .custom("BIT(8)")
                .not_null(),
        )
        .col(
            ColumnDef::new(bits::Column::Bit16)
                .custom("BIT(16)")
                .not_null(),
        )
        .col(
            ColumnDef::new(bits::Column::Bit32)
                .custom("BIT(32)")
                .not_null(),
        )
        .col(
            ColumnDef::new(bits::Column::Bit64)
                .custom("BIT(64)")
                .not_null(),
        )
        .to_owned();

    create_table(db, &create_table_stmt, Bits)
}

pub fn create_dyn_table_name_lazy_static_table(db: &DbConn) -> Result<(), DbErr> {
    use dyn_table_name::*;

    let entities = [Entity { table_name: 1 }, Entity { table_name: 2 }];
    for entity in entities {
        let create_table_stmt = sea_query::Table::create()
            .table(entity.table_ref())
            .col(
                ColumnDef::new(Column::Id)
                    .integer()
                    .not_null()
                    .auto_increment()
                    .primary_key(),
            )
            .col(ColumnDef::new(Column::Name).string().not_null())
            .to_owned();

        create_table(db, &create_table_stmt, entity)?;
    }

    Ok(())
}

pub fn create_value_type_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let general_stmt = sea_query::Table::create()
        .table(value_type::value_type_general::Entity)
        .col(
            ColumnDef::new(value_type::value_type_general::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(
            ColumnDef::new(value_type::value_type_general::Column::Number)
                .integer()
                .not_null(),
        )
        .col(
            ColumnDef::new(value_type::value_type_general::Column::Tag1)
                .string()
                .not_null(),
        )
        .col(
            ColumnDef::new(value_type::value_type_general::Column::Tag2)
                .text()
                .not_null(),
        )
        .to_owned();

    create_table(db, &general_stmt, value_type::value_type_general::Entity)?;
    create_table_from_entity(db, value_type::value_type_pk::Entity)
}

pub fn create_value_type_postgres_table(db: &DbConn) -> Result<ExecResult, DbErr> {
    let postgres_stmt = sea_query::Table::create()
        .table(value_type::value_type_pg::Entity)
        .col(
            ColumnDef::new(value_type::value_type_pg::Column::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(
            ColumnDef::new(value_type::value_type_pg::Column::Number)
                .integer()
                .not_null(),
        )
        .col(
            ColumnDef::new(value_type::value_type_pg::Column::StrVec)
                .array(sea_query::ColumnType::String(StringLen::None))
                .not_null(),
        )
        .to_owned();

    create_table(db, &postgres_stmt, value_type::value_type_pg::Entity)
}
