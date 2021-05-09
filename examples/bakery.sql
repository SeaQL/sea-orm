DROP TABLE IF EXISTS `cake`;

CREATE TABLE `cake` (
  `id` int NOT NULL AUTO_INCREMENT,
  `name` varchar(255) DEFAULT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;

INSERT INTO `cake` (`id`, `name`) VALUES
	(1, 'New York Cheese'),
	(2, 'Chocolate Fudge');

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