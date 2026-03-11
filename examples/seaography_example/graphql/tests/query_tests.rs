use async_graphql::{dynamic::*, Response};
use sea_orm::Database;
use seaography::async_graphql;

async fn schema() -> Schema {
    let database = Database::connect(
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://../bakery.db".into()),
    )
    .await
    .unwrap();
    sea_orm_seaography_example::query_root::schema(database, None, None).unwrap()
}

fn assert_eq(a: Response, b: &str) {
    assert_eq!(
        a.data.into_json().unwrap(),
        serde_json::from_str::<serde_json::Value>(b).unwrap()
    )
}

#[tokio::test]
async fn test_cake_with_bakery() {
    let schema = schema().await;

    assert_eq(
        schema
            .execute(
                r#"
                {
                  cake(filters: { name: { contains: "Chocolate" } }) {
                    nodes {
                      name
                      price
                      bakery {
                        name
                      }
                    }
                  }
                }
                "#,
            )
            .await,
        r#"
        {
          "cake": {
            "nodes": [
              {
                "name": "Chocolate Cake",
                "price": "10.25",
                "bakery": {
                  "name": "SeaSide Bakery"
                }
              },
              {
                "name": "Double Chocolate",
                "price": "12.5",
                "bakery": {
                  "name": "SeaSide Bakery"
                }
              },
              {
                "name": "Double Chocolate",
                "price": "12.5",
                "bakery": {
                  "name": "LakeSide Bakery"
                }
              }
            ]
          }
        }
        "#,
    )
}

#[tokio::test]
async fn test_cake_with_baker() {
    let schema = schema().await;

    assert_eq(
        schema
            .execute(
                r#"
                {
                  cake(
                    filters: { name: { contains: "Cheese" } }
                    having: { baker: { name: { eq: "Alice" } } }
                  ) {
                    nodes {
                      name
                      price
                      baker {
                        nodes {
                          name
                        }
                      }
                    }
                  }
                }
                "#,
            )
            .await,
        r#"
        {
          "cake": {
            "nodes": [
              {
                "name": "New York Cheese",
                "price": "12.5",
                "baker": {
                  "nodes": [
                    {
                      "name": "Alice"
                    },
                    {
                      "name": "Bob"
                    }
                  ]
                }
              },
              {
                "name": "New York Cheese",
                "price": "12.5",
                "baker": {
                  "nodes": [
                    {
                      "name": "Alice"
                    },
                    {
                      "name": "Bob"
                    }
                  ]
                }
              },
              {
                "name": "Blueburry Cheese",
                "price": "11.5",
                "baker": {
                  "nodes": [
                    {
                      "name": "Alice"
                    }
                  ]
                }
              }
            ]
          }
        }
        "#,
    )
}

#[tokio::test]
async fn test_bakery_with_cake_with_baker() {
    let schema = schema().await;

    assert_eq(
        schema
            .execute(
                r#"
                {
                  bakery(pagination: { page: { limit: 1, page: 0 } }, orderBy: { name: ASC }) {
                    nodes {
                      name
                      cake {
                        nodes {
                          name
                          price
                          baker {
                            nodes {
                              name
                            }
                          }
                        }
                      }
                    }
                  }
                }
                "#,
            )
            .await,
        r#"
        {
          "bakery": {
            "nodes": [
              {
                "name": "LakeSide Bakery",
                "cake": {
                  "nodes": [
                    {
                      "name": "Double Chocolate",
                      "price": "12.5",
                      "baker": {
                        "nodes": [
                          {
                            "name": "Bob"
                          }
                        ]
                      }
                    },
                    {
                      "name": "Lemon Cake",
                      "price": "8.8",
                      "baker": {
                        "nodes": [
                          {
                            "name": "Bob"
                          }
                        ]
                      }
                    },
                    {
                      "name": "Strawberry Cake",
                      "price": "9.9",
                      "baker": {
                        "nodes": [
                          {
                            "name": "Bob"
                          }
                        ]
                      }
                    },
                    {
                      "name": "Orange Cake",
                      "price": "6.5",
                      "baker": {
                        "nodes": [
                          {
                            "name": "Bob"
                          }
                        ]
                      }
                    },
                    {
                      "name": "New York Cheese",
                      "price": "12.5",
                      "baker": {
                        "nodes": [
                          {
                            "name": "Alice"
                          },
                          {
                            "name": "Bob"
                          }
                        ]
                      }
                    },
                    {
                      "name": "Blueburry Cheese",
                      "price": "11.5",
                      "baker": {
                        "nodes": [
                          {
                            "name": "Bob"
                          }
                        ]
                      }
                    }
                  ]
                }
              }
            ]
          }
        }
        "#,
    )
}
