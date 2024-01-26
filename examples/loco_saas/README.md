> Adapted from https://github.com/loco-rs/loco/tree/master/starters/saas

# Welcome to Loco :train:

Loco is a web and API framework running on Rust.

This is the **SaaS starter** which includes a `User` model and authentication based on JWT.


## Quick Start

You need:

* A local postgres instance
* A local Redis instance

Check out your development [configuration](config/development.yaml).

> To configure a database , please run a local postgres database with <code>loco:loco</code> and a db named <code>[app name]_development.</code>: 
<code>docker run -d -p 5432:5432 -e POSTGRES_USER=loco -e POSTGRES_DB=[app name]_development -e POSTGRES_PASSWORD="loco" postgres:15.3-alpine</code>

Now start your app:

```
$ cargo loco start
Finished dev [unoptimized + debuginfo] target(s) in 21.63s
    Running `target/debug/myapp start`

    :
    :
    :

controller/app_routes.rs:203: [Middleware] Adding log trace id

                      ▄     ▀
                                 ▀  ▄
                  ▄       ▀     ▄  ▄ ▄▀
                                    ▄ ▀▄▄
                        ▄     ▀    ▀  ▀▄▀█▄
                                          ▀█▄
▄▄▄▄▄▄▄  ▄▄▄▄▄▄▄▄▄   ▄▄▄▄▄▄▄▄▄▄▄ ▄▄▄▄▄▄▄▄▄ ▀▀█
 ██████  █████   ███ █████   ███ █████   ███ ▀█
 ██████  █████   ███ █████   ▀▀▀ █████   ███ ▄█▄
 ██████  █████   ███ █████       █████   ███ ████▄
 ██████  █████   ███ █████   ▄▄▄ █████   ███ █████
 ██████  █████   ███  ████   ███ █████   ███ ████▀
   ▀▀▀██▄ ▀▀▀▀▀▀▀▀▀▀  ▀▀▀▀▀▀▀▀▀▀  ▀▀▀▀▀▀▀▀▀▀ ██▀
       ▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀

started on port 3000
```

Seed the database:

```
$ cargo loco task seed_data
    Finished dev [unoptimized + debuginfo] target(s) in 0.49s
     Running `target/debug/loco_starter_template-cli task seed_data`
2024-01-26T07:16:06.357285Z  INFO loco_rs::config: loading environment from selected_path="config/development.yaml"
2024-01-26T07:16:06.363480Z  WARN loco_rs::boot: pretty backtraces are enabled (this is great for development but has a runtime cost for production. disable with `logger.pretty_backtrace` in your config yaml)
```

List all notes:

```
$ curl localhost:3000/api/notes
[{"created_at":"2023-11-12T12:34:56.789","updated_at":"2023-11-12T12:34:56.789","id":1,"title":"Loco note 1","content":"Loco note 1 content"},{"created_at":"2023-11-12T12:34:56.789","updated_at":"2023-11-12T12:34:56.789","id":2,"title":"Loco note 2","content":"Loco note 2 content"}]%
```

## Getting help

Check out [a quick tour](https://loco.rs/docs/getting-started/tour/) or [the complete guide](https://loco.rs/docs/getting-started/guide/).
