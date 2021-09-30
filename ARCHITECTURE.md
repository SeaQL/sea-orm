# Architecture

> Let's dive under the Sea 🤿

To understand the architecture of SeaORM, let's discuss what is an ORM. ORM exists to provide abstractions over common operations you would do against a database and hide the implementation details like the actual SQL queries.

With a good ORM, you shouldn't bother to look under the API surface. Until you do. I hear you say *'abstraction leaks'*, and yes, it does.

The approach SeaORM takes is **'layered abstraction'**, where you'd dig one layer beneath if you want to. That's why we made SeaQuery into a standalone repository. It's useful on its own, and with a public API surface and a separate namespace, it's far more difficult to create confusing internal APIs than a monolithic approach.

The central idea in SeaORM is nearly everything is runtime configurable. At compile time, it does not know what the underlying database is.

What benefits does database-agnostic bring? For example, you can:

1. Make your app work on any database, depending on runtime configuration
1. Use the same models and transfer them across different databases
1. Share entities across different projects by creating a 'data structure crate', where the database is chosen by downstream 'behaviour crates'

The API of SeaORM is not a thin shell, but consist of layers, with each layer underneath being less abstract.

There are different stages when the API is being utilized.

So there are two dimensions to navigate the SeaORM code base, **'stage'** and **'abstractness'**.

First is the declaration stage. Entities and relations among them are being defined with the `EntityTrait`, `ColumnTrait` & `RelationTrait` etc.

Second is the query building stage.

The top most layer is `Entity`'s `find*`, `insert`, `update` & `delete` methods, where you can intuitively perform basic CRUD operations.

One layer down, is the `Select`, `Insert`, `Update` & `Delete` structs, where they each have their own API for more advanced operations.

One layer down, is the SeaQuery `SelectStatement`, `InsertStatement`, `UpdateStatement` & `DeleteStatement`, where they have a rich API for you to fiddle with the SQL syntax tree.

Third is the execution stage. A separate set of structs, `Selector`, `Inserter`, `Updater` & `Deleter`, are responsible for executing the statements against a database connection.

Finally is the resolution stage, when query results are converted into Rust structs for consumption.

Because only the execution and resolution stages are database specific, we have the possibility to use a different driver by replacing those.

I imagine some day, we will support a number of databases, with a matrix of different syntaxes, protocols and form-factors.