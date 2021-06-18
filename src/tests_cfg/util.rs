use crate::{tests_cfg::*, IntoMockRow, MockRow};
use sea_query::Value;

impl From<cake_filling::Model> for MockRow {
    fn from(model: cake_filling::Model) -> Self {
        let map = maplit::btreemap! {
            "cake_id" => Into::<Value>::into(model.cake_id),
            "filling_id" => Into::<Value>::into(model.filling_id),
        };
        map.into_mock_row()
    }
}

impl From<cake::Model> for MockRow {
    fn from(model: cake::Model) -> Self {
        let map = maplit::btreemap! {
            "id" => Into::<Value>::into(model.id),
            "name" => Into::<Value>::into(model.name),
        };
        map.into_mock_row()
    }
}

impl From<filling::Model> for MockRow {
    fn from(model: filling::Model) -> Self {
        let map = maplit::btreemap! {
            "id" => Into::<Value>::into(model.id),
            "name" => Into::<Value>::into(model.name),
        };
        map.into_mock_row()
    }
}

impl From<fruit::Model> for MockRow {
    fn from(model: fruit::Model) -> Self {
        let map = maplit::btreemap! {
            "id" => Into::<Value>::into(model.id),
            "name" => Into::<Value>::into(model.name),
            "cake_id" => Into::<Value>::into(model.cake_id),
        };
        map.into_mock_row()
    }
}
