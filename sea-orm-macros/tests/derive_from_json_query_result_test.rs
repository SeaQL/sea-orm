use sea_orm::Value;
use sea_orm_macros::FromJsonQueryResult;
use serde::{Deserialize, Serialize};

#[test]
fn test_serialization_error_message() {
    // Test that structs can be serialized without error
    #[derive(Serialize, Deserialize, FromJsonQueryResult)]
    struct TestStruct {
        value: i32,
    }

    let data = TestStruct { value: 42 };
    let _: Value = data.into(); // Should not panic for valid data
}

#[test]
#[should_panic(
    expected = r#"Failed to serialize 'NonSerializableStruct': Error("intentionally failing serialization", line: 0, column: 0)"#
)]
fn test_serialization_of_non_serializable_struct_panics() {
    use serde::ser::{Serialize, Serializer};

    #[derive(Deserialize, FromJsonQueryResult)]
    struct NonSerializableStruct;

    impl Serialize for NonSerializableStruct {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Err(serde::ser::Error::custom(
                "intentionally failing serialization",
            ))
        }
    }

    // This should fail since NonSerializableStruct's serialization always fails
    let _ = Value::from(NonSerializableStruct);
}
