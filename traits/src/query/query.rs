use crate::data_types::types::{CqlMap, ToCqlRow, CqlType};
use crate::nosql::interface::{NoSql, CqlStore};
use crate::data_types::types::{IntoValue, AstrStatement};
use std::marker::PhantomData;

pub trait QueryResultType{
    type Output;
}

pub enum QueryError{
    E01,
    E02,
    E03
}

#[async_trait::async_trait]
pub trait QueryInterface<S: CqlStore> : QueryResultType{
    async fn execute(self, store : &mut S) -> Result<Self::Output, QueryError>;
    fn into_output(query_output : S::Output) -> Option<Self::Output>;
    fn into_statement(self) -> S::Statement;
}

pub struct FindOne<T: NoSql>{
    binds: CqlMap,
    query : String,
    _model : PhantomData<T>
}

impl <T: NoSql> QueryResultType for FindOne<T>{
    type Output = T;
}

#[async_trait::async_trait]
impl <T: NoSql + Send> QueryInterface<stargate_grpc::StargateClient> for FindOne<T> {
    async fn execute(self, store : &mut stargate_grpc::StargateClient) -> Result<Self::Output, QueryError> {
        let statement = self.into_statement();
        let result = store.execute(statement)
            .await
            .map_err(|_e| QueryError::E01)?; //TODO: add error context here
        
        Self::into_output(result)
            .ok_or(QueryError::E02) // TODO: add error context here

    }

    fn into_output(query_output : <stargate_grpc::StargateClient as CqlStore>::Output) -> Option<Self::Output> {
        query_output
            .to_row_iter()
            .into_iter()
            .next()
            .and_then(|e| {
                T::from_cql(&CqlType::Row(e)).ok()
            })   
    }

    fn into_statement(self) -> <stargate_grpc::StargateClient as CqlStore>::Statement{
        let mut res_binds: Vec<(String, Box<dyn IntoValue + Send + 'static>)> = Vec::with_capacity(self.binds.len());
        let _ = self.binds
            .into_iter()
            .map(|(key, val)| res_binds.push((key, Box::new(val))));
        AstrStatement::new(self.query, res_binds ,Self::get_keyspace()) // TODO generate query string in query object
    }

}

struct FindAll<T: NoSql>{
    wh_clause: CqlMap,
    _model : PhantomData<T>
}

impl <T : NoSql> FindOne<T>{
    fn get_keyspace() -> &'static str{
        T::keyspace()
    }

    pub fn create(binds : CqlMap, query : String) -> Self{
        Self {
            binds : binds,
            query : query,
            _model : PhantomData
        }
    }
}

impl <T: NoSql> QueryResultType for FindAll<T>{
    type Output = Vec<T>;
}

struct Update<T: NoSql>{
    wh_clause : CqlMap,
    set_clause : CqlMap,
    _model : PhantomData<T>
}

impl <T:NoSql> QueryResultType for Update<T>{
    type Output = usize;
}

struct Create<T: NoSql>{
    model : T
}

impl <T:NoSql> QueryResultType for Create<T>{
    type Output = bool;
}

struct Delete<T: NoSql>{
    wh_clause : CqlMap,
    _model: PhantomData<T>
}


impl <T:NoSql> QueryResultType for Delete<T>{
    type Output = bool;
}
