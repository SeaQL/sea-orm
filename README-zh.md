<div align="center">

  <img src="https://www.sea-ql.org/SeaORM/img/SeaORM banner.png"/>

  <h1>SeaORM</h1>

  <h3>🐚 一个异步且动态的 Rust ORM</h3>

  [![crate](https://img.shields.io/crates/v/sea-orm.svg)](https://crates.io/crates/sea-orm)
  [![docs](https://docs.rs/sea-orm/badge.svg)](https://docs.rs/sea-orm)
  [![build status](https://github.com/SeaQL/sea-orm/actions/workflows/rust.yml/badge.svg)](https://github.com/SeaQL/sea-orm/actions/workflows/rust.yml)

</div>

# SeaORM

[英文文档](./README.md)

#### SeaORM 是一个关系型 ORM，帮助你在 Rust 中构建 Web 服务，同时提供动态语言的使用体验。

[![GitHub stars](https://img.shields.io/github/stars/SeaQL/sea-orm.svg?style=social&label=Star&maxAge=1)](https://github.com/SeaQL/sea-orm/stargazers/)
如果你喜欢我们的工作，请考虑给我们加星标、分享并参与贡献！

完成 [SeaQL 社区调查 2025](https://www.sea-ql.org/community-survey/) 可以帮助我们维护 SeaORM！

[![Discord](https://img.shields.io/discord/873880840487206962?label=Discord)](https://discord.com/invite/uCPdDXzbdv)
加入我们的 Discord 服务器，与 SeaQL 社区的其他成员交流！

## 快速开始

+ [文档](https://www.sea-ql.org/SeaORM)
+ [教程](https://www.sea-ql.org/sea-orm-tutorial)
+ [示例集](https://www.sea-ql.org/sea-orm-cookbook)

集成示例：

+ [Actix v4 示例](https://github.com/SeaQL/sea-orm/tree/master/examples/actix_example)
+ [Axum 示例](https://github.com/SeaQL/sea-orm/tree/master/examples/axum_example)
+ [GraphQL 示例](https://github.com/SeaQL/sea-orm/tree/master/examples/graphql_example)
+ [jsonrpsee 示例](https://github.com/SeaQL/sea-orm/tree/master/examples/jsonrpsee_example)
+ [Loco TODO 示例](https://github.com/SeaQL/sea-orm/tree/master/examples/loco_example) / [Loco REST 入门](https://github.com/SeaQL/sea-orm/tree/master/examples/loco_starter)
+ [Poem 示例](https://github.com/SeaQL/sea-orm/tree/master/examples/poem_example)
+ [Rocket 示例](https://github.com/SeaQL/sea-orm/tree/master/examples/rocket_example) / [Rocket OpenAPI 示例](https://github.com/SeaQL/sea-orm/tree/master/examples/rocket_okapi_example)
+ [Salvo 示例](https://github.com/SeaQL/sea-orm/tree/master/examples/salvo_example)
+ [Tonic 示例](https://github.com/SeaQL/sea-orm/tree/master/examples/tonic_example)
+ [Seaography 示例](https://github.com/SeaQL/sea-orm/tree/master/examples/seaography_example)

## 特性

1. 异步

    基于 [SQLx](https://github.com/launchbadge/sqlx)，SeaORM 从第一天起就支持异步。

2. 动态

    构建在 [SeaQuery](https://github.com/SeaQL/sea-query) 之上，SeaORM 允许你构建复杂的动态查询。

3. 面向服务

    快速构建能够在 REST、GraphQL 和 gRPC API 中进行关联、过滤、排序和分页数据的服务。

4. 生产就绪

    SeaORM 功能丰富、测试完善，并已被多家公司和初创企业用于生产环境。

## SeaORM 快速体验

### 实体
```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "cake")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::fruit::Entity")]
    Fruit,
}

impl Related<super::fruit::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Fruit.def()
    }
}
```

### 查询
```rust
// 查找所有模型
let cakes: Vec<cake::Model> = Cake::find().all(db).await?;

// 查找并过滤
let chocolate: Vec<cake::Model> = Cake::find()
    .filter(cake::Column::Name.contains("chocolate"))
    .all(db)
    .await?;

// 查找单个模型
let cheese: Option<cake::Model> = Cake::find_by_id(1).one(db).await?;
let cheese: cake::Model = cheese.unwrap();

// 查找关联模型（惰性）
let fruits: Vec<fruit::Model> = cheese.find_related(Fruit).all(db).await?;

// 查找关联模型（急切）
let cake_with_fruits: Vec<(cake::Model, Vec<fruit::Model>)> =
    Cake::find().find_with_related(Fruit).all(db).await?;
```

### 嵌套查询

```rust
use sea_orm::DerivePartialModel;

#[derive(DerivePartialModel)]
#[sea_orm(entity = "cake::Entity")]
struct CakeWithFruit {
    id: i32,
    name: String,
    #[sea_orm(nested)]
    fruit: Option<fruit::Model>,
}

let cakes: Vec<CakeWithFruit> = cake::Entity::find()
    .left_join(fruit::Entity)
    .into_partial_model()
    .all(db)
    .await?;
```

### 插入
```rust
let apple = fruit::ActiveModel {
    name: Set("Apple".to_owned()),
    ..Default::default() // 不需要设置主键
};

let pear = fruit::ActiveModel {
    name: Set("Pear".to_owned()),
    ..Default::default()
};

// 插入单个
let pear = pear.insert(db).await?;

// 插入多个并返回最后插入的id（需要数据库与列类型支持）
Fruit::insert_many([apple, pear]).exec(db).await?;
result.last_insert_id == Some(2);
```

### 高级插入
```rust
// 插入多条记录并返回（需要数据库支持）
let models: Vec<fruit::Model> = Fruit::insert_many([apple, pear])
    .exec_with_returning(db)
    .await?;
models[0]
    == fruit::Model {
        id: 1,
        name: "Apple".to_owned(),
        cake_id: None,
    };

// 使用 ON CONFLICT，在主键冲突时忽略插入, 并为 MySQL 提供特定的 polyfill
let result = Fruit::insert_many([apple, pear])
    .on_conflict_do_nothing()
    .exec(db)
    .await?;

matches!(result, TryInsertResult::Conflicted);
```

### 更新
```rust
use sea_orm::sea_query::{Expr, Value};

let pear: Option<fruit::Model> = Fruit::find_by_id(1).one(db).await?;
let mut pear: fruit::ActiveModel = pear.unwrap().into();

pear.name = Set("Sweet pear".to_owned());

// 更新单个
let pear: fruit::Model = pear.update(db).await?;

// 更新多个：UPDATE "fruit" SET "cake_id" = NULL WHERE "fruit"."name" LIKE '%Apple%'
Fruit::update_many()
    .col_expr(fruit::Column::CakeId, Expr::value(Value::Int(None)))
    .filter(fruit::Column::Name.contains("Apple"))
    .exec(db)
    .await?;

```
### 保存
```rust
let banana = fruit::ActiveModel {
    id: NotSet,
    name: Set("Banana".to_owned()),
    ..Default::default()
};

// 创建，因为主键 `id` 是 `NotSet`
let mut banana = banana.save(db).await?;

banana.name = Set("Banana Mongo".to_owned());

// 更新，因为主键 `id` 是 `Set`
let banana = banana.save(db).await?;

```
### 删除
```rust
// 删除单个
let orange: Option<fruit::Model> = Fruit::find_by_id(1).one(db).await?;
let orange: fruit::Model = orange.unwrap();
fruit::Entity::delete(orange.into_active_model())
    .exec(db)
    .await?;

// 或者更简单
let orange: Option<fruit::Model> = Fruit::find_by_id(1).one(db).await?;
let orange: fruit::Model = orange.unwrap();
orange.delete(db).await?;

// 删除多个：DELETE FROM "fruit" WHERE "fruit"."name" LIKE 'Orange'
fruit::Entity::delete_many()
    .filter(fruit::Column::Name.contains("Orange"))
    .exec(db)
    .await?;

```

## 🧭 Seaography：即时 GraphQL API

[Seaography](https://github.com/SeaQL/seaography) 是一个构建在 SeaORM 之上的 GraphQL 框架。Seaography 允许你快速构建 GraphQL 解析器。只需几个命令，你就可以从 SeaORM 实体启动一个 GraphQL 服务器！

查看 [Seaography 示例](https://github.com/SeaQL/sea-orm/tree/master/examples/seaography_example) 了解更多。

<img src="https://raw.githubusercontent.com/SeaQL/sea-orm/master/examples/seaography_example/Seaography%20example.png"/>

## 🖥️ SeaORM Pro：轻松管理面板

[SeaORM Pro](https://www.sea-ql.org/sea-orm-pro/) 是一个管理面板解决方案，让你可以快速轻松地为应用程序启动管理面板 - 不需要前端开发技能，但有当然更好！

特性：

+ 完整的 CRUD 操作
+ 基于 React + GraphQL 构建
+ 内置 GraphQL 解析器
+ 使用简单的 TOML 自定义 UI

了解更多

+ [示例仓库](https://github.com/SeaQL/sea-orm-pro)
+ [Loco 入门](https://www.sea-ql.org/sea-orm-pro/docs/install-and-config/getting-started-loco/)
+ [Axum 入门](https://www.sea-ql.org/sea-orm-pro/docs/install-and-config/getting-started-axum/)

![](https://raw.githubusercontent.com/SeaQL/sea-orm/refs/heads/master/docs/sea-orm-pro-dark.png#gh-dark-mode-only)
![](https://raw.githubusercontent.com/SeaQL/sea-orm/refs/heads/master/docs/sea-orm-pro-light.png#gh-light-mode-only)

## 版本发布

[SeaORM 1.0](https://www.sea-ql.org/blog/2024-08-04-sea-orm-1.0/) 是一个稳定版本。1.x 版本将更新到至少 2025 年 10 月，之后我们将决定是发布 2.0 版本还是延长 1.x 的生命周期。

这并不意味着 SeaORM 已经"完成"，我们设计了一种架构，可以在不进行重大突破性更改的情况下提供新功能。事实上，更多功能即将推出！

+ [变更日志](https://github.com/SeaQL/sea-orm/tree/master/CHANGELOG.md)

### 谁在使用 SeaORM？

以下是一些使用 SeaORM 构建的优秀开源软件的简短列表。[完整列表在这里](https://github.com/SeaQL/sea-orm/blob/master/COMMUNITY.md#built-with-seaorm)。欢迎提交你的项目！

| 项目 | GitHub | 标语 |
|---------|--------|---------|
| [Zed](https://github.com/zed-industries/zed) | ![GitHub stars](https://img.shields.io/github/stars/zed-industries/zed.svg?style=social) | 高性能、多人代码编辑器 |
| [OpenObserve](https://github.com/openobserve/openobserve) | ![GitHub stars](https://img.shields.io/github/stars/openobserve/openobserve.svg?style=social) | 开源可观测性平台 |
| [RisingWave](https://github.com/risingwavelabs/risingwave) | ![GitHub stars](https://img.shields.io/github/stars/risingwavelabs/risingwave.svg?style=social) | 流处理和管理平台 |
| [LLDAP](https://github.com/nitnelave/lldap) | ![GitHub stars](https://img.shields.io/github/stars/nitnelave/lldap.svg?style=social) | 轻量级 LDAP 用户管理服务器 |
| [Warpgate](https://github.com/warp-tech/warpgate) | ![GitHub stars](https://img.shields.io/github/stars/warp-tech/warpgate.svg?style=social) | 智能 SSH 堡垒，适用于任何 SSH 客户端 |
| [Svix](https://github.com/svix/svix-webhooks) | ![GitHub stars](https://img.shields.io/github/stars/svix/svix-webhooks.svg?style=social) | 企业级 Webhooks 服务 |
| [Ryot](https://github.com/IgnisDa/ryot) | ![GitHub stars](https://img.shields.io/github/stars/ignisda/ryot.svg?style=social) | 你永远需要的唯一自托管追踪器 |
| [Lapdev](https://github.com/lapce/lapdev) | ![GitHub stars](https://img.shields.io/github/stars/lapce/lapdev.svg?style=social) | 自托管远程开发环境 |
| [System Initiative](https://github.com/systeminit/si) | ![GitHub stars](https://img.shields.io/github/stars/systeminit/si.svg?style=social) | DevOps 自动化平台 |
| [OctoBase](https://github.com/toeverything/OctoBase) | ![GitHub stars](https://img.shields.io/github/stars/toeverything/OctoBase.svg?style=social) | 轻量级、可扩展、离线协作数据后端 |

## 许可证

根据以下任一许可证授权：

- Apache 许可证 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) 或 <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT 许可证
  ([LICENSE-MIT](LICENSE-MIT) 或 <http://opensource.org/licenses/MIT>)

由你选择。

## 贡献

除非你明确声明，否则你有意提交的任何贡献，根据 Apache-2.0 许可证的定义，都将按照上述许可证双重授权，且不附带任何附加条款或条件。

我们诚挚邀请你参与、贡献，并携手共建 Rust 的未来。

向我们的贡献者们致以最真诚的感谢！

[![Contributors](https://opencollective.com/sea-orm/contributors.svg?width=1000&button=false)](https://github.com/SeaQL/sea-orm/graphs/contributors)

## 赞助

[SeaQL.org](https://www.sea-ql.org/) 是一个由热情的开发者运营的独立开源组织。如果你喜欢使用我们的库，请为我们的仓库点赞和分享。如果你愿意，通过 [GitHub Sponsor](https://github.com/sponsors/SeaQL) 进行小额捐赠将不胜感激，并将大大有助于维持组织的运营。

### 金牌赞助商

<table><tr>
<td><a href="https://qdx.co/">
  <img src="https://www.sea-ql.org/static/sponsors/QDX.svg" width="138"/>
</a></td>
</tr></table>

[QDX](https://qdx.co/) 开创了量子动力学驱动的药物发现，利用人工智能和超级计算加速分子建模。
我们非常感谢 QDX 赞助 SeaORM 的开发，这是为其数据工程工作流提供支持的 SQL 工具包。

### 银牌赞助商

我们感谢银牌赞助商：Digital Ocean 赞助我们的服务器。以及 JetBrains 赞助我们的 IDE。

<table><tr>
<td><a href="https://www.digitalocean.com/">
  <img src="https://www.sea-ql.org/static/sponsors/DigitalOcean.svg" width="125">
</a></td>

<td><a href="https://www.jetbrains.com/">
  <img src="https://www.sea-ql.org/static/sponsors/JetBrains.svg" width="125">
</a></td>
</tr></table>

## 吉祥物

Ferris 的朋友，寄居蟹 Terres 是 SeaORM 的官方吉祥物。他的爱好是收集贝壳。

<img alt="Terres" src="https://www.sea-ql.org/SeaORM/img/Terres.png" width="400"/>

### Rustacean 贴纸包 🦀

Rustacean 贴纸包是表达你对 Rust 热情的完美方式。
我们的贴纸采用优质防水乙烯基制成，具有独特的哑光 finish。
把它们贴在你的笔记本电脑、笔记本或任何设备上，展示你对 Rust 的热爱！

贴纸包内容：
- SeaQL 项目的标志：SeaQL、SeaORM、SeaQuery、Seaography、FireDBG
- SeaQL 的吉祥物：寄居蟹 Terres
- Rust 的吉祥物：螃蟹 Ferris
- Rustacean 文字

[支持 SeaQL 并获取贴纸包！](https://www.sea-ql.org/sticker-pack/) 所有收益直接用于 SeaQL 项目的持续开发。

<a href="https://www.sea-ql.org/sticker-pack/"><img alt="Rustacean Sticker Pack by SeaQL" src="https://www.sea-ql.org/static/sticker-pack-1s.jpg" width="600"/></a>
