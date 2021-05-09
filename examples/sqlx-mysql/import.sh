cp ../../src/tests_cfg/cake.rs src/example_cake.rs
cp ../../src/tests_cfg/fruit.rs src/example_fruit.rs

sed -i 's/^use crate::/use sea_orm::/g' src/*.rs