use sea_query::{IntoTableRef, TableName, TableRef};

//TODO: this exists because [`sea_query::TableRef`] is very inconvenient to work with

/// A table's identity: its name plus the schema (namespace) it lives in.
///
/// Plain owned strings, so it can be compared, hashed, sorted and printed
/// directly. [`sea_query::TableRef`] can hold a subquery or an alias, is
/// neither `Eq` nor `Hash`, and nests its parts three levels deep — none of
/// which schema diffing wants to carry around.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TableId {
    /// The schema (namespace) the table lives in. `None` means the
    /// connection's current/default schema.
    pub schema: Option<String>,
    /// The bare table name, without any qualifier.
    pub name: String,
}

impl TableId {
    /// A table in the connection's default schema.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            schema: None,
            name: name.into(),
        }
    }

    /// A table in a named schema (namespace).
    pub fn qualified(schema: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            schema: Some(schema.into()),
            name: name.into(),
        }
    }

    /// The same table under a different name, staying in its schema.
    pub(crate) fn renamed(&self, name: &str) -> Self {
        Self {
            schema: self.schema.clone(),
            name: name.to_owned(),
        }
    }

    /// Read the identity out of a [`sea_query`] table reference.
    ///
    /// Panics on a subquery/values/function reference: entity definitions and
    /// schema discovery only ever produce plain tables, and those other shapes
    /// have no identity to diff against.
    pub(crate) fn from_table_ref(table_ref: &TableRef) -> Self {
        match table_ref {
            TableRef::Table(table_name, _) => Self::from_table_name(table_name),
            other => unreachable!("expected a plain table reference, got {other:?}"),
        }
    }

    /// Read the identity out of a [`sea_query`] table name. A database
    /// qualifier, if any, is dropped — nothing here addresses tables across
    /// databases.
    pub(crate) fn from_table_name(table_name: &TableName) -> Self {
        let TableName(schema, name) = table_name;
        Self {
            schema: schema.as_ref().map(|schema| schema.1.to_string()),
            name: name.to_string(),
        }
    }

    /// Rebuild a [`sea_query`] table reference, for statement building.
    pub(crate) fn to_table_ref(&self) -> TableRef {
        match &self.schema {
            Some(schema) => (schema.clone(), self.name.clone()).into_table_ref(),
            None => self.name.clone().into_table_ref(),
        }
    }
}

/// Renders as `schema.table`, or just `table` when unqualified.
impl std::fmt::Display for TableId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.schema {
            Some(schema) => write!(f, "{schema}.{}", self.name),
            None => f.write_str(&self.name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A qualified and an unqualified table ref must round-trip through
    /// `TableId` unchanged — statement building relies on it.
    #[test]
    fn table_ref_round_trips() {
        for table_ref in [
            ("sys", "widget").into_table_ref(),
            "widget".into_table_ref(),
        ] {
            let id = TableId::from_table_ref(&table_ref);
            assert_eq!(id.to_table_ref(), table_ref);
        }
    }

    #[test]
    fn display_qualifies_with_schema() {
        assert_eq!(
            TableId::qualified("sys", "widget").to_string(),
            "sys.widget"
        );
        assert_eq!(TableId::new("widget").to_string(), "widget");
    }
}
