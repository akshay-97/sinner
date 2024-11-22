use scylla::macros::{DeserializeRow, SerializeRow};
use std::collections::HashMap;
use traits::{
    data_types::types::{CqlType, FromCqlData, ToCqlData},
    nosql::interface::NoSql,
    query::client::{FilterBy, Insertable, Selectable},
};

#[derive(SerializeRow, DeserializeRow)]
pub(crate) struct Schema {
    pub(crate) version: String,
    time: time::OffsetDateTime,
}

impl Schema {
    pub(crate) fn new(version: String, time: time::OffsetDateTime) -> Self {
        Self { version, time }
    }
}
