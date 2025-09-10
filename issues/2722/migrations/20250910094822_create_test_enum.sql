CREATE TYPE example_enum AS ENUM ('first_variant', 'second_variant');

CREATE TABLE example_table (
    id integer GENERATED ALWAYS AS IDENTITY,
    value example_enum NOT NULL,
    other_field integer
);
