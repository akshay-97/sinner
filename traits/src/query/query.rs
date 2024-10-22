use crate::data_types::types::{CqlMap, Consistency};
use crate::nosql::interface::{NoSql, CqlStore};
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

trait QueryInterface<S: CqlStore> : QueryResultType{
    fn execute(self, store : &S) -> Result<Self::Output, QueryError>;
    fn into_output(query_output : S::Output) -> Option<Self::Output>;
    //fn into_iter_output(query_output : S::Output) -> Option<IntoIterator
    fn to_statement(self) -> String;
}

struct FindOne<T: NoSql>{
    wh_clause: CqlMap,
    _model : PhantomData<T>
}

impl <T: NoSql> QueryResultType for FindOne<T>{
    type Output = Option<T>;
}

impl <T:NoSql> QueryInterface<crate::nosql::interface::AstraClient> for FindOne<T>{
    fn execute(self, store : &crate::nosql::interface::AstraClient) -> Result<Self::Output, QueryError> {
        
    }

    fn to_statement(self) -> String{

    }

    fn into_output(query_output : <crate::nosql::interface::AstraClient as CqlStore>::Output) -> Option<Self::Output> {
        
    }
}

struct FindAll<T: NoSql>{
    wh_clause: CqlMap,
    _model : PhantomData<T>
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
