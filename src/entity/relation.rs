use crate::{EntityTrait, Identity, IdentityOf, Iterable, QuerySelect, Select};
use core::marker::PhantomData;
use sea_query::{Alias, Condition, DynIden, JoinType, SeaRc, TableRef};
use std::fmt::Debug;

/// Defines the type of relationship
#[derive(Clone, Debug)]
pub enum RelationType {
    /// An Entity has one relationship
    HasOne,
    /// An Entity has many relationships
    HasMany,
}

/// Action to perform on a foreign key whenever there are changes
/// to an ActiveModel
pub type ForeignKeyAction = sea_query::ForeignKeyAction;

/// Constraints a type to implement the trait to create a relationship
pub trait RelationTrait: Iterable + Debug + 'static {
    /// The method to call
    fn def(&self) -> RelationDef;
}

/// Checks if Entities are related
pub trait Related<R>
where
    R: EntityTrait,
{
    /// Check if an entity is related to another entity
    fn to() -> RelationDef;

    /// Check if an entity is related through another entity
    fn via() -> Option<RelationDef> {
        None
    }

    /// Find related Entities
    fn find_related() -> Select<R> {
        Select::<R>::new().join_join_rev(JoinType::InnerJoin, Self::to(), Self::via())
    }
}

/// Defines a relationship
pub struct RelationDef {
    /// The type of relationship defined in [RelationType]
    pub rel_type: RelationType,
    /// Reference from another Entity
    pub from_tbl: TableRef,
    /// Reference to another ENtity
    pub to_tbl: TableRef,
    /// Reference to from a Column
    pub from_col: Identity,
    /// Reference to another column
    pub to_col: Identity,
    /// Defines the owner of the Relation
    pub is_owner: bool,
    /// Defines an operation to be performed on a Foreign Key when a
    /// `DELETE` Operation is performed
    pub on_delete: Option<ForeignKeyAction>,
    /// Defines an operation to be performed on a Foreign Key when a
    /// `UPDATE` Operation is performed
    pub on_update: Option<ForeignKeyAction>,
    /// Custom join ON condition
    pub on_condition: Option<Box<dyn Fn(DynIden, DynIden) -> Condition>>,
    /// The name of foreign key constraint
    pub fk_name: Option<String>,
}

impl std::fmt::Debug for RelationDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("RelationDef");
        d.field("rel_type", &self.rel_type)
            .field("from_tbl", &self.from_tbl)
            .field("to_tbl", &self.to_tbl)
            .field("from_col", &self.from_col)
            .field("to_col", &self.to_col)
            .field("is_owner", &self.is_owner)
            .field("on_delete", &self.on_delete)
            .field("on_update", &self.on_update);
        debug_on_condition(&mut d, &self.on_condition);
        d.field("fk_name", &self.fk_name).finish()
    }
}

fn debug_on_condition(
    d: &mut core::fmt::DebugStruct<'_, '_>,
    on_condition: &Option<Box<dyn Fn(DynIden, DynIden) -> Condition>>,
) {
    match on_condition {
        Some(func) => {
            d.field(
                "on_condition",
                &func(
                    SeaRc::new(Alias::new("left")),
                    SeaRc::new(Alias::new("right")),
                ),
            );
        }
        None => {
            d.field("on_condition", &Option::<Condition>::None);
        }
    }
}

/// Defines a helper to build a relation
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
    on_condition: Option<Box<dyn Fn(DynIden, DynIden) -> Condition>>,
    fk_name: Option<String>,
}

impl<E, R> std::fmt::Debug for RelationBuilder<E, R>
where
    E: EntityTrait,
    R: EntityTrait,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_struct("RelationBuilder");
        d.field("entities", &self.entities)
            .field("rel_type", &self.rel_type)
            .field("from_tbl", &self.from_tbl)
            .field("to_tbl", &self.to_tbl)
            .field("from_col", &self.from_col)
            .field("to_col", &self.to_col)
            .field("is_owner", &self.is_owner)
            .field("on_delete", &self.on_delete)
            .field("on_update", &self.on_update);
        debug_on_condition(&mut d, &self.on_condition);
        d.field("fk_name", &self.fk_name).finish()
    }
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
            on_condition: self.on_condition,
            fk_name: None,
        }
    }

    /// Set custom join ON condition.
    ///
    /// This method takes a closure with parameters
    /// denoting the left-hand side and right-hand side table in the join expression.
    ///
    /// # Examples
    ///
    /// assert_eq!(
    ///     cake::Entity::find()
    ///         .join(
    ///             JoinType::LeftJoin,
    ///             cake_filling::Relation::Cake
    ///                 .def()
    ///                 .rev()
    ///                 .on_condition(|_left, right| {
    ///                     Expr::tbl(right, cake_filling::Column::CakeId)
    ///                         .gt(10)
    ///                         .into_condition()
    ///                 })
    ///         )
    ///         .build(DbBackend::MySql)
    ///         .to_string(),
    ///     [
    ///         "SELECT `cake`.`id`, `cake`.`name` FROM `cake`",
    ///         "LEFT JOIN `cake_filling` ON `cake`.`id` = `cake_filling`.`cake_id` AND `cake_filling`.`cake_id` > 10",
    ///     ]
    ///     .join(" ")
    /// );
    pub fn on_condition<F>(mut self, f: F) -> Self
    where
        F: Fn(DynIden, DynIden) -> Condition + 'static,
    {
        self.on_condition = Some(Box::new(f));
        self
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
            on_condition: None,
            fk_name: None,
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
            on_condition: None,
            fk_name: None,
        }
    }

    /// Build a relationship from an Entity
    pub fn from<T>(mut self, identifier: T) -> Self
    where
        T: IdentityOf<E>,
    {
        self.from_col = Some(identifier.identity_of());
        self
    }

    /// Build a relationship to an Entity
    pub fn to<T>(mut self, identifier: T) -> Self
    where
        T: IdentityOf<R>,
    {
        self.to_col = Some(identifier.identity_of());
        self
    }

    /// An operation to perform on a foreign key when a delete operation occurs
    pub fn on_delete(mut self, action: ForeignKeyAction) -> Self {
        self.on_delete = Some(action);
        self
    }

    /// An operation to perform on a foreign key when an update operation occurs
    pub fn on_update(mut self, action: ForeignKeyAction) -> Self {
        self.on_update = Some(action);
        self
    }

    /// Set custom join ON condition.
    ///
    /// This method takes a closure with parameters
    /// denoting the left-hand side and right-hand side table in the join expression.
    pub fn on_condition<F>(mut self, f: F) -> Self
    where
        F: Fn(DynIden, DynIden) -> Condition + 'static,
    {
        self.on_condition = Some(Box::new(f));
        self
    }

    /// Set the name of foreign key constraint
    pub fn fk_name(mut self, fk_name: &str) -> Self {
        self.fk_name = Some(fk_name.to_owned());
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
            on_condition: b.on_condition,
            fk_name: b.fk_name,
        }
    }
}
