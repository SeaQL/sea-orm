# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## 0.2.3 - 2021-09-22

- [[#152]] DatabaseConnection impl `Clone`
- [[#175]] Impl `TryGetableMany` for diffrent types of generics
- Codegen `TimestampWithTimeZone` fixup

[#152]: https://github.com/SeaQL/sea-orm/issues/152
[#175]: https://github.com/SeaQL/sea-orm/issues/175

## 0.2.2 - 2021-09-18

- [[#105]] Compact entity format
- [[#132]] Add ActiveModel `insert` & `update`
- [[#129]] Add `set` method to `UpdateMany`
- [[#118]] Initial lock support
- [[#167]] Add `FromQueryResult::find_by_statement`

[#105]: https://github.com/SeaQL/sea-orm/issues/105
[#132]: https://github.com/SeaQL/sea-orm/issues/132
[#129]: https://github.com/SeaQL/sea-orm/issues/129
[#118]: https://github.com/SeaQL/sea-orm/issues/118
[#167]: https://github.com/SeaQL/sea-orm/issues/167

## 0.2.1 - 2021-09-04

- Update dependencies

## 0.2.0 - 2021-09-03

- [[#37]] Rocket example
- [[#114]] `log` crate and `env-logger`
- [[#103]] `InsertResult` to return the primary key's type
- [[#89]] Represent several relations between same types by `Linked`
- [[#59]] Transforming an Entity into `TableCreateStatement`

[#37]: https://github.com/SeaQL/sea-orm/issues/37
[#114]: https://github.com/SeaQL/sea-orm/issues/114
[#103]: https://github.com/SeaQL/sea-orm/issues/103
[#89]: https://github.com/SeaQL/sea-orm/issues/89
[#59]: https://github.com/SeaQL/sea-orm/issues/59

## 0.1.3 - 2021-08-30

- [[#108]] Remove impl TryGetable for Option<T>

[#108]: https://github.com/SeaQL/sea-orm/issues/108

## 0.1.2 - 2021-08-23

- [[#68]] Added `DateTimeWithTimeZone` as supported attribute type
- [[#70]] Generate arbitrary named entity
- [[#80]] Custom column name
- [[#81]] Support join on multiple columns
- [[#99]] Implement FromStr for ColumnTrait

[#68]: https://github.com/SeaQL/sea-orm/issues/68
[#70]: https://github.com/SeaQL/sea-orm/issues/70
[#80]: https://github.com/SeaQL/sea-orm/issues/80
[#81]: https://github.com/SeaQL/sea-orm/issues/81
[#99]: https://github.com/SeaQL/sea-orm/issues/99

## 0.1.1 - 2021-08-08

- Early release of SeaORM
