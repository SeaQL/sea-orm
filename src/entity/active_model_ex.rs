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

0. erase types into dynamic active model?
1. flatten tree into insert / update / delete-not-in and edges
2. extract unset foreign keys, look for items in edges, if item not exists, bail
3. topological sort and reorder transaction list
4. extract top level primary key, (if returning) use entity loader to get back result

// belongs to
if self.bakery_id.is_set() {
    // good
} else if self.bakery.is_set() {
    if self.bakery.id.is_set() {
        self.bakery_id = self.bakery.id;
    } else {
        self.bakery_id = self.bakery.save()?;
    }
} else if self.bakery_id.is_optional() {
    // okay
} else {
    bail
}

self.save();

// has many
if self.lineitems.is_replace() {
    self.lineitems.delete_by_parent_id(self.id, self.lineitems.extract_ids());
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