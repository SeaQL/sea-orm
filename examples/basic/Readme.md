# SeaORM SQLx example

Prepare:

Setup a test database and configure the connection string in `main.rs`.
Run `bakery.sql` to setup the test table and data.

Running: `cargo run`

```sh
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

find one by primary key: SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`id` = 1 LIMIT 1

Model { id: 1, name: "New York Cheese" }

find one by name: SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`name` LIKE '%chocolate%' LIMIT 1

Some(Model { id: 2, name: "Chocolate Forest" })

find models belong to: SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit` INNER JOIN `cake` ON `cake`.`id` = `fruit`.`cake_id` WHERE `cake`.`id` = 1

Model { id: 1, name: "Blueberry", cake_id: Some(1) }

Model { id: 2, name: "Rasberry", cake_id: Some(1) }

===== =====

find fruits and cakes: SELECT `fruit`.`id` AS `A_id`, `fruit`.`name` AS `A_name`, `fruit`.`cake_id` AS `A_cake_id`, `cake`.`id` AS `B_id`, `cake`.`name` AS `B_name` FROM `fruit` LEFT JOIN `cake` ON `fruit`.`cake_id` = `cake`.`id`
with loader: 
SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit`
SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`id` IN (1, 1, 2, NULL, NULL, NULL, NULL, NULL, NULL)

(Model { id: 1, name: "Blueberry", cake_id: Some(1) }, Some(Model { id: 1, name: "New York Cheese" }))
(Model { id: 2, name: "Rasberry", cake_id: Some(1) }, Some(Model { id: 1, name: "New York Cheese" }))
(Model { id: 3, name: "Strawberry", cake_id: Some(2) }, Some(Model { id: 2, name: "Chocolate Forest" }))
(Model { id: 4, name: "Apple", cake_id: None }, None)
(Model { id: 5, name: "Banana", cake_id: None }, None)
(Model { id: 6, name: "Cherry", cake_id: None }, None)
(Model { id: 7, name: "Lemon", cake_id: None }, None)
(Model { id: 8, name: "Orange", cake_id: None }, None)
(Model { id: 9, name: "Pineapple", cake_id: None }, None)
===== =====

find cakes with fruits: SELECT `cake`.`id` AS `A_id`, `cake`.`name` AS `A_name`, `fruit`.`id` AS `B_id`, `fruit`.`name` AS `B_name`, `fruit`.`cake_id` AS `B_cake_id` FROM `cake` LEFT JOIN `fruit` ON `cake`.`id` = `fruit`.`cake_id` ORDER BY `cake`.`id` ASC
with loader: 
SELECT `cake`.`id`, `cake`.`name` FROM `cake`
SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit` WHERE `fruit`.`cake_id` IN (1, 2)

(Model { id: 1, name: "New York Cheese" }, [Model { id: 1, name: "Blueberry", cake_id: Some(1) }, Model { id: 2, name: "Rasberry", cake_id: Some(1) }])

(Model { id: 2, name: "Chocolate Forest" }, [Model { id: 3, name: "Strawberry", cake_id: Some(2) }])

===== =====

count fruits by cake: SELECT `cake`.`name`, COUNT(`fruit`.`id`) AS `num_of_fruits` FROM `cake` LEFT JOIN `fruit` ON `cake`.`id` = `fruit`.`cake_id` GROUP BY `cake`.`name`

SelectResult { name: "New York Cheese", num_of_fruits: 2 }

SelectResult { name: "Chocolate Forest", num_of_fruits: 1 }

===== =====

find cakes and fillings: SELECT `cake`.`id` AS `A_id`, `cake`.`name` AS `A_name`, `filling`.`id` AS `B_id`, `filling`.`name` AS `B_name` FROM `cake` LEFT JOIN `cake_filling` ON `cake`.`id` = `cake_filling`.`cake_id` LEFT JOIN `filling` ON `cake_filling`.`filling_id` = `filling`.`id` ORDER BY `cake`.`id` ASC
with loader: 
SELECT `cake`.`id`, `cake`.`name` FROM `cake`
SELECT `cake_filling`.`cake_id`, `cake_filling`.`filling_id` FROM `cake_filling` WHERE `cake_filling`.`cake_id` IN (1, 2)
SELECT `filling`.`id`, `filling`.`name` FROM `filling` WHERE `filling`.`id` IN (1, 2, 2, 3)

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

===== =====

find all cakes paginated: 
SELECT `cake`.`id`, `cake`.`name` FROM `cake` LIMIT 3 OFFSET 0
Model { id: 1, name: "New York Cheese" }
Model { id: 2, name: "Chocolate Forest" }
SELECT `cake`.`id`, `cake`.`name` FROM `cake` LIMIT 3 OFFSET 3

