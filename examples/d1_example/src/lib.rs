//! Cloudflare D1 Example for Sea-ORM
//!
//! This example demonstrates how to use Sea-ORM with Cloudflare D1.
//!
//! # Setup
//!
//! 1. Create a D1 database in your Cloudflare Workers project:
//!    ```bash
//!    wrangler d1 create sea-orm-d1-example
//!    ```
//!
//! 2. Add the D1 binding to your `wrangler.toml`:
//!    ```toml
//!    [[d1_databases]]
//!    binding = "DB"
//!    database_name = "sea-orm-d1-example"
//!    database_id = "your-database-id"
//!    ```
//!
//! 3. Create the schema migration:
//!    ```sql
//!    CREATE TABLE IF NOT EXISTS cake (
//!        id INTEGER PRIMARY KEY AUTOINCREMENT,
//!        name TEXT NOT NULL,
//!        price REAL DEFAULT NULL,
//!        category TEXT DEFAULT NULL
//!    );
//!    ```
//!
//! 4. Run migrations:
//!    ```bash
//!    wrangler d1 execute sea-orm-d1-example --file=./schema.sql --remote
//!    ```
//!
//! # Features Demonstrated
//!
//! - `/cakes` - Direct SQL queries using `D1Connection`
//! - `/cakes-entity` - Entity queries using `D1QueryExecutor::find_all()`
//! - `/cakes-filtered` - Entity queries with filters and ordering
//! - `/cakes-search?q=...` - Entity queries with search parameters

mod cake;

use sea_orm::{ColumnTrait, DbBackend, D1Connection, D1QueryExecutor, EntityTrait, QueryFilter, QueryOrder, Statement, Value};
use worker::{event, Context, Env, Method, Request, Response, Result};

#[event(fetch)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    // Get D1 binding from environment
    let d1 = match env.d1("DB") {
        Ok(d1) => d1,
        Err(e) => return Response::error(format!("Failed to get D1 binding: {}", e), 500),
    };

    // Connect to Sea-ORM
    let db = match sea_orm::Database::connect_d1(d1).await {
        Ok(db) => db,
        Err(e) => return Response::error(format!("Failed to connect: {}", e), 500),
    };

    // Get D1 connection for direct access
    let d1_conn = db.as_d1_connection();

    // Route handling
    let url = req.url()?;
    let path = url.path();

    match path {
        "/" => Response::ok("Welcome to Sea-ORM D1 Example! Try /cakes, /cakes-entity, /cakes-filtered, or /cakes-search"),
        "/cakes" => handle_list_cakes(d1_conn).await,
        "/cakes-entity" => handle_list_cakes_entity(d1_conn).await,
        "/cakes-filtered" => handle_filtered_cakes(d1_conn).await,
        path if path.starts_with("/cakes-search") => handle_search_cakes(d1_conn, req).await,
        path if path == "/cakes" && req.method() == Method::Post => {
            handle_create_cake(d1_conn, req).await
        }
        path if path.starts_with("/cakes/") => {
            let id = path.trim_start_matches("/cakes/");
            match req.method() {
                Method::Get => handle_get_cake(d1_conn, id).await,
                Method::Delete => handle_delete_cake(d1_conn, id).await,
                _ => Response::error("Method not allowed", 405),
            }
        }
        _ => Response::error("Not found", 404),
    }
}

/// List all cakes using the Entity pattern (D1QueryExecutor)
async fn handle_list_cakes_entity(d1_conn: &D1Connection) -> Result<Response> {
    // Use Entity::find() with D1QueryExecutor!
    let cakes: Vec<cake::Model> = match d1_conn.find_all(cake::Entity::find()).await {
        Ok(cakes) => cakes,
        Err(e) => return Response::error(format!("Query failed: {}", e), 500),
    };

    // Convert to response format
    let results: Vec<CakeResponse> = cakes
        .into_iter()
        .map(|cake| CakeResponse {
            id: cake.id,
            name: cake.name,
        })
        .collect();

    Response::from_json(&results)
}

/// List cakes with filters and ordering using Entity pattern
async fn handle_filtered_cakes(d1_conn: &D1Connection) -> Result<Response> {
    // Use Entity::find() with filter and ordering
    let cakes: Vec<cake::Model> = match d1_conn
        .find_all(
            cake::Entity::find()
                .filter(cake::Column::Category.is_not_null())
                .order_by_asc(cake::Column::Name),
        )
        .await
    {
        Ok(cakes) => cakes,
        Err(e) => return Response::error(format!("Query failed: {}", e), 500),
    };

    // Convert to response format
    let results: Vec<CakeDetailResponse> = cakes
        .into_iter()
        .map(|cake| CakeDetailResponse {
            id: cake.id,
            name: cake.name,
            price: cake.price,
            category: cake.category,
        })
        .collect();

    Response::from_json(&results)
}

