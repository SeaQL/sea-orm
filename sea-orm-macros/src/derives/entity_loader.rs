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
    let mut one_fields: Punctuated<_, Comma> = Punctuated::new();
    let mut with_impl = TokenStream::new();
    let mut select_impl = TokenStream::new();
    let mut assemble_one = TokenStream::new();
    let mut load_many = TokenStream::new();

    one_fields.push(quote!(mut model));

    for entity in schema.fields.iter() {
        let field = &entity.field;
        let is_one = entity.is_one;
        let entity: TokenStream = entity.entity.parse().unwrap();
        field_bools.push(quote!(#field: bool));

        with_impl.extend(quote! {
            if entity.table_ref() == #entity.table_ref() {
                self.with.#field = true;
            }
        });

        if is_one {
            one_fields.push(quote!(#field));

            select_impl.extend(quote! {
                let select = if self.with.#field {
                    select.find_also(Entity, #entity)
                } else {
                    select.select_also_fake(#entity)
                };
            });

            assemble_one.extend(quote! {
                model.#field.set(#field);
            });
        } else {
            load_many.extend(quote! {
                if self.with.#field {
                    let #field = models.load_many(#entity, db).await?;

                    for (model, #field) in models.iter_mut().zip(#field) {
                        model.#field.set(#field);
                    }
                }
            });
        }
    }

    quote! {

    pub struct EntityLoader {
        select: sea_orm::Select<Entity>,
        with: EntityLoaderWith,
    }

    #[derive(Debug, Default)]
    struct EntityLoaderWith {
        #field_bools
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
            #with_impl
            self
        }

        pub async fn all<C: sea_orm::ConnectionTrait>(self, db: &C) -> Result<Vec<Model>, sea_orm::DbErr> {
            let select = self.select;

            #select_impl

            let models = select.all(db).await?;

            let mut models = models.into_iter().map(|(#one_fields)| {
                #assemble_one
                model
            }).collect::<Vec<_>>();

            #load_many

            Ok(models)
        }
    }

    }
}
