## Specifiy a database url

```
export DATABASE_URL=mysql://sea:sea@localhost/bakery
```

## Setup the Database first

Cd into `migration` folder, follow instructions there, but basically:

```
cargo run
```

## Install Seaography

```
cargo install seaography-cli@^1.0.0-rc.2
```

## Generate Seaography project

```
rm -rf graphql # this entire folder is generated
sea-orm-cli generate entity --output-dir graphql/src/entities --seaography
seaography-cli graphql graphql/src/entities $DATABASE_URL sea-orm-seaography-example
```

## Running the project

```
cd graphql
cargo run
```

## Run some queries

### Bakery -> Cake -> Baker

```graphql
{
  bakery(pagination: { page: { limit: 10, page: 0 } }, orderBy: { name: ASC }) {
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
```

### List gluten-free cakes and know where to buy them

```graphql
{
  cake(filters: { glutenFree: { eq: 1 } }) {
    nodes {
      name
      price
      glutenFree
      bakery {
        name
      }
    }
  }
}
```
