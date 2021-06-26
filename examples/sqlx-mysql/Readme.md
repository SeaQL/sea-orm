# SeaORM SQLx MySql example

Prepare:

Setup a test database and configure the connection string in `main.rs`.
Run `bakery.sql` to setup the test table and data.

Running:

```sh
cargo run
```

All about selects:

```sh
Database { connection: SqlxMySqlPoolConnection }

===== =====

find all cakes: SELECT `cake`.`id`, `cake`.`name` FROM `cake`

Model { id: 1, name: "New York Cheese" }

Model { id: 2, name: "Chocolate Forest" }

find all fruits: SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit`

Model { id: 1, name: "Blueberry", cake_id: Some(1) }

Model { id: 2, name: "Rasberry", cake_id: Some(1) }

Model { id: 3, name: "Strawberry", cake_id: Some(2) }

Model { id: 4, name: "Apple", cake_id: None }

Model { id: 5, name: "Banana", cake_id: None }

Model { id: 6, name: "Cherry", cake_id: None }

Model { id: 7, name: "Lemon", cake_id: None }

Model { id: 8, name: "Orange", cake_id: None }

Model { id: 9, name: "Pineapple", cake_id: None }

===== =====

find cakes and fruits: SELECT `cake`.`id` AS `A_id`, `cake`.`name` AS `A_name`, `fruit`.`id` AS `B_id`, `fruit`.`name` AS `B_name`, `fruit`.`cake_id` AS `B_cake_id` FROM `cake` LEFT JOIN `fruit` ON `cake`.`id` = `fruit`.`cake_id`

(Model { id: 1, name: "New York Cheese" }, Some(Model { id: 1, name: "Blueberry", cake_id: Some(1) }))

(Model { id: 1, name: "New York Cheese" }, Some(Model { id: 2, name: "Rasberry", cake_id: Some(1) }))

(Model { id: 2, name: "Chocolate Forest" }, Some(Model { id: 3, name: "Strawberry", cake_id: Some(2) }))

===== =====

find one by primary key: SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`id` = 1 LIMIT 1

Model { id: 1, name: "New York Cheese" }

find one by name: SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`name` LIKE '%chocolate%' LIMIT 1

Some(Model { id: 2, name: "Chocolate Forest" })

find models belong to: SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit` INNER JOIN `cake` ON `cake`.`id` = `fruit`.`cake_id` WHERE `cake`.`id` = 1

Model { id: 1, name: "Blueberry", cake_id: Some(1) }

Model { id: 2, name: "Rasberry", cake_id: Some(1) }

===== =====

count fruits by cake: SELECT `cake`.`name`, COUNT(`fruit`.`id`) AS `num_of_fruits` FROM `cake` LEFT JOIN `fruit` ON `cake`.`id` = `fruit`.`cake_id` GROUP BY `cake`.`name`

SelectResult { name: "New York Cheese", num_of_fruits: 2 }

SelectResult { name: "Chocolate Forest", num_of_fruits: 1 }

===== =====

find cakes and fillings: SELECT `cake`.`id` AS `A_id`, `cake`.`name` AS `A_name`, `filling`.`id` AS `B_id`, `filling`.`name` AS `B_name` FROM `cake` LEFT JOIN `cake_filling` ON `cake`.`id` = `cake_filling`.`cake_id` LEFT JOIN `filling` ON `cake_filling`.`filling_id` = `filling`.`id` ORDER BY `cake`.`id` ASC

(Model { id: 1, name: "New York Cheese" }, [Model { id: 1, name: "Vanilla" }, Model { id: 2, name: "Lemon" }])

(Model { id: 2, name: "Chocolate Forest" }, [Model { id: 2, name: "Lemon" }, Model { id: 3, name: "Mango" }])

find fillings for cheese cake: SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`id` = 1 LIMIT 1
SELECT `filling`.`id`, `filling`.`name` FROM `filling` INNER JOIN `cake_filling` ON `cake_filling`.`filling_id` = `filling`.`id` INNER JOIN `cake` ON `cake`.`id` = `cake_filling`.`cake_id` WHERE `cake`.`id` = 1

Model { id: 1, name: "Vanilla" }

Model { id: 2, name: "Lemon" }

find cakes for lemon: SELECT `filling`.`id`, `filling`.`name` FROM `filling` WHERE `filling`.`id` = 2 LIMIT 1
SELECT `cake`.`id`, `cake`.`name` FROM `cake` INNER JOIN `cake_filling` ON `cake_filling`.`cake_id` = `cake`.`id` INNER JOIN `filling` ON `filling`.`id` = `cake_filling`.`filling_id` WHERE `filling`.`id` = 2

Model { id: 1, name: "New York Cheese" }

Model { id: 2, name: "Chocolate Forest" }

```

All about operations:

```sh
INSERT INTO `fruit` (`name`) VALUES ('pear')

Inserted: InsertResult { last_insert_id: 21 }

SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit` WHERE `fruit`.`id` = 21 LIMIT 1

Pear: Some(Model { id: 21, name: "pear", cake_id: None })

UPDATE `fruit` SET `name` = 'Sweet pear' WHERE `fruit`.`id` = 21

Updated: ActiveModel { id: ActiveValue { value: 21, state: Unchanged }, name: ActiveValue { value: "Sweet pear", state: Set }, cake_id: ActiveValue { value: None, state: Unchanged } }

===== =====

INSERT INTO `fruit` (`name`) VALUES ('banana')
SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit` WHERE `fruit`.`id` = 22 LIMIT 1

Inserted: ActiveModel { id: ActiveValue { value: 22, state: Unchanged }, name: ActiveValue { value: "banana", state: Unchanged }, cake_id: ActiveValue { value: None, state: Unchanged } }

UPDATE `fruit` SET `name` = 'banana banana' WHERE `fruit`.`id` = 22

Updated: ActiveModel { id: ActiveValue { value: 22, state: Unchanged }, name: ActiveValue { value: "banana banana", state: Set }, cake_id: ActiveValue { value: None, state: Unchanged } }

DELETE FROM `fruit` WHERE `fruit`.`id` = 22

Deleted: DeleteResult { rows_affected: 1 }

===== =====

INSERT INTO `fruit` (`name`) VALUES ('pineapple')
SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit` WHERE `fruit`.`id` = 23 LIMIT 1

Saved: ActiveModel { id: ActiveValue { value: 23, state: Unchanged }, name: ActiveValue { value: "pineapple", state: Unchanged } }
```