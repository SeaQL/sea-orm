#[test]
fn when_user_import_nothing_macro_still_works_test() {
    #[derive(sea_orm::DeriveValueType)]
    struct MyString(String);
}

#[test]
fn when_user_alias_result_macro_still_works_test() {
    #[allow(dead_code)]
    type Result<T> = std::result::Result<T, ()>;
    #[derive(sea_orm::DeriveValueType)]
    struct MyString(String);
}

#[test]
fn when_stringy_newtype_works_test() {
    #[allow(dead_code)]
    #[derive(sea_orm::DeriveValueType)]
    #[sea_orm(value_type = "String")]
    struct Foo {
        inner: i32,
    }

    impl std::fmt::Display for Foo {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.inner.fmt(f)
        }
    }

    impl std::str::FromStr for Foo {
        type Err = std::num::ParseIntError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Self { inner: s.parse()? })
        }
    }
}

#[test]
fn when_explicit_stringy_newtype_works_test() {
    #[derive(sea_orm::DeriveValueType)]
    #[sea_orm(value_type = "String")]
    struct Foo(i32);

    impl std::fmt::Display for Foo {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            self.0.fmt(f)
        }
    }

    impl std::str::FromStr for Foo {
        type Err = std::num::ParseIntError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Self(s.parse()?))
        }
    }
}

#[test]
fn when_custom_from_str_works() {
    #[derive(sea_orm::DeriveValueType)]
    #[sea_orm(
        value_type = "String",
        from_str = "Foo::from_str",
        to_str = "Foo::to_str"
    )]
    struct Foo(i32);

    impl Foo {
        fn from_str(_s: &str) -> Result<Self, std::convert::Infallible> {
            Ok(Self(42))
        }

        fn to_str(&self) -> String {
            42.to_string()
        }
    }
}
