DROP TABLE IF EXISTS `cake`;

CREATE TABLE `cake` (
  `id` int NOT NULL AUTO_INCREMENT,
  `name` varchar(255) DEFAULT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

INSERT INTO `cake` (`id`, `name`) VALUES
	(1, 'New York Cheese'),
	(2, 'Chocolate Forest');

DROP TABLE IF EXISTS `fruit`;

CREATE TABLE `fruit` (
  `id` int NOT NULL AUTO_INCREMENT,
  `name` varchar(255) DEFAULT NULL,
  `cake_id` int DEFAULT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

INSERT INTO `fruit` (`id`, `name`, `cake_id`) VALUES
  (1, 'Blueberry', 1),
  (2, 'Rasberry', 1),
  (3, 'Strawberry', 2);

INSERT INTO `fruit` (`name`, `cake_id`) VALUES
  ('Apple', NULL),
  ('Banana', NULL),
  ('Cherry', NULL),
  ('Lemon', NULL),
  ('Orange', NULL),
  ('Pineapple', NULL);

DROP TABLE IF EXISTS `filling`;

CREATE TABLE `filling` (
  `id` int NOT NULL AUTO_INCREMENT,
  `name` varchar(255) DEFAULT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

INSERT INTO `filling` (`id`, `name`) VALUES
  (1, 'Vanilla'),
  (2, 'Lemon'),
  (3, 'Mango');

DROP TABLE IF EXISTS `cake_filling`;

CREATE TABLE `cake_filling` (
  `cake_id` int NOT NULL,
  `filling_id` int NOT NULL,
  PRIMARY KEY (`cake_id`, `filling_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

INSERT INTO `cake_filling` (`cake_id`, `filling_id`) VALUES
  (1, 1),
  (1, 2),
  (2, 2),
  (2, 3);