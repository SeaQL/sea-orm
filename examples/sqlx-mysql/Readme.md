# SeaORM SQLx MySql example

Prepare:

Setup a test database and configure the connection string in `main.rs`.
Run `bakery.sql` to setup the test table and data.

Running:

```sh
cargo run
```

Example output:

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

===== =====

find cakes and fruits: SELECT `cake`.`id` AS `A_id`, `cake`.`name` AS `A_name`, `fruit`.`id` AS `B_id`, `fruit`.`name` AS `B_name`, `fruit`.`cake_id` AS `B_cake_id` FROM `cake` LEFT JOIN `fruit` ON `cake`.`id` = `fruit`.`cake_id`

(Model { id: 1, name: "New York Cheese" }, Model { id: 2, name: "Rasberry", cake_id: Some(1) })

(Model { id: 1, name: "New York Cheese" }, Model { id: 1, name: "Blueberry", cake_id: Some(1) })

(Model { id: 2, name: "Chocolate Forest" }, Model { id: 3, name: "Strawberry", cake_id: Some(2) })

===== =====

find one by primary key: SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`id` = 1 LIMIT 1

Model { id: 1, name: "New York Cheese" }

find one by like: SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`name` LIKE '%chocolate%' LIMIT 1

Model { id: 2, name: "Chocolate Forest" }

find models belong to: SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit` INNER JOIN `cake` ON `cake`.`id` = `fruit`.`cake_id` WHERE `cake`.`id` = 1

Model { id: 1, name: "Blueberry", cake_id: Some(1) }

Model { id: 2, name: "Rasberry", cake_id: Some(1) }

===== =====

count fruits by cake: SELECT `cake`.`name`, COUNT(`fruit`.`id`) AS `num_of_fruits` FROM `cake` LEFT JOIN `fruit` ON `cake`.`id` = `fruit`.`cake_id` GROUP BY `cake`.`name`

SelectResult { name: "New York Cheese", num_of_fruits: 2 }

SelectResult { name: "Chocolate Forest", num_of_fruits: 1 }

===== =====

find cakes and fillings: SELECT `cake`.`id` AS `A_id`, `cake`.`name` AS `A_name`, `filling`.`id` AS `B_id`, `filling`.`name` AS `B_name` FROM `cake` LEFT JOIN `cake_filling` ON `cake`.`id` = `cake_filling`.`cake_id` LEFT JOIN `filling` ON `cake_filling`.`filling_id` = `filling`.`id`

(Model { id: 1, name: "New York Cheese" }, Model { id: 1, name: "Vanilla" })

(Model { id: 1, name: "New York Cheese" }, Model { id: 2, name: "Lemon" })

(Model { id: 2, name: "Chocolate Forest" }, Model { id: 2, name: "Lemon" })

(Model { id: 2, name: "Chocolate Forest" }, Model { id: 3, name: "Mango" })

```