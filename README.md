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

Quickly build search models that help you filter, sort and paginate data in APIs.

## Design Goals

1. Intuitive and ergonomic

API should state the intention clearly. Provide syntax sugar for common things.

2. Fast(er) compilation

Balance between compile-time checking and compilation speed.

3. Avoid 'symbol soup'

Avoid macros with DSL, use derive macros where appropriate. Be friendly with IDE tools.

## Test Time

After some bitterness we realized it is not possible to capture everything compile time. But we don't 
want to encounter problems at run time either. The solution is to perform checking at 'test time' to
uncover problems. These checks will be removed at production so there will be no run time penalty.