
use crate::data_types::types::{FromCqlData, ToCqlData, ToCqlRow};

pub trait NoSql : FromCqlData + ToCqlData {
    // fn partition_key() -> [&'static str,usize];
    // fn clustering_key() ->[&'static str, usize];
    fn table_name() -> &'static str;
    fn keyspace() -> &'static str;
    fn insert_statement() -> &'static str;
}


pub trait CqlStore{
    type Output: ToCqlRow;
}

// test astra impl
pub struct AstraClient{
    session : String,
}

pub struct ResultSet;

impl ToCqlRow for ResultSet{
    fn to_row(self) -> crate::data_types::types::CqlMap {
        todo!()
    }
}

impl CqlStore for AstraClient{
    type Output = ResultSet;
}

// pub trait StorageInterface{
//     fn execute<T: NoSql>(&self, query: Query<T>) -> QueryResult;

// }

// pub type QueryResult = Result<Qres, CqlError>;

// pub enum Qres{
//     Success(Vec<u8>),
//     NoRes(),
//     SuccessCount(usize),
// }

// pub enum CqlError{
//     E01,
//     E02,
//     E03
// }