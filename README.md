<div align="center">

  <img src="docs/SeaORM banner.png"/>

  <h1>SeaORM</h1>

  <p>
    <strong>An async ORM for Rust</strong>
  </p>

  <sub>Built with 🔥 by 🌊🦀🐚</sub>

</div>

# SeaORM - An async ORM for Rust

Inspired by ActiveRecord, Eloquent and TypeORM, SeaORM aims to provide you an intuitive and ergonomic 
API to make working with databases in Rust a first-class experience.

> This is an early WIP of SeaORM, and is not yet published. See [example](examples/sqlx-mysql/src) for demo usage.

## Features

1. Async

Relying on SQLx, SeaORM is a new library with async support from day 1.

2. Dynamic

Built upon SeaQuery, a dynamic query builder, SeaORM allows you to build complex queries without 'fighting the ORM'.

3. Testable

Use mock connections to write unit tests for your logic.

4. API oriented

Quickly build search models that help you join, filter, sort and paginate data in APIs.