find all fruits paginated: 
SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit` LIMIT 3 OFFSET 0
Model { id: 1, name: "Blueberry", cake_id: Some(1) }
Model { id: 2, name: "Rasberry", cake_id: Some(1) }
Model { id: 3, name: "Strawberry", cake_id: Some(2) }
SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit` LIMIT 3 OFFSET 3
Model { id: 4, name: "Apple", cake_id: None }
Model { id: 5, name: "Banana", cake_id: None }
Model { id: 6, name: "Cherry", cake_id: None }
SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit` LIMIT 3 OFFSET 6
Model { id: 7, name: "Lemon", cake_id: None }
Model { id: 8, name: "Orange", cake_id: None }
Model { id: 9, name: "Pineapple", cake_id: None }
SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit` LIMIT 3 OFFSET 9

find all fruits with stream: 
SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit` LIMIT 3 OFFSET 0
Model { id: 1, name: "Blueberry", cake_id: Some(1) }
Model { id: 2, name: "Rasberry", cake_id: Some(1) }
Model { id: 3, name: "Strawberry", cake_id: Some(2) }
SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit` LIMIT 3 OFFSET 3
Model { id: 4, name: "Apple", cake_id: None }
Model { id: 5, name: "Banana", cake_id: None }
Model { id: 6, name: "Cherry", cake_id: None }
SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit` LIMIT 3 OFFSET 6
Model { id: 7, name: "Lemon", cake_id: None }
Model { id: 8, name: "Orange", cake_id: None }
Model { id: 9, name: "Pineapple", cake_id: None }
SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit` LIMIT 3 OFFSET 9

find all fruits in json with stream: 
SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit` LIMIT 3 OFFSET 0
Object {"cake_id": Number(1), "id": Number(1), "name": String("Blueberry")}
Object {"cake_id": Number(1), "id": Number(2), "name": String("Rasberry")}
Object {"cake_id": Number(2), "id": Number(3), "name": String("Strawberry")}
SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit` LIMIT 3 OFFSET 3
Object {"cake_id": Null, "id": Number(4), "name": String("Apple")}
Object {"cake_id": Null, "id": Number(5), "name": String("Banana")}
Object {"cake_id": Null, "id": Number(6), "name": String("Cherry")}
SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit` LIMIT 3 OFFSET 6
Object {"cake_id": Null, "id": Number(7), "name": String("Lemon")}
Object {"cake_id": Null, "id": Number(8), "name": String("Orange")}
Object {"cake_id": Null, "id": Number(9), "name": String("Pineapple")}
SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit` LIMIT 3 OFFSET 9
===== =====

fruits first page: 
SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit` LIMIT 3 OFFSET 0
Model { id: 1, name: "Blueberry", cake_id: Some(1) }
Model { id: 2, name: "Rasberry", cake_id: Some(1) }
Model { id: 3, name: "Strawberry", cake_id: Some(2) }
===== =====

fruits number of page: 
SELECT COUNT(*) AS num_items FROM (SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit`) AS `sub_query`
3
===== =====

INSERT INTO `fruit` (`name`) VALUES ('pear')
Inserted: last_insert_id = 64
SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit` WHERE `fruit`.`id` = 64 LIMIT 1
Pear: Some(Model { id: 64, name: "pear", cake_id: None })
UPDATE `fruit` SET `name` = 'Sweet pear' WHERE `fruit`.`id` = 64
SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit` WHERE `fruit`.`id` = 64 LIMIT 1
Updated: Model { id: 64, name: "Sweet pear", cake_id: None }
DELETE FROM `fruit` WHERE `fruit`.`id` = 64
Deleted: DeleteResult { rows_affected: 1 }
===== =====

INSERT INTO `fruit` (`name`) VALUES ('Banana')
SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit` WHERE `fruit`.`id` = 65 LIMIT 1
Inserted: ActiveModel { id: Unchanged(65), name: Unchanged("Banana"), cake_id: Unchanged(None) }
UPDATE `fruit` SET `name` = 'Banana Mongo' WHERE `fruit`.`id` = 65
SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit` WHERE `fruit`.`id` = 65 LIMIT 1
Updated: ActiveModel { id: Unchanged(65), name: Unchanged("Banana Mongo"), cake_id: Unchanged(None) }
DELETE FROM `fruit` WHERE `fruit`.`id` = 65
Deleted: DeleteResult { rows_affected: 1 }
===== =====

INSERT INTO `fruit` (`name`) VALUES ('Pineapple')
SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit` WHERE `fruit`.`id` = 66 LIMIT 1
Saved: ActiveModel { id: Unchanged(66), name: Unchanged("Pineapple"), cake_id: Unchanged(None) }
DELETE FROM `fruit` WHERE `fruit`.`id` = 66
Deleted: DeleteResult { rows_affected: 1 }
```