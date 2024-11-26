use crate::data_types::types::{FromCqlData, ToCqlData};

pub trait NoSql: FromCqlData + ToCqlData {
    fn table_name() -> &'static str;
    fn keyspace() -> &'static str;
    fn insert_statement() -> &'static str;
}

#[async_trait::async_trait]
pub trait CqlStore: Sized {
    type Output;
    type Statement;
    type StoreError;
    type Query;

    async fn execute(
        self,
        statement: Self::Statement,
    ) -> Result<Self::Output, Self::StoreError>;
    async fn into_query(&self, statement: Self::Statement) -> Self::Query;
}
