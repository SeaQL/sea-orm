use crate::rbac::entity::resource::Model as Resource;

#[derive(Debug)]
pub struct Table<'a>(pub &'a str);

#[derive(Debug)]
pub struct SchemaTable<'a, 'b>(pub &'a str, pub &'b str);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceRequest {
    pub schema: Option<String>,
    pub table: String,
}

impl<'a> From<Table<'a>> for ResourceRequest {
    fn from(table: Table<'a>) -> ResourceRequest {
        ResourceRequest {
            schema: None,
            table: table.0.to_owned(),
        }
    }
}

impl<'a, 'b> From<SchemaTable<'a, 'b>> for ResourceRequest {
    fn from(schema_table: SchemaTable<'a, 'b>) -> ResourceRequest {
        ResourceRequest {
            schema: Some(schema_table.0.to_owned()),
            table: schema_table.1.to_owned(),
        }
    }
}

impl From<Resource> for ResourceRequest {
    fn from(resource: Resource) -> Self {
        Self {
            schema: resource.schema,
            table: resource.table,
        }
    }
}
