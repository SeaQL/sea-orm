CREATE TABLE `src` (
  `id` int NOT NULL,
  PRIMARY KEY (`id`)
)

CREATE TABLE `dest` (
  `id` int NOT NULL,
  `src_id` int DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `fk_dest_src` (`src_id`),
  CONSTRAINT `fk_dest_src` FOREIGN KEY (`src_id`) REFERENCES `src`.`src` (`id`)
)