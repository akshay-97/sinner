use crate::data_types::types::{CqlMap, Consistency};
use crate::nosql::interface::{NoSql, CqlStore};
use crate::data_types::types::{IntoValue, AstrStatement};
use std::marker::PhantomData;

// pub struct Query<T : NoSql>{
//     consistency : Option<Consistency>,
//     op : Op,
//     _entity: PhantomData<T>
// }

trait QueryResultType{
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

struct FindOne<T: NoSql>{
    wh_clause: CqlMap,
    _model : PhantomData<T>
}

impl <T: NoSql> QueryResultType for FindOne<T>{
    type Output = Option<T>;
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
        None // TODO: impl   
    }

    fn into_statement(self) -> <stargate_grpc::StargateClient as CqlStore>::Statement{
        let mut res_binds: Vec<(String, Box<dyn IntoValue + Send + 'static>)> = Vec::with_capacity(self.wh_clause.len());
        let binds = self.wh_clause
            .into_iter()
            .map(|(key, val)| res_binds.push((key, Box::new(val))));
        AstrStatement::new("query_str", res_binds ,Self::get_keyspace()) // TODO generate query string in query object
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
