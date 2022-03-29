use crate::{
    EntityTrait, Iden, IdenStatic, IntoMockRow, Iterable, MockRow, ModelTrait, SelectA, SelectB,
};
use std::collections::BTreeMap;

// impl<M> IntoMockRow for M
// where
//     M: ModelTrait,
// {
//     fn into_mock_row(self) -> MockRow {
//         let mut values = BTreeMap::new();
//         for col in <<M::Entity as EntityTrait>::Column>::iter() {
//             values.insert(col.to_string(), self.get(col));
//         }
//         MockRow { values }
//     }
// }

// impl<M, N> IntoMockRow for (M, N)
// where
//     M: ModelTrait,
//     N: ModelTrait,
// {
//     fn into_mock_row(self) -> MockRow {
//         let mut mapped_join = BTreeMap::new();

//         for column in <<M as ModelTrait>::Entity as EntityTrait>::Column::iter() {
//             mapped_join.insert(
//                 format!("{}{}", SelectA.as_str(), column.as_str()),
//                 self.0.get(column),
//             );
//         }
//         for column in <<N as ModelTrait>::Entity as EntityTrait>::Column::iter() {
//             mapped_join.insert(
//                 format!("{}{}", SelectB.as_str(), column.as_str()),
//                 self.1.get(column),
//             );
//         }

//         mapped_join.into_mock_row()
//     }
// }
