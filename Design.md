# Readability

## Turbofish and inference

Consider the following method:
```rust
fn left_join<E>(self) -> Self
where
    E: EntityTrait,
{
    // ...
}
```
which has to be invoked like:
```rust
.left_join::<fruit::Entity>()
```

If we instead do:
```rust
fn left_join<E>(self, _: E) -> Self
where
    E: EntityTrait,
{
    // ...
}
```
then the Turbofish can be omitted:
```rust
.left_join(fruit::Entity)
```
provided that `fruit::Entity` is a unit struct.

## Builder pattern

Instead of:
```rust
fn has_many(entity: Entity, from: Column, to: Column);

has_many(cake::Entity, cake::Column::Id, fruit::Column::CakeId)
```

we'd prefer having a builder and stating the params explicitly:
```rust
has_many(cake::Entity).from(cake::Column::Id).to(fruit::Column::CakeId)
```