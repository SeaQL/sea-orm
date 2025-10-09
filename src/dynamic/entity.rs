use super::{FieldType, ModelType};
use crate::{ColumnDef, ColumnTrait, DbBackend, EntityTrait, Iterable, ModelTrait, Value};
use sea_query::{
    ArrayType, BinOper, DynIden, Expr, ExprTrait, IntoColumnRef, IntoIden, IntoLikeExpr,
    IntoTableRef, SelectStatement, SimpleExpr, TableRef,
};
use std::sync::Arc;

#[derive(Debug)]
pub struct Entity {
    schema_name: Option<Arc<str>>,
    table_name: Arc<str>,
    columns: Vec<Column>,
}

#[derive(Debug)]
pub struct Column {
    table_name: Arc<str>,
    column_name: Arc<str>,
    column_def: ColumnDef,
    value_type: ArrayType,
    enum_type_name: Option<Arc<str>>,
}

impl Entity {
    pub fn schema_name(&self) -> Option<&str> {
        self.schema_name.as_deref()
    }

    pub fn table_name(&self) -> &str {
        &self.table_name
    }

    pub fn table_ref(&self) -> TableRef {
        match self.schema_name() {
            Some(schema) => (schema.to_owned(), self.table_name().to_owned()).into_table_ref(),
            None => self.table_name().to_owned().into_table_ref(),
        }
    }

    pub fn iter_columns(&self) -> impl Iterator<Item = &Column> {
        self.columns.iter()
    }
}

impl Entity {
    pub fn from_entity<E: EntityTrait>(entity: E) -> Self {
        Self {
            schema_name: entity.schema_name().map(Arc::from),
            table_name: Arc::from(entity.table_name()),
            columns: <E::Column as Iterable>::iter()
                .map(|c| {
                    let (tbl, col) = c.as_column_ref();
                    Column {
                        table_name: Arc::from(tbl.inner()),
                        column_name: Arc::from(col.inner()),
                        column_def: c.def(),
                        value_type: <E::Model as ModelTrait>::get_value_type(c),
                        enum_type_name: c.enum_type_name().map(Arc::from),
                    }
                })
                .collect(),
        }
    }

    pub fn to_model_type(&self) -> ModelType {
        ModelType {
            fields: self
                .columns
                .iter()
                .map(|c| FieldType {
                    field: c.column_name.clone(),
                    type_: c.value_type.clone(),
                })
                .collect(),
        }
    }
}

impl Column {
    pub fn def(&self) -> ColumnDef {
        self.column_def.clone()
    }

    pub fn column_name(&self) -> &str {
        &self.column_name
    }

    pub fn enum_type_name(&self) -> Option<&str> {
        self.enum_type_name.as_deref()
    }

    pub fn entity_name(&self) -> DynIden {
        self.table_name.to_string().into_iden()
    }

    pub fn as_column_ref(&self) -> (DynIden, DynIden) {
        (
            self.entity_name(),
            self.column_name().to_owned().into_iden(),
        )
    }

    crate::entity::column::methods::bind_oper!(pub eq, Equal);
    crate::entity::column::methods::bind_oper!(pub ne, NotEqual);
    crate::entity::column::methods::bind_oper!(pub gt, GreaterThan);
    crate::entity::column::methods::bind_oper!(pub gte, GreaterThanOrEqual);
    crate::entity::column::methods::bind_oper!(pub lt, SmallerThan);
    crate::entity::column::methods::bind_oper!(pub lte, SmallerThanOrEqual);

    pub fn between<V>(&self, a: V, b: V) -> SimpleExpr
    where
        V: Into<Value>,
    {
        Expr::col(self.as_column_ref()).between(a, b)
    }

    pub fn not_between<V>(&self, a: V, b: V) -> SimpleExpr
    where
        V: Into<Value>,
    {
        Expr::col(self.as_column_ref()).not_between(a, b)
    }

    pub fn like<T>(&self, s: T) -> SimpleExpr
    where
        T: IntoLikeExpr,
    {
        Expr::col(self.as_column_ref()).like(s)
    }

    pub fn not_like<T>(&self, s: T) -> SimpleExpr
    where
        T: IntoLikeExpr,
    {
        Expr::col(self.as_column_ref()).not_like(s)
    }

    pub fn starts_with<T>(&self, s: T) -> SimpleExpr
    where
        T: Into<String>,
    {
        let pattern = format!("{}%", s.into());
        Expr::col(self.as_column_ref()).like(pattern)
    }

    pub fn ends_with<T>(&self, s: T) -> SimpleExpr
    where
        T: Into<String>,
    {
        let pattern = format!("%{}", s.into());
        Expr::col(self.as_column_ref()).like(pattern)
    }

    pub fn contains<T>(&self, s: T) -> SimpleExpr
    where
        T: Into<String>,
    {
        let pattern = format!("%{}%", s.into());
        Expr::col(self.as_column_ref()).like(pattern)
    }

    crate::entity::column::methods::bind_func_no_params!(pub max);
    crate::entity::column::methods::bind_func_no_params!(pub min);
    crate::entity::column::methods::bind_func_no_params!(pub sum);
    crate::entity::column::methods::bind_func_no_params!(pub count);
    crate::entity::column::methods::bind_func_no_params!(pub is_null);
    crate::entity::column::methods::bind_func_no_params!(pub is_not_null);

    pub fn if_null<V>(&self, v: V) -> SimpleExpr
    where
        V: Into<Value>,
    {
        Expr::col(self.as_column_ref()).if_null(v)
    }

    crate::entity::column::methods::bind_vec_func!(pub is_in);
    crate::entity::column::methods::bind_vec_func!(pub is_not_in);

    crate::entity::column::methods::bind_subquery_func!(pub in_subquery);
    crate::entity::column::methods::bind_subquery_func!(pub not_in_subquery);

    pub fn into_expr(self) -> Expr {
        SimpleExpr::Column(self.as_column_ref().into_column_ref())
    }

    #[allow(clippy::match_single_binding)]
    pub fn into_returning_expr(self, db_backend: DbBackend) -> Expr {
        match db_backend {
            _ => Expr::col(self.column_name().to_owned()),
        }
    }

    pub fn select_as(&self, expr: Expr) -> SimpleExpr {
        crate::entity::column::cast_enum_as(
            expr,
            &self.def(),
            crate::entity::column::select_enum_as,
        )
    }

    pub fn save_as(&self, val: Expr) -> SimpleExpr {
        crate::entity::column::cast_enum_as(val, &self.def(), crate::entity::column::save_enum_as)
    }
}