/// Search cakes by name using query parameter
async fn handle_search_cakes(d1_conn: &D1Connection, req: Request) -> Result<Response> {
    let url = req.url()?;
    let query = url.query_pairs().find(|(key, _)| key == "q");
    let search_term = query.map(|(_, v)| v.to_string()).unwrap_or_default();

    if search_term.is_empty() {
        return Response::error("Missing 'q' query parameter", 400);
    }

    // Use Entity::find() with LIKE filter (case-sensitive in SQLite)
    let cakes: Vec<cake::Model> = match d1_conn
        .find_all(
            cake::Entity::find()
                .filter(cake::Column::Name.like(&format!("%{}%", search_term)))
                .order_by_asc(cake::Column::Name),
        )
        .await
    {
        Ok(cakes) => cakes,
        Err(e) => return Response::error(format!("Query failed: {}", e), 500),
    };

    let results: Vec<CakeResponse> = cakes
        .into_iter()
        .map(|cake| CakeResponse {
            id: cake.id,
            name: cake.name,
        })
        .collect();

    Response::from_json(&serde_json::json!({
        "query": search_term,
        "count": results.len(),
        "results": results
    }))
}

/// List all cakes
async fn handle_list_cakes(d1_conn: &D1Connection) -> Result<Response> {
    let stmt = Statement::from_string(
        DbBackend::Sqlite,
        "SELECT id, name FROM cake ORDER BY id".to_string(),
    );

    let cakes = match d1_conn.query_all(stmt).await {
        Ok(cakes) => cakes,
        Err(e) => return Response::error(format!("Query failed: {}", e), 500),
    };

    let mut results = Vec::new();
    for row in cakes {
        let id: i32 = match row.try_get_by("id") {
            Ok(id) => id,
            Err(e) => return Response::error(format!("Failed to get id: {}", e), 500),
        };
        let name: String = match row.try_get_by("name") {
            Ok(name) => name,
            Err(e) => return Response::error(format!("Failed to get name: {}", e), 500),
        };
        results.push(CakeResponse { id, name });
    }

    Response::from_json(&results)
}

/// Create a new cake
async fn handle_create_cake(d1_conn: &D1Connection, mut req: Request) -> Result<Response> {
    let body = match req.json::<CreateCakeRequest>().await {
        Ok(body) => body,
        Err(e) => return Response::error(format!("Invalid JSON: {}", e), 400),
    };

    let stmt = Statement::from_sql_and_values(
        DbBackend::Sqlite,
        "INSERT INTO cake (name) VALUES (?) RETURNING id, name",
        vec![Value::from(body.name)],
    );

    let result = match d1_conn.query_one(stmt).await {
        Ok(result) => result,
        Err(e) => return Response::error(format!("Query failed: {}", e), 500),
    };

    match result {
        Some(row) => {
            let id: i32 = match row.try_get_by("id") {
                Ok(id) => id,
                Err(e) => return Response::error(format!("Failed to get id: {}", e), 500),
            };
            let name: String = match row.try_get_by("name") {
                Ok(name) => name,
                Err(e) => return Response::error(format!("Failed to get name: {}", e), 500),
            };
            Response::from_json(&CakeResponse { id, name })
        }
        None => Response::error("Failed to create cake", 500),
    }
}

/// Get a cake by ID
async fn handle_get_cake(d1_conn: &D1Connection, id: &str) -> Result<Response> {
    let id: i32 = match id.parse() {
        Ok(id) => id,
        Err(_) => return Response::error("Invalid ID", 400),
    };

    let stmt = Statement::from_sql_and_values(
        DbBackend::Sqlite,
        "SELECT id, name FROM cake WHERE id = ?",
        vec![Value::from(id)],
    );

    let result = match d1_conn.query_one(stmt).await {
        Ok(result) => result,
        Err(e) => return Response::error(format!("Query failed: {}", e), 500),
    };

    match result {
        Some(row) => {
            let id: i32 = match row.try_get_by("id") {
                Ok(id) => id,
                Err(e) => return Response::error(format!("Failed to get id: {}", e), 500),
            };
            let name: String = match row.try_get_by("name") {
                Ok(name) => name,
                Err(e) => return Response::error(format!("Failed to get name: {}", e), 500),
            };
            Response::from_json(&CakeResponse { id, name })
        }
        None => Response::error("Cake not found", 404),
    }
}

/// Delete a cake by ID
async fn handle_delete_cake(d1_conn: &D1Connection, id: &str) -> Result<Response> {
    let id: i32 = match id.parse() {
        Ok(id) => id,
        Err(_) => return Response::error("Invalid ID", 400),
    };

    let stmt = Statement::from_sql_and_values(
        DbBackend::Sqlite,
        "DELETE FROM cake WHERE id = ?",
        vec![Value::from(id)],
    );

    let result = match d1_conn.execute(stmt).await {
        Ok(result) => result,
        Err(e) => return Response::error(format!("Execute failed: {}", e), 500),
    };

    if result.rows_affected() > 0 {
        Response::from_json(&serde_json::json!({ "deleted": true }))
    } else {
        Response::error("Cake not found", 404)
    }
}

/// Response type for cake
#[derive(serde::Serialize)]
struct CakeResponse {
    id: i32,
    name: String,
}

/// Response type for cake with full details (price and category)
#[derive(serde::Serialize)]
struct CakeDetailResponse {
    id: i32,
    name: String,
    price: Option<f64>,
    category: Option<String>,
}

/// Request type for creating a cake
#[derive(serde::Deserialize)]
struct CreateCakeRequest {
    name: String,
}
