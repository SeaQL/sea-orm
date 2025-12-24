order::ActiveModelEx {
    id: NotSet, // auto-increment
    total: Set(22.into()),
    bakery_id: NotSet, // to be auto-set
    customer_id: NotSet, // to be auto-set
    placed_at: Utc::now(),
    bakery: Set(bakery), // will auto-set bakery_id above
    customer: Some(customer::ActiveModelEx {
        id: NotSet, // id is not set, we will create a new customer
        name: Set("Jack".into()),
        notes: Set(None),
    }),
    shippment: Some(shippment::ActiveModelEx {
        id: NotSet,
        order_id: NotSet, // auto-set
        address: Set("street".into()),
    }),
    lineitems: ActiveModelOperation::Append(vec![
        lineitem::ActiveModelEx {
            id: NotSet, // auto-increment
            price: Set(2.into()),
            order_id: NotSet, // will be set after order is inserted
            cake_id: NotSet // to be auto-set
            cake: Set(cake::ActiveModelEx {
                id: Unchanged(12), // this cake already exists, so will not be created
                name: Unchanged("Chocolate Cake".into())
            }),
        }
    ])
}

// belongs to
if self.bakery_id.is_set() {
    // good
} else if self.bakery.is_set() {
    if self.bakery.id.is_set() {
        self.bakery_id = self.bakery.id;
    } else {
        self.bakery_id = self.bakery.save()?;
    }
}

self.save();

// has one
if let Some(shippment) = self.shippment {
    shippment.order_id = Set(self.id)
    shippment.save();
}

// has many
if self.lineitems.is_replace() {
    // delete all items that's not present in current set
    let redundant = self.load_many(id.not_in(self.lineitems.extract_ids()));
    for item in redundant {
        item.delete();
    }
}
for lineitem in self.lineitems {
    lineitem.set_parent_key::<Self>(self.id);
    lineitem.save();
}

fn lineitem.save() {
    if self.cake_id.is_set() {
        // good
    } else if self.cake.is_set() {
        if self.cake.id.is_set() {
            self.cake_id = self.cake.id;
        } else {
            self.cake_id = self.cake.save()?;
        }
    } else if self.cake_id.is_optional() {
        // okay
    } else if is_update {
        // probably okay
    } else {
        bail
    }
}

// has many via

for tag in self.tags {
    self.tags.push(tag.save());
    let via = via::ActiveModel::default();
    via.set_parent_key(self);
    via.set_parent_key(tag);
    via::Entity::insert(via).on_conflict_do_nothing().exec();
}
