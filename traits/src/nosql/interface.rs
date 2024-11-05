
use crate::
    data_types::types::{FromCqlData, ToCqlData,};

pub trait NoSql : FromCqlData + ToCqlData {
    // fn partition_key() -> [&'static str,usize];
    // fn clustering_key() ->[&'static str, usize];
    fn table_name() -> &'static str;
    fn keyspace() -> &'static str;
    fn insert_statement() -> &'static str{
        todo!()
    }
}

#[async_trait::async_trait]
pub trait CqlStore{
    type Output;
    type Statement;
    type StoreError;
    type Query;

    async fn execute(&mut self, statement: Self::Statement) -> Result<Self::Output, Self::StoreError>;
    fn into_query(statement: Self::Statement) -> Self::Query;
}

// trait FromCqlRow : Sized {}

// struct PreparedStatement<'a>{
//     statement : &'static str,
//     binds : Binds<'a>,
// }