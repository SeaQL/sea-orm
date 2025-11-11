use sea_orm::sea_query::{self, Expr, ExprTrait, Query, SelectStatement, SimpleExpr};
use sea_orm::{
    ActiveModelTrait, Condition, ConnectionTrait, DbErr, DeriveIden, DynIden, EntityTrait,
};
#[allow(unused_imports)]
use sea_schema::probe::SchemaProbe;

pub fn query_tables<C>(db: &C) -> Result<SelectStatement, DbErr>
where
    C: ConnectionTrait,
{
    match db.get_database_backend() {
        #[cfg(feature = "sqlx-mysql")]
        DbBackend::MySql => Ok(sea_schema::mysql::MySql.query_tables()),
        #[cfg(feature = "sqlx-postgres")]
        DbBackend::Postgres => Ok(sea_schema::postgres::Postgres.query_tables()),
        #[cfg(feature = "sqlx-sqlite")]
        DbBackend::Sqlite => Ok(sea_schema::sqlite::Sqlite.query_tables()),
        #[allow(unreachable_patterns)]
        other => Err(DbErr::BackendNotSupported {
            db: other.as_str(),
            ctx: "query_tables",
        }),
    }
}

// this function is only called after checking db backend, the panic is unreachable
pub fn get_current_schema<C>(db: &C) -> SimpleExpr
where
    C: ConnectionTrait,
{
    match db.get_database_backend() {
        #[cfg(feature = "sqlx-mysql")]
        DbBackend::MySql => sea_schema::mysql::MySql::get_current_schema(),
        #[cfg(feature = "sqlx-postgres")]
        DbBackend::Postgres => sea_schema::postgres::Postgres::get_current_schema(),
        #[cfg(feature = "sqlx-sqlite")]
        DbBackend::Sqlite => sea_schema::sqlite::Sqlite::get_current_schema(),
        #[allow(unreachable_patterns)]
        other => panic!("{other:?} feature is off"),
    }
}

#[derive(DeriveIden)]
enum InformationSchema {
    #[sea_orm(iden = "information_schema")]
    Schema,
    #[sea_orm(iden = "TABLE_NAME")]
    TableName,
    #[sea_orm(iden = "CONSTRAINT_NAME")]
    ConstraintName,
    TableConstraints,
    TableSchema,
    ConstraintType,
}

pub fn query_mysql_foreign_keys<C>(db: &C) -> SelectStatement
where
    C: ConnectionTrait,
{
    let mut stmt = Query::select();
    stmt.columns([
        InformationSchema::TableName,
        InformationSchema::ConstraintName,
    ])
    .from((
        InformationSchema::Schema,
        InformationSchema::TableConstraints,
    ))
    .cond_where(
        Condition::all()
            .add(get_current_schema(db).equals((
                InformationSchema::TableConstraints,
                InformationSchema::TableSchema,
            )))
            .add(
                Expr::col((
                    InformationSchema::TableConstraints,
                    InformationSchema::ConstraintType,
                ))
                .eq("FOREIGN KEY"),
            ),
    );
    stmt
}

#[derive(DeriveIden)]
enum PgType {
    Table,
    Oid,
    Typname,
    Typnamespace,
    Typelem,
}

#[derive(DeriveIden)]
enum PgDepend {
    Table,
    Objid,
    Deptype,
    Refclassid,
}

#[derive(DeriveIden)]
enum PgNamespace {
    Table,
    Oid,
    Nspname,
}

pub fn query_pg_types<C>(db: &C) -> SelectStatement
where
    C: ConnectionTrait,
{
    Query::select()
        .column(PgType::Typname)
        .from(PgType::Table)
        .left_join(
            PgNamespace::Table,
            Expr::col((PgNamespace::Table, PgNamespace::Oid))
                .equals((PgType::Table, PgType::Typnamespace)),
        )
        .left_join(
            PgDepend::Table,
            Expr::col((PgDepend::Table, PgDepend::Objid))
                .equals((PgType::Table, PgType::Oid))
                .and(
                    Expr::col((PgDepend::Table, PgDepend::Refclassid))
                        .eq(Expr::cust("'pg_extension'::regclass::oid")),
                )
                .and(Expr::col((PgDepend::Table, PgDepend::Deptype)).eq(Expr::cust("'e'"))),
        )
        .and_where(get_current_schema(db).equals((PgNamespace::Table, PgNamespace::Nspname)))
        .and_where(Expr::col((PgType::Table, PgType::Typelem)).eq(0))
        .and_where(Expr::col((PgDepend::Table, PgDepend::Objid)).is_null())
        .take()
}

pub trait QueryTable {
    type Statement;

    fn table_name(self, table_name: DynIden) -> Self::Statement;
}

impl QueryTable for SelectStatement {
    type Statement = SelectStatement;

    fn table_name(mut self, table_name: DynIden) -> SelectStatement {
        self.from(table_name);
        self
    }
}

impl QueryTable for sea_query::TableCreateStatement {
    type Statement = sea_query::TableCreateStatement;

    fn table_name(mut self, table_name: DynIden) -> sea_query::TableCreateStatement {
        self.table(table_name);
        self
    }
}

impl<A> QueryTable for sea_orm::Insert<A>
where
    A: ActiveModelTrait,
{
    type Statement = sea_orm::Insert<A>;

    fn table_name(mut self, table_name: DynIden) -> sea_orm::Insert<A> {
        sea_orm::QueryTrait::query(&mut self).into_table(table_name);
        self
    }
}

impl<E> QueryTable for sea_orm::DeleteMany<E>
where
    E: EntityTrait,
{
    type Statement = sea_orm::DeleteMany<E>;

    fn table_name(mut self, table_name: DynIden) -> sea_orm::DeleteMany<E> {
        sea_orm::QueryTrait::query(&mut self).from_table(table_name);
        self
    }
}
