use std::collections::HashMap;
use traits::{
    data_types::types::{CqlType, FromCqlData, ToCqlData},
    nosql::interface::NoSql,
    query::client::{FilterBy, Insertable, Selectable},
};

#[sin::nosql(partition_key = [version], table = migration_metadata, keyspace = metadata)]
pub(crate) struct Schema {
    version: String,
    time: i128,
}

impl Schema {
    pub(crate) fn new(version: String, time: i128) -> Self {
        Self { version, time }
    }
}
