use crate::data_types::types::CqlMap;
use crate::{
    nosql::interface::{CqlStore, NoSql},
    query_builder::select::SelectQuery,
};
use std::marker::PhantomData;

pub trait QueryResultType {
    type Output;
}

#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("E01")]
    E01,
    #[error("E02")]
    E02,
    #[error("E03")]
    E03,
}

#[async_trait::async_trait]
pub trait QueryInterface<S: CqlStore>: QueryResultType {
    async fn execute(self, store: S) -> Result<Self::Output, QueryError>
    where
        S: 'async_trait;
    fn into_output(query_output: S::Output) -> Option<Self::Output>;
    fn into_statement(self) -> S::Statement;
}

pub struct FindOne<T: NoSql> {
    pub(crate) binds: CqlMap,
    pub(crate) query: String,
    _model: PhantomData<T>,
}

impl<T: NoSql> FindOne<T> {
    pub fn create_query(binds: CqlMap, query: String) -> Self {
        Self {
            binds,
            query,
            _model: PhantomData,
        }
    }
}

impl<T: NoSql> QueryResultType for FindOne<T> {
    type Output = T;
}

impl<T: NoSql> QueryResultType for SelectQuery<T> {
    type Output = T;
}

pub struct FindAll<T: NoSql> {
    binds: CqlMap,
    query: String,
    _model: PhantomData<T>,
}

impl<T: NoSql> FindAll<T> {
    pub fn create_query(binds: CqlMap, query: String) -> Self {
        Self {
            binds: binds,
            query: query,
            _model: PhantomData,
        }
    }
}

impl<T: NoSql> FindOne<T> {
    fn get_keyspace() -> &'static str {
        T::keyspace()
    }

    fn create(binds: CqlMap, query: String) -> Self {
        Self {
            binds: binds,
            query: query,
            _model: PhantomData,
        }
    }
}

impl<T: NoSql> QueryResultType for FindAll<T> {
    type Output = Vec<T>;
}

pub struct Update<T: NoSql> {
    where_binds: CqlMap,
    set_binds: CqlMap,
    query: String,
    _model: PhantomData<T>,
}

impl<T: NoSql> Update<T> {
    pub fn create_query(where_binds: CqlMap, set_binds: CqlMap, query: String) -> Self {
        Self {
            where_binds,
            set_binds,
            query,
            _model: PhantomData,
        }
    }
}
impl<T: NoSql> QueryResultType for Update<T> {
    type Output = usize;
}

pub struct Create<T: NoSql> {
    pub(crate) model: T,
}

impl<T: NoSql> Create<T> {
    pub fn create_query(model: T) -> Self {
        Self { model }
    }
}

impl<T: NoSql> QueryResultType for Create<T> {
    type Output = bool;
}

struct Delete<T: NoSql> {
    wh_clause: CqlMap,
    _model: PhantomData<T>,
}

impl<T: NoSql> QueryResultType for Delete<T> {
    type Output = bool;
}
