CREATE TABLE underscores (
    `id` INT UNSIGNED NOT NULL PRIMARY KEY AUTO_INCREMENT,
    `a_b_c_d` INT NOT NULL,
    `a_b_c_dd` INT NOT NULL,
    `a_b_cc_d` INT NOT NULL,
    `a_bb_c_d` INT NOT NULL,
    `aa_b_c_d` INT NOT NULL
);

INSERT INTO underscores (
    `a_b_c_d`,
    `a_b_c_dd`,
    `a_b_cc_d`,
    `a_bb_c_d`,
    `aa_b_c_d`
)
VALUES (1, 2, 3, 4, 5);
