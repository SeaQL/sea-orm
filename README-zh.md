<div align="center">

  <img alt="SeaORM" src="https://www.sea-ql.org/blog/img/SeaORM 2.0 Banner.png"/>

  <h1></h1>
  <h3>一个强大且动态的 Rust ORM</h3>

  [![crate](https://img.shields.io/crates/v/sea-orm.svg)](https://crates.io/crates/sea-orm)
  [![build status](https://github.com/SeaQL/sea-orm/actions/workflows/rust.yml/badge.svg)](https://github.com/SeaQL/sea-orm/actions/workflows/rust.yml)
  [![GitHub stars](https://img.shields.io/github/stars/SeaQL/sea-orm.svg?style=social&label=Star&maxAge=1)](https://github.com/SeaQL/sea-orm/stargazers/)
  <br>请给我们一个 ⭐ 以支持我们！

</div>

# 🐚 SeaORM

SeaORM 是一个关系型 ORM，帮助你在 Rust 中构建 Web 服务，同时提供动态语言的使用体验。

### 高级关系

以高层次、概念化的方式建模复杂关系：一对一、一对多、多对多，甚至自引用。

### 熟悉的概念

受 Ruby、Python 和 Node.js 生态系统中流行 ORM 的启发，SeaORM 提供的开发体验让你感觉似曾相识。

### 功能丰富

SeaORM 是一个功能齐全的 ORM，内置过滤、分页和嵌套查询，加速构建 REST、GraphQL 和 gRPC API。

### 生产就绪

SeaORM 周下载量超过 25 万次，已被全球的初创企业和大型企业采用，适用于生产环境。

## 快速开始

[![Discord](https://img.shields.io/discord/873880840487206962?label=Discord)](https://discord.com/invite/uCPdDXzbdv)
加入我们的 Discord 服务器，与其他成员交流！

+ [中文文档](https://www.sea-ql.org/SeaORM/zh-CN/docs/index/)

集成示例：

+ [Actix 示例](https://github.com/SeaQL/sea-orm/tree/master/examples/actix_example)
+ [Axum 示例](https://github.com/SeaQL/sea-orm/tree/master/examples/axum_example)
+ [GraphQL 示例](https://github.com/SeaQL/sea-orm/tree/master/examples/graphql_example)
+ [jsonrpsee 示例](https://github.com/SeaQL/sea-orm/tree/master/examples/jsonrpsee_example)
+ [Loco 示例](https://github.com/SeaQL/sea-orm/tree/master/examples/loco_example) / [Loco REST 入门](https://github.com/SeaQL/sea-orm/tree/master/examples/loco_starter)
+ [Poem 示例](https://github.com/SeaQL/sea-orm/tree/master/examples/poem_example)
+ [Rocket 示例](https://github.com/SeaQL/sea-orm/tree/master/examples/rocket_example) / [Rocket OpenAPI 示例](https://github.com/SeaQL/sea-orm/tree/master/examples/rocket_okapi_example)
+ [Salvo 示例](https://github.com/SeaQL/sea-orm/tree/master/examples/salvo_example)
+ [Tonic 示例](https://github.com/SeaQL/sea-orm/tree/master/examples/tonic_example)
+ [Seaography 示例 (Bakery)](https://github.com/SeaQL/sea-orm/tree/master/examples/seaography_example) / [Seaography 示例 (Sakila)](https://github.com/SeaQL/seaography/tree/main/examples/sqlite)

如果你想要一个简洁的单文件示例来展示 SeaORM 的精华，可以试试：
+ [快速入门](https://github.com/SeaQL/sea-orm/blob/master/examples/quickstart/src/main.rs)

让我们快速了解一下 SeaORM 的独特功能。

## 灵活的实体格式
你不需要手写这些！实体文件可以使用 `sea-orm-cli` 从现有数据库生成，
以下代码通过 `--entity-format dense` 生成 *(2.0 新增)*。
```rust
mod user {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "user")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub name: String,
        #[sea_orm(unique)]
        pub email: String,
        #[sea_orm(has_one)]
        pub profile: HasOne<super::profile::Entity>,
        #[sea_orm(has_many)]
        pub posts: HasMany<super::post::Entity>,
    }
}
mod post {
    use sea_orm::entity::prelude::*;

    #[sea_orm::model]
    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "post")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub user_id: i32,
        pub title: String,
        #[sea_orm(belongs_to, from = "user_id", to = "id")]
        pub author: HasOne<super::user::Entity>,
        #[sea_orm(has_many, via = "post_tag")] // 多对多关系，使用中间表
        pub tags: HasMany<super::tag::Entity>,
    }
}
```

## 智能实体加载器
实体加载器智能地对一对一关系使用 join，对一对多关系使用 data loader，
即使在执行嵌套查询时也能消除 N+1 问题。
```rust
// 加载路径:
// user -> profile
// user -> post
//         post -> post_tag -> tag
let smart_user = user::Entity::load()
    .filter_by_id(42) // 等价于 .filter(user::COLUMN.id.eq(42))
    .with(profile::Entity) // 一对一使用 join
    .with((post::Entity, tag::Entity)) // 一对多使用 data loader
    .one(db)
    .await?
    .unwrap();

// 底层执行 3 个查询:
// 1. SELECT FROM user JOIN profile WHERE id = $
// 2. SELECT FROM post WHERE user_id IN (..)
// 3. SELECT FROM tag JOIN post_tag WHERE post_id IN (..)

smart_user
    == user::ModelEx {
        id: 42,
        name: "Bob".into(),
        email: "bob@sea-ql.org".into(),
        profile: HasOne::Loaded(
            profile::ModelEx {
                picture: "image.jpg".into(),
            }
            .into(),
        ),
        posts: HasMany::Loaded(vec![post::ModelEx {
            title: "Nice weather".into(),
            tags: HasMany::Loaded(vec![tag::ModelEx {
                tag: "sunny".into(),
            }]),
        }]),
    };
```

## ActiveModel：简化嵌套持久化
通过流畅的 builder API，在单次操作中持久化整个对象图：用户、个人资料（一对一）、
帖子（一对多）和标签（多对多）。SeaORM 自动确定依赖关系，
以正确的顺序插入或删除对象。

```rust
// 创建上面展示的嵌套对象:
let user = user::ActiveModel::builder()
    .set_name("Bob")
    .set_email("bob@sea-ql.org")
    .set_profile(profile::ActiveModel::builder().set_picture("image.jpg"))
    .add_post(
        post::ActiveModel::builder()
            .set_title("Nice weather")
            .add_tag(tag::ActiveModel::builder().set_tag("sunny")),
    )
    .save(db)
    .await?;
```

## Schema 优先还是实体优先？你的选择

SeaORM 提供了强大的迁移系统，让你轻松创建表、修改 Schema 和填充数据。

SeaORM 2.0 还提供了一流的[实体优先工作流](https://www.sea-ql.org/blog/2025-10-30-sea-orm-2.0/)：
只需定义新实体或向现有实体添加列，
SeaORM 将自动检测变更并创建新的表、列、唯一键和外键。

```rust
// SeaORM 解析外键依赖，按拓扑顺序创建表。
// 需要 `entity-registry` 和 `schema-sync` feature flags。
db.get_schema_registry("my_crate::entity::*").sync(db).await;
```

## 简洁的原生 SQL

让 SeaORM 处理 95% 的事务查询。
对于过于复杂而难以表达的剩余情况，
SeaORM 仍然提供便捷的原生 SQL 支持。
```rust
let user = Item { name: "Bob" }; // 嵌套参数访问
let ids = [2, 3, 4]; // 通过 `..` 运算符展开

let user: Option<user::Model> = user::Entity::find()
    .from_raw_sql(raw_sql!(
        Sqlite,
        r#"SELECT "id", "name" FROM "user"
           WHERE "name" LIKE {user.name}
           AND "id" in ({..ids})
        "#
    ))
    .one(db)
    .await?;
```

## 同步支持

[`sea-orm-sync`](https://crates.io/crates/sea-orm-sync) 提供完整的 SeaORM API，无需异步运行时，非常适合使用 SQLite 的轻量级 CLI 程序。

参见[快速入门示例](https://github.com/SeaQL/sea-orm/blob/master/sea-orm-sync/examples/quickstart/src/main.rs)了解用法。

## 基础操作

### 查询
SeaORM 在实体层面建模一对多和多对多关系，
让你通过中间表在一次调用中遍历多对多链接。
```rust
// 查找所有模型
let cakes: Vec<cake::Model> = Cake::find().all(db).await?;

// 查找并过滤
let chocolate: Vec<cake::Model> = Cake::find()
    .filter(Cake::COLUMN.name.contains("chocolate"))
    .all(db)
    .await?;

// 查找单个模型
let cheese: Option<cake::Model> = Cake::find_by_id(1).one(db).await?;
let cheese: cake::Model = cheese.unwrap();

// 查找关联模型（惰性）
let fruit: Option<fruit::Model> = cheese.find_related(Fruit).one(db).await?;

// 查找关联模型（急切加载）：用于一对一关系
let cake_with_fruit: Vec<(cake::Model, Option<fruit::Model>)> =
    Cake::find().find_also_related(Fruit).all(db).await?;

// 查找关联模型（急切加载）：同时适用于一对多和多对多关系
let cake_with_fillings: Vec<(cake::Model, Vec<filling::Model>)> = Cake::find()
    .find_with_related(Filling) // 多对多关系会执行两次 join
    .all(db) // 行会自动按左侧实体合并
    .await?;
```
### 嵌套查询

Partial model 通过只查询所需字段来避免过度获取；
它还使编写深层嵌套的关系查询变得简单。
```rust
use sea_orm::DerivePartialModel;

#[derive(DerivePartialModel)]
#[sea_orm(entity = "cake::Entity")]
struct CakeWithFruit {
    id: i32,
    name: String,
    #[sea_orm(nested)]
    fruit: Option<fruit::Model>, // 可以是普通模型或另一个 partial model
}

let cakes: Vec<CakeWithFruit> = Cake::find()
    .left_join(fruit::Entity) // 无需指定 join 条件
    .into_partial_model() // 只会查询 partial model 中的列
    .all(db)
    .await?;
```

### 插入
SeaORM 的 ActiveModel 让你直接使用 Rust 数据结构，
通过简单的 API 进行持久化。
批量插入大量不同数据源的行也很方便。
```rust
let apple = fruit::ActiveModel {
    name: Set("Apple".to_owned()),
    ..Default::default() // 无需设置主键
};

let pear = fruit::ActiveModel {
    name: Set("Pear".to_owned()),
    ..Default::default()
};

// 插入单个：Active Record 风格
let apple = apple.insert(db).await?;
apple.id == 1;

// 插入单个：Repository 风格
let result = Fruit::insert(apple).exec(db).await?;
result.last_insert_id == 1;

// 插入多个，返回最后插入的 id
let result = Fruit::insert_many([apple, pear]).exec(db).await?;
result.last_insert_id == Some(2);
```

### 高级插入
你可以利用数据库特有的功能执行 upsert 和幂等插入。
```rust
// 插入多条并返回（需要数据库支持）
let models: Vec<fruit::Model> = Fruit::insert_many([apple, pear])
    .exec_with_returning(db)
    .await?;
models[0]
    == fruit::Model {
        id: 1, // 数据库分配的值
        name: "Apple".to_owned(),
        cake_id: None,
    };

// 使用 ON CONFLICT 在主键冲突时忽略，并提供 MySQL 特定的 polyfill
let result = Fruit::insert_many([apple, pear])
    .on_conflict_do_nothing()
    .exec(db)
    .await?;

matches!(result, TryInsertResult::Conflicted);
```

### 更新
ActiveModel 通过只更新你修改过的字段来避免竞态条件，
绝不会覆盖未改动的列。
你还可以使用流畅的查询构建 API 构造复杂的批量更新查询。
```rust
use sea_orm::sea_query::{Expr, Value};

let pear: Option<fruit::Model> = Fruit::find_by_id(1).one(db).await?;
let mut pear: fruit::ActiveModel = pear.unwrap().into();

pear.name = Set("Sweet pear".to_owned()); // 更新单个字段的值

// 更新单个：只更新修改过的列
let pear: fruit::Model = pear.update(db).await?;

// 更新多个：UPDATE "fruit" SET "cake_id" = "cake_id" + 2
//            WHERE "fruit"."name" LIKE '%Apple%'
Fruit::update_many()
    .col_expr(fruit::COLUMN.cake_id, fruit::COLUMN.cake_id.add(2))
    .filter(fruit::COLUMN.name.contains("Apple"))
    .exec(db)
    .await?;
```
### 保存
你可以使用 ActiveModel 执行"插入或更新"操作，轻松组合事务操作。
```rust
let banana = fruit::ActiveModel {
    id: NotSet,
    name: Set("Banana".to_owned()),
    ..Default::default()
};

// 创建，因为主键 `id` 是 `NotSet`
let mut banana = banana.save(db).await?;

banana.id == Unchanged(2);
banana.name = Set("Banana Mongo".to_owned());

// 更新，因为主键 `id` 已存在
let banana = banana.save(db).await?;
```
### 删除
与插入和更新一致的 ActiveModel API。
```rust
// 删除单个：Active Record 风格
let orange: Option<fruit::Model> = Fruit::find_by_id(1).one(db).await?;
let orange: fruit::Model = orange.unwrap();
orange.delete(db).await?;

// 删除单个：Repository 风格
let orange = fruit::ActiveModel {
    id: Set(2),
    ..Default::default()
};
fruit::Entity::delete(orange).exec(db).await?;

// 删除多个：DELETE FROM "fruit" WHERE "fruit"."name" LIKE '%Orange%'
fruit::Entity::delete_many()
    .filter(fruit::COLUMN.name.contains("Orange"))
    .exec(db)
    .await?;

```
### 原生 SQL 查询
`raw_sql!` 宏类似 `format!` 宏，但没有 SQL 注入风险。
它支持嵌套参数插值、数组和元组展开，甚至重复组，
为构造复杂查询提供了极大的灵活性。

```rust
#[derive(FromQueryResult)]
struct CakeWithBakery {
    name: String,
    #[sea_orm(nested)]
    bakery: Option<Bakery>,
}

#[derive(FromQueryResult)]
struct Bakery {
    #[sea_orm(alias = "bakery_name")]
    name: String,
}

let cake_ids = [2, 3, 4]; // 通过 `..` 运算符展开

// 可以将原生 SQL 与多种 API 配合使用，包括嵌套查询
let cake: Option<CakeWithBakery> = CakeWithBakery::find_by_statement(raw_sql!(
    Sqlite,
    r#"SELECT "cake"."name", "bakery"."name" AS "bakery_name"
       FROM "cake"
       LEFT JOIN "bakery" ON "cake"."bakery_id" = "bakery"."id"
       WHERE "cake"."id" IN ({..cake_ids})"#
))
.one(db)
.await?;
```

## 🧭 Seaography：即时 GraphQL API

[Seaography](https://github.com/SeaQL/seaography) 是一个构建在 SeaORM 之上的 GraphQL 框架。
Seaography 允许你快速构建 GraphQL 解析器。
只需几个命令，你就可以从 SeaORM 实体启动一个功能完备的 GraphQL 服务器，
包含过滤、分页、关系查询和变更操作！

查看 [Seaography 示例](https://github.com/SeaQL/sea-orm/tree/master/examples/seaography_example) 了解更多。

<img src="https://raw.githubusercontent.com/SeaQL/sea-orm/master/examples/seaography_example/Seaography%20example.png"/>

## 🖥️ SeaORM Pro：专业管理面板

[SeaORM Pro](https://github.com/SeaQL/sea-orm-pro/) 是一个管理面板解决方案，让你可以快速轻松地为应用程序启动管理面板——不需要前端开发技能，但有当然更好！

SeaORM Pro 已更新以支持 SeaORM 2.0 的最新功能。

特性：

+ 完整的 CRUD 操作
+ 基于 React + GraphQL 构建
+ 内置 GraphQL 解析器
+ 使用 TOML 配置自定义 UI
+ 基于角色的访问控制 *(2.0 新增)*

阅读[快速入门](https://www.sea-ql.org/sea-orm-pro/docs/install-and-config/getting-started/)指南了解更多。

![](https://raw.githubusercontent.com/SeaQL/sea-orm/refs/heads/master/docs/sea-orm-pro-dark.png#gh-dark-mode-only)
![](https://raw.githubusercontent.com/SeaQL/sea-orm/refs/heads/master/docs/sea-orm-pro-light.png#gh-light-mode-only)

## SQL Server 支持

[SQL Server for SeaORM](https://www.sea-ql.org/SeaORM-X/) 为 MSSQL 提供相同的 SeaORM API。我们移植了所有测试用例和示例，并配有 MSSQL 专属文档。如果你正在构建企业软件，可以[申请商业访问权限](https://forms.office.com/r/1MuRPJmYBR)。目前基于 SeaORM 1.0，但我们会在 SeaORM 2.0 最终发布时为现有用户提供免费升级。

## 版本发布

SeaORM 2.0 已进入候选发布阶段。我们期待你的试用，并通过[分享反馈](https://github.com/SeaQL/sea-orm/discussions/)来帮助塑造最终版本。

+ [变更日志](https://github.com/SeaQL/sea-orm/tree/master/CHANGELOG.md)

SeaORM 2.0 将是我们迄今最重要的版本——包含一些破坏性变更、大量增强功能，以及对开发者体验的明确聚焦。

+ [SeaORM 2.0 先睹为快](https://www.sea-ql.org/blog/2025-09-16-sea-orm-2.0/)
+ [深入了解 SeaORM 2.0](https://www.sea-ql.org/blog/2025-09-24-sea-orm-2.0/)
+ [SeaORM 2.0 中的基于角色的访问控制](https://www.sea-ql.org/blog/2025-09-30-sea-orm-rbac/)
+ [Seaography 2.0：强大且可扩展的 GraphQL 框架](https://www.sea-ql.org/blog/2025-10-08-seaography/)
+ [SeaORM 2.0：新实体格式](https://www.sea-ql.org/blog/2025-10-20-sea-orm-2.0/)
+ [SeaORM 2.0：实体优先工作流](https://www.sea-ql.org/blog/2025-10-30-sea-orm-2.0/)
+ [SeaORM 2.0：强类型列](https://www.sea-ql.org/blog/2025-11-11-sea-orm-2.0/)
+ [SeaORM Pro 2.0 新特性](https://www.sea-ql.org/blog/2025-11-21-whats-new-in-seaormpro-2.0/)
+ [SeaORM 2.0：嵌套 ActiveModel](https://www.sea-ql.org/blog/2025-11-25-sea-orm-2.0/)
+ [SeaORM 2.0 全面介绍](https://www.sea-ql.org/blog/2025-12-05-sea-orm-2.0/)
+ [我们如何让 SeaORM 支持同步](https://www.sea-ql.org/blog/2025-12-12-sea-orm-2.0/)
+ [SeaORM 2.0 迁移指南](https://www.sea-ql.org/blog/2026-01-12-sea-orm-2.0/)
+ [SeaORM 现已支持 Arrow 和 Parquet](https://www.sea-ql.org/blog/2026-02-22-sea-orm-arrow/)
+ [SeaORM 2.0 支持 SQL Server](https://www.sea-ql.org/blog/2026-02-25-sea-orm-x/)

如果你大量使用 SeaQuery，建议查看我们关于 SeaQuery 1.0 发布的博客文章：

+ [SeaQuery 1.0 之路](https://www.sea-ql.org/blog/2025-08-30-sea-query-1.0/)

## 许可证

根据以下任一许可证授权：

-   Apache 许可证 2.0
    ([LICENSE-APACHE](LICENSE-APACHE) 或 <http://www.apache.org/licenses/LICENSE-2.0>)
-   MIT 许可证
    ([LICENSE-MIT](LICENSE-MIT) 或 <http://opensource.org/licenses/MIT>)

由你选择。

## 贡献

除非你明确声明，否则你有意提交的任何贡献，根据 Apache-2.0 许可证的定义，都将按照上述许可证双重授权，且不附带任何附加条款或条件。

我们诚挚邀请你参与、贡献，并携手共建 Rust 的未来。

向我们的贡献者们致以最真诚的感谢！

[![Contributors](https://opencollective.com/sea-orm/contributors.svg?width=1000&button=false)](https://github.com/SeaQL/sea-orm/graphs/contributors)

## 谁在使用 SeaORM？

以下是一些使用 SeaORM 构建的优秀开源软件的简短列表。欢迎[提交你的项目](https://github.com/SeaQL/sea-orm/blob/master/COMMUNITY.md#built-with-seaorm)！

| 项目 | GitHub | 标语 |
|---------|--------|---------|
| [Zed](https://github.com/zed-industries/zed) | ![GitHub stars](https://img.shields.io/github/stars/zed-industries/zed.svg?style=social) | 高性能、多人代码编辑器 |
| [OpenObserve](https://github.com/openobserve/openobserve) | ![GitHub stars](https://img.shields.io/github/stars/openobserve/openobserve.svg?style=social) | 开源可观测性平台 |
| [RisingWave](https://github.com/risingwavelabs/risingwave) | ![GitHub stars](https://img.shields.io/github/stars/risingwavelabs/risingwave.svg?style=social) | 流处理和管理平台 |
| [LLDAP](https://github.com/nitnelave/lldap) | ![GitHub stars](https://img.shields.io/github/stars/nitnelave/lldap.svg?style=social) | 轻量级 LDAP 用户管理服务器 |
| [Warpgate](https://github.com/warp-tech/warpgate) | ![GitHub stars](https://img.shields.io/github/stars/warp-tech/warpgate.svg?style=social) | 智能 SSH 堡垒，适用于任何 SSH 客户端 |
| [Svix](https://github.com/svix/svix-webhooks) | ![GitHub stars](https://img.shields.io/github/stars/svix/svix-webhooks.svg?style=social) | 企业级 Webhooks 服务 |
| [Ryot](https://github.com/IgnisDa/ryot) | ![GitHub stars](https://img.shields.io/github/stars/ignisda/ryot.svg?style=social) | 你唯一需要的自托管追踪器 |
| [Lapdev](https://github.com/lapce/lapdev) | ![GitHub stars](https://img.shields.io/github/stars/lapce/lapdev.svg?style=social) | 自托管远程开发环境 |
| [System Initiative](https://github.com/systeminit/si) | ![GitHub stars](https://img.shields.io/github/stars/systeminit/si.svg?style=social) | DevOps 自动化平台 |
| [OctoBase](https://github.com/toeverything/OctoBase) | ![GitHub stars](https://img.shields.io/github/stars/toeverything/OctoBase.svg?style=social) | 轻量级、可扩展、离线协作数据后端 |

## 赞助

[SeaQL.org](https://www.sea-ql.org/) 是一个由热情的开发者运营的独立开源组织。如果你愿意，通过 [GitHub Sponsor](https://github.com/sponsors/SeaQL) 进行小额捐赠将不胜感激，并将大大有助于维持组织的运营。

### 金牌赞助商

<table><tr>
<td><a href="https://qdx.co/">
  <img src="https://www.sea-ql.org/static/sponsors/QDX.svg" width="138"/>
</a></td>
</tr></table>

[QDX](https://qdx.co/) 开创了量子动力学驱动的药物发现，利用人工智能和超级计算加速分子建模。
我们非常感谢 QDX 赞助 SeaORM 的开发，这是为其数据密集型应用提供支持的 SQL 工具包。

### 银牌赞助商

我们感谢银牌赞助商：Digital Ocean 赞助我们的服务器，以及 JetBrains 赞助我们的 IDE。

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

## 🦀 Rustacean 贴纸包
Rustacean 贴纸包是表达你对 Rust 热情的完美方式。我们的贴纸采用优质防水乙烯基制成，具有独特的哑光效果。

贴纸包内容：

+ SeaQL 项目的标志：SeaQL、SeaORM、SeaQuery、Seaography
+ 吉祥物：Ferris x 3、寄居蟹 Terres
+ Rustacean 文字标识

[支持 SeaQL 并获取贴纸包！](https://www.sea-ql.org/sticker-pack/) 所有收益直接用于 SeaQL 项目的持续开发。

<a href="https://www.sea-ql.org/sticker-pack/"><img alt="Rustacean Sticker Pack by SeaQL" src="https://www.sea-ql.org/static/sticker-pack-1s.jpg" width="600"/></a>
