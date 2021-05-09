# SeaORM SQLx MySql example

Running:
```sh
cargo run
```

Example output:
```sh
Database { connection: SqlxMySqlPoolConnection }

find all cakes: SELECT `cake`.`id`, `cake`.`name` FROM `cake`

Model { id: 1, name: "New York Cheese" }

Model { id: 2, name: "Chocolate Fudge" }

find all fruits: SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit`

Model { id: 1, name: "Blueberry", cake_id: Some(1) }

Model { id: 2, name: "Rasberry", cake_id: Some(1) }

Model { id: 3, name: "Strawberry", cake_id: Some(2) }

find one by primary key: SELECT `cake`.`id`, `cake`.`name` FROM `cake` WHERE `cake`.`id` = 1 LIMIT 1

Model { id: 1, name: "New York Cheese" }

find models belong to: SELECT `fruit`.`id`, `fruit`.`name`, `fruit`.`cake_id` FROM `fruit` INNER JOIN `cake` ON `cake`.`id` = `fruit`.`cake_id` WHERE `cake`.`id` = 1

Model { id: 1, name: "Blueberry", cake_id: Some(1) }

Model { id: 2, name: "Rasberry", cake_id: Some(1) }

count fruits by cake: SELECT `cake`.`name`, COUNT(`fruit`.`id`) AS `num_of_fruits` FROM `cake` LEFT JOIN `fruit` ON `cake`.`id` = `fruit`.`cake_id` GROUP BY `cake`.`name`

SelectResult { name: "New York Cheese", num_of_fruits: 2 }

SelectResult { name: "Chocolate Fudge", num_of_fruits: 1 }
```