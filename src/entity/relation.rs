use crate::{
    ColumnTrait, EntityTrait, Identity, IdentityOf, Iterable, ModelTrait, QuerySelect, Select,
};
use core::marker::PhantomData;
use sea_query::{JoinType, TableRef};
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub enum RelationType {
    HasOne,
    HasMany,
}

pub type ForeignKeyAction = sea_query::ForeignKeyAction;

pub trait RelationTrait: Iterable + Debug + 'static {
    fn def(&self) -> RelationDef;
}

pub trait Related<R>
where
    R: EntityTrait,
{
    fn to() -> RelationDef;

    fn via() -> Option<RelationDef> {
        None
    }

    fn find_related() -> Select<R>
    where
        Self: EntityTrait,
    {
        let mut select =
            Select::<R>::new().join_join_rev(JoinType::InnerJoin, Self::to(), Self::via());
        match <<Self as EntityTrait>::Model as ModelTrait>::soft_delete_column() {
            Some(soft_delete_column) if !select.with_deleted => {
                select.query().and_where(soft_delete_column.is_null());
            }
            _ => {}
        }
        select
    }
}

#[derive(Debug)]
pub struct RelationDef {
    pub rel_type: RelationType,
    pub from_tbl: TableRef,
    pub to_tbl: TableRef,
    pub from_col: Identity,
    pub to_col: Identity,
    pub is_owner: bool,
    pub on_delete: Option<ForeignKeyAction>,
    pub on_update: Option<ForeignKeyAction>,
}

#[derive(Debug)]
pub struct RelationBuilder<E, R>
where
    E: EntityTrait,
    R: EntityTrait,
{
    entities: PhantomData<(E, R)>,
    rel_type: RelationType,
    from_tbl: TableRef,
    to_tbl: TableRef,
    from_col: Option<Identity>,
    to_col: Option<Identity>,
    is_owner: bool,
    on_delete: Option<ForeignKeyAction>,
    on_update: Option<ForeignKeyAction>,
}

impl RelationDef {
    /// Reverse this relation (swap from and to)
    pub fn rev(self) -> Self {
        Self {
            rel_type: self.rel_type,
            from_tbl: self.to_tbl,
            to_tbl: self.from_tbl,
            from_col: self.to_col,
            to_col: self.from_col,
            is_owner: !self.is_owner,
            on_delete: self.on_delete,
            on_update: self.on_update,
        }
    }
}

impl<E, R> RelationBuilder<E, R>
where
    E: EntityTrait,
    R: EntityTrait,
{
    pub(crate) fn new(rel_type: RelationType, from: E, to: R, is_owner: bool) -> Self {
        Self {
            entities: PhantomData,
            rel_type,
            from_tbl: from.table_ref(),
            to_tbl: to.table_ref(),
            from_col: None,
            to_col: None,
            is_owner,
            on_delete: None,
            on_update: None,
        }
    }

    pub(crate) fn from_rel(rel_type: RelationType, rel: RelationDef, is_owner: bool) -> Self {
        Self {
            entities: PhantomData,
            rel_type,
            from_tbl: rel.from_tbl,
            to_tbl: rel.to_tbl,
            from_col: Some(rel.from_col),
            to_col: Some(rel.to_col),
            is_owner,
            on_delete: None,
            on_update: None,
        }
    }

    pub fn from<T>(mut self, identifier: T) -> Self
    where
        T: IdentityOf<E>,
    {
        self.from_col = Some(identifier.identity_of());
        self
    }

    pub fn to<T>(mut self, identifier: T) -> Self
    where
        T: IdentityOf<R>,
    {
        self.to_col = Some(identifier.identity_of());
        self
    }

    pub fn on_delete(mut self, action: ForeignKeyAction) -> Self {
        self.on_delete = Some(action);
        self
    }

    pub fn on_update(mut self, action: ForeignKeyAction) -> Self {
        self.on_update = Some(action);
        self
    }
}

impl<E, R> From<RelationBuilder<E, R>> for RelationDef
where
    E: EntityTrait,
    R: EntityTrait,
{
    fn from(b: RelationBuilder<E, R>) -> Self {
        RelationDef {
            rel_type: b.rel_type,
            from_tbl: b.from_tbl,
            to_tbl: b.to_tbl,
            from_col: b.from_col.unwrap(),
            to_col: b.to_col.unwrap(),
            is_owner: b.is_owner,
            on_delete: b.on_delete,
            on_update: b.on_update,
        }
    }
}
