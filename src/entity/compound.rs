#![allow(missing_docs)]
use super::EntityTrait;
use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct BelongsTo<E: EntityTrait> {
    phantom: PhantomData<E>,
    pub(crate) item: Option<Box<E::Model>>,
}

// TODO impl serde

impl<E: EntityTrait> Default for BelongsTo<E> {
    fn default() -> Self {
        Self {
            phantom: PhantomData,
            item: None,
        }
    }
}

impl<E: EntityTrait> PartialEq for BelongsTo<E> {
    fn eq(&self, _: &Self) -> bool {
        // same type regard as true
        true
    }
}

impl<E: EntityTrait> Eq for BelongsTo<E> {}

impl<E: EntityTrait> BelongsTo<E> {
    pub fn get(&self) -> Option<&E::Model> {
        self.item.as_deref()
    }

    pub fn set(&mut self, item: Option<Box<E::Model>>) {
        self.item = item
    }
}

#[derive(Debug, Clone)]
pub struct HasMany<E: EntityTrait> {
    phantom: PhantomData<E>,
    pub(crate) items: Vec<E::Model>,
}

impl<E: EntityTrait> Default for HasMany<E> {
    fn default() -> Self {
        Self {
            phantom: PhantomData,
            items: Default::default(),
        }
    }
}

impl<E: EntityTrait> PartialEq for HasMany<E> {
    fn eq(&self, _: &Self) -> bool {
        // same type regard as true
        true
    }
}

impl<E: EntityTrait> Eq for HasMany<E> {}

impl<E: EntityTrait> HasMany<E> {
    pub fn get(&self) -> &[E::Model] {
        &self.items
    }

    pub fn set(&mut self, items: Vec<E::Model>) {
        self.items = items
    }
}
