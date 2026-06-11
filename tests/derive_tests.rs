use sea_orm::{FromQueryResult, TryGetable};

#[derive(FromQueryResult)]
struct SimpleTest {
    _foo: i32,
    _bar: String,
}

#[derive(FromQueryResult)]
struct GenericTest<T: TryGetable> {
    _foo: i32,
    _bar: T,
}

#[derive(FromQueryResult)]
struct DoubleGenericTest<T: TryGetable, F: TryGetable> {
    _foo: T,
    _bar: F,
}

#[derive(FromQueryResult)]
struct BoundsGenericTest<T: TryGetable + Copy + Clone + 'static> {
    _foo: T,
}

#[derive(FromQueryResult)]
struct WhereGenericTest<T>
where
    T: TryGetable + Copy + Clone + 'static,
{
    _foo: T,
}

#[derive(FromQueryResult)]
struct AlreadySpecifiedBoundsGenericTest<T: TryGetable> {
    _foo: T,
}

#[derive(FromQueryResult)]
struct MixedGenericTest<T: TryGetable + Clone, F>
where
    F: TryGetable + Copy + Clone + 'static,
{
    _foo: T,
    _bar: F,
}

trait MyTrait {
    type Item: TryGetable;
}

#[derive(FromQueryResult)]
struct TraitAssociateTypeTest<T>
where
    T: MyTrait,
{
    _foo: T::Item,
}

#[derive(FromQueryResult)]
struct FromQueryAttributeTests {
    #[sea_orm(skip)]
    _foo: i32,
    _bar: String,
}

#[derive(FromQueryResult)]
struct FromQueryResultNested {
    #[sea_orm(nested)]
    _test: SimpleTest,
}

#[cfg(feature = "postgres-array")]
mod postgres_array {
    use crate::FromQueryResult;
    use sea_orm::DeriveValueType;

    #[derive(DeriveValueType)]
    pub struct IngredientId(i32);

    #[derive(Copy, Clone, Debug, PartialEq, Eq, DeriveValueType)]
    #[sea_orm(value_type = "String")]
    pub struct NumericLabel {
        pub value: i64,
    }

    impl std::fmt::Display for NumericLabel {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.value)
        }
    }

    impl std::str::FromStr for NumericLabel {
        type Err = std::num::ParseIntError;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Self { value: s.parse()? })
        }
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq, DeriveValueType)]
    #[sea_orm(value_type = "String")]
    pub enum TextureKind {
        Hard,
        Soft,
    }

    impl std::fmt::Display for TextureKind {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{}",
                match self {
                    Self::Hard => "hard",
                    Self::Soft => "soft",
                }
            )
        }
    }

    impl std::str::FromStr for TextureKind {
        type Err = sea_query::ValueTypeErr;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(match s {
                "hard" => Self::Hard,
                "soft" => Self::Soft,
                _ => return Err(sea_query::ValueTypeErr),
            })
        }
    }

    #[derive(FromQueryResult)]
    pub struct IngredientPathRow {
        pub ingredient_path: Vec<IngredientId>,
        pub numeric_label_path: Vec<NumericLabel>,
        pub texture_path: Vec<TextureKind>,
    }
}
