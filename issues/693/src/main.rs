mod container;
mod content;

use container::prelude::*;
use content::prelude::*;
use sea_orm::{DbBackend, EntityTrait, QueryTrait};

fn main() {
    assert_eq!(
        Container::find().find_with_related(Content).build(DbBackend::MySql).to_string(),
        [
            "SELECT `container`.`db_id` AS `A_db_id`, `content`.`id` AS `B_id`, `content`.`container_id` AS `B_container_id`",
            "FROM `container`",
            "LEFT JOIN `content` ON `container`.`db_id` = `content`.`container_id`",
            "ORDER BY `container`.`db_id` ASC",
        ].join(" ")
    );
}
