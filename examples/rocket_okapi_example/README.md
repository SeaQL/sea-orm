# Rocket and Rocket-API with SeaORM example app

1. Modify the `url` var in `api/Rocket.toml` to point to your chosen database

1. Turn on the appropriate database feature for your chosen db in `service/Cargo.toml` (the `"sqlx-postgres",` line)

1. Execute `cargo run` to start the server

1. You can go to ```http://localhost:8000/swagger-ui/index.html``` to see the api documentation about this demo project.
   ![swagger](swagger.png)
1. Additionally, you can navigate to ```http://localhost:8000/rapidoc/index.html``` to see the rapidoc format for api documentation 
   ![rapidoc](rapidoc.png)
