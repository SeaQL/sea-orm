# Comparison with Diesel

SeaORM and Diesel shares the same goal: to offer you a complete solution in interfacing with databases.

Both SeaORM and Diesel works with MySQL, Postgres and SQLite, so you aren't constrained going with either.

However, there are things we chose to do differently.

## Architecture

First off, perhaps the number one requested feature, async Rust support. While using async may not offer you better performance today, programming in async is an architectural decision you have to make early on. By choosing SeaORM, we together look forward to Rust's async ecosystem maturing.

Under the hood, SeaORM together with SQLx offers you a pure Rust technology stack. While Diesel is tied to native drivers. Each side has their pros and cons, so it's up to your preference.

SeaORM has an open core design. If you don't like the idea of ORM, you'll definitely still want to use SeaQuery, the underlying query builder. It is light weight and can be easily integrated into any project. The SeaQuery API is also available to you when using SeaORM, so you receive the benefits of high level abstraction while still having the power of a flexible query builder when you need it.

SeaSchema is our schema discovery library, but it is not sealed inside SeaORM. So you can reuse our data structures for developing libraries inter-operating with SeaORM.

## Programming paradigm

In addition to the sync vs async foundation, the biggest distinction between Diesel and SeaORM is static vs dynamic. Diesel has an everything-compile-time design which has its pros and cons. SeaORM is dynamic, in which things are established runtime. It offers more flexibility. While you loses some compile-time guarantee, SeaORM helps you to prove correctness by unit testing instead.

Both libraries make heavy use of traits and generics, but SeaORM generate less types from your schema. That probably means looser type/lifetime constraints and faster compilation.

You don't have to use macros when using SeaORM. We provide some derive macros for convenience, but they are entirely optional.

## Tooling

While in the Diesel ecosystem there are awesome libraries like barrel, SeaORM bundle and maintain the tooling for schema management. That means a familiar API and a more unified experience.

## Similarities

Both Diesel and SeaORM are schema first, meaning it all starts from your existing database schema, instead of starting from your OOP classes.

Both Diesel and SeaORM are relational, meaning you can do complex joins with defined relations.

## Final words

Diesel is a well established library in the Rust ecosystem. SeaORM is very new. Neither can replace the other. We hope that the Rust community will grow stronger together.