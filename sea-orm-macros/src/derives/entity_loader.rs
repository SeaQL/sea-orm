use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, punctuated::Punctuated, token::Comma};

#[derive(Default)]
pub struct EntityLoaderSchema {
    pub fields: Vec<EntityLoaderField>,
}

pub struct EntityLoaderField {
    pub is_one: bool,
    pub field: Ident,
    /// super::bakery::Entity
    pub entity: String,
}

impl EntityLoaderSchema {
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

pub fn expand_entity_loader(schema: EntityLoaderSchema) -> TokenStream {
    let mut field_bools: Punctuated<_, Comma> = Punctuated::new();
    let mut field_nests: Punctuated<_, Comma> = Punctuated::new();
    let mut one_fields: Punctuated<_, Comma> = Punctuated::new();
    let mut with_impl = TokenStream::new();
    let mut with_nest_impl = TokenStream::new();
    let mut select_impl = TokenStream::new();
    let mut assemble_one = TokenStream::new();
    let mut load_one = TokenStream::new();
    let mut load_many = TokenStream::new();
    let mut load_one_nest = TokenStream::new();
    let mut load_many_nest = TokenStream::new();
    let mut load_one_nest_nest = TokenStream::new();
    let mut load_many_nest_nest = TokenStream::new();
    let mut arity = 1;

    one_fields.push(quote!(mut model));

    for entity_field in schema.fields.iter() {
        let field = &entity_field.field;
        let is_one = entity_field.is_one;
        let entity: TokenStream = entity_field.entity.parse().unwrap();
        let entity_module: TokenStream = entity_field
            .entity
            .trim_end_matches("::Entity")
            .parse()
            .unwrap();

        field_bools.push(quote!(pub #field: bool));
        field_nests.push(quote!(pub #field: #entity_module::EntityLoaderWith));

        with_impl.extend(quote! {
            if target == #entity.table_ref() {
                self.#field = true;
            }
        });
        with_nest_impl.extend(quote! {
            if entity.table_ref() == #entity.table_ref() {
                self.with.#field = true;
                self.nest.#field.set(nested.table_ref());
            }
        });

        if is_one {
            arity += 1;
            if arity <= 3 {
                // do not go beyond SelectThree
                one_fields.push(quote!(#field));

                select_impl.extend(quote! {
                    let select = if self.with.#field && self.nest.#field.is_empty() {
                        self.with.#field = false;
                        select.find_also(Entity, #entity)
                    } else {
                        select.select_also_fake(#entity)
                    };
                });

                assemble_one.extend(quote! {
                    model.#field.set(#field);
                });
            }

            load_one.extend(quote! {
                if with.#field {
                    let #field = models.load_one(#entity, db).await?;
                    let #field = #entity_module::EntityLoader::load_nest(#field, &nest.#field, db).await?;

                    for (model, #field) in models.iter_mut().zip(#field) {
                        model.#field.set(#field);
                    }
                }
            });
            load_one_nest.extend(quote! {
                if with.#field {
                    let #field = models.as_slice().load_one(#entity, db).await?;

                    for (model, #field) in models.iter_mut().zip(#field) {
                        if let Some(model) = model.as_mut() {
                            model.#field.set(#field);
                        }
                    }
                }
            });
            load_one_nest_nest.extend(quote! {
                if with.#field {
                    let #field = models.as_slice().load_one(#entity, db).await?;

                    for (models, #field) in models.iter_mut().zip(#field) {
                        for (model, #field) in models.iter_mut().zip(#field) {
                            model.#field.set(#field);
                        }
                    }
                }
            });
        } else {
            load_many.extend(quote! {
                if with.#field {
                    let #field = models.load_many(#entity, db).await?;
                    let #field = #entity_module::EntityLoader::load_nest_nest(#field, &nest.#field, db).await?;

                    for (model, #field) in models.iter_mut().zip(#field) {
                        model.#field.set(#field);
                    }
                }
            });
            load_many_nest.extend(quote! {
                if with.#field {
                    let #field = models.as_slice().load_many(#entity, db).await?;

                    for (model, #field) in models.iter_mut().zip(#field) {
                        if let Some(model) = model.as_mut() {
                            model.#field.set(#field);
                        }
                    }
                }
            });
            load_many_nest_nest.extend(quote! {
                if with.#field {
                    let #field = models.as_slice().load_many(#entity, db).await?;

                    for (models, #field) in models.iter_mut().zip(#field) {
                        for (model, #field) in models.iter_mut().zip(#field) {
                            model.#field.set(#field);
                        }
                    }
                }
            });
        }
    }

    quote! {

    pub struct EntityLoader {
        select: sea_orm::Select<Entity>,
        with: EntityLoaderWith,
        nest: EntityLoaderNest,
    }

    #[derive(Debug, Default, PartialEq, Eq)]
    pub struct EntityLoaderWith {
        #field_bools
    }

    #[derive(Debug, Default, PartialEq, Eq)]
    pub struct EntityLoaderNest {
        #field_nests
    }

    impl EntityLoaderWith {
        pub fn is_empty(&self) -> bool {
            self == &Self::default()
        }
        pub fn set(&mut self, target: sea_orm::sea_query::TableRef) {
            #with_impl
        }
    }

    impl sea_orm::QueryFilter for EntityLoader {
        type QueryStatement = <sea_orm::Select<Entity> as sea_orm::QueryFilter>::QueryStatement;

        fn query(&mut self) -> &mut Self::QueryStatement {
            sea_orm::QueryFilter::query(&mut self.select)
        }
    }

    impl sea_orm::QueryOrder for EntityLoader {
        type QueryStatement = <sea_orm::Select<Entity> as sea_orm::QueryOrder>::QueryStatement;

        fn query(&mut self) -> &mut Self::QueryStatement {
            sea_orm::QueryOrder::query(&mut self.select)
        }
    }

    impl sea_orm::compound::EntityLoaderTrait<Entity> for EntityLoader {}

    impl Entity {
        pub fn load() -> EntityLoader {
            EntityLoader {
                select: Entity::find(),
                with: Default::default(),
                nest: Default::default(),
            }
        }
    }

    impl EntityLoader {
        pub async fn one<C: sea_orm::ConnectionTrait>(
            mut self,
            db: &C,
        ) -> Result<Option<Model>, sea_orm::DbErr> {
            use sea_orm::QuerySelect;

            self.select = self.select.limit(1);
            Ok(self.all(db).await?.into_iter().next())
        }

        pub fn with<R>(mut self, entity: R) -> Self
        where
            R: EntityTrait,
            Entity: Related<R>,
        {
            self.with.set(entity.table_ref());
            self
        }

        pub fn nest<R, S>(mut self, entity: R, nested: S) -> Self
        where
            R: EntityTrait,
            Entity: Related<R>,
            S: EntityTrait,
            R: Related<S>,
        {
            #with_nest_impl
            self
        }

        pub async fn all<C: sea_orm::ConnectionTrait>(mut self, db: &C) -> Result<Vec<Model>, sea_orm::DbErr> {
            let select = self.select;

            #select_impl

            let models = select.all(db).await?;

            let models = models.into_iter().map(|(#one_fields)| {
                #assemble_one
                model
            }).collect::<Vec<_>>();

            let models = Self::load(models, &self.with, &self.nest, db).await?;

            Ok(models)
        }

        pub async fn load<C: sea_orm::ConnectionTrait>(mut models: Vec<Model>, with: &EntityLoaderWith, nest: &EntityLoaderNest, db: &C) -> Result<Vec<Model>, DbErr> {
            #load_one
            #load_many
            Ok(models)
        }

        pub async fn load_nest<C: sea_orm::ConnectionTrait>(mut models: Vec<Option<Model>>, with: &EntityLoaderWith, db: &C) -> Result<Vec<Option<Model>>, DbErr> {
            #load_one_nest
            #load_many_nest
            Ok(models)
        }

        pub async fn load_nest_nest<C: sea_orm::ConnectionTrait>(mut models: Vec<Vec<Model>>, with: &EntityLoaderWith, db: &C) -> Result<Vec<Vec<Model>>, DbErr> {
            use sea_orm::NestedLoaderTrait;
            #load_one_nest_nest
            #load_many_nest_nest
            Ok(models)
        }
    }

    }
}
