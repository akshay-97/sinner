use super::conds::*;
use std::marker::PhantomData;

use crate::{
    data_types::types::CqlMap,
    query_builder::{query::CassandraQuery, QueryBuilder},
};

pub struct SelectBuilder<T> {
    table: String,
    fields: CqlMap,
    _phantom: PhantomData<T>,
}

impl<T> SelectBuilder<T> {
    pub fn new(table: String, fields: CqlMap) -> Self {
        Self {
            table,
            fields,
            _phantom: PhantomData,
        }
    }

    pub fn wh(self) -> SelectClause<T> {
        let mut query = CassandraQuery::new();
        self.walk_ast(&mut query);

        SelectClause::new(query, self.fields)
    }

    pub fn build(self) -> SelectQuery<T> {
        let mut query = CassandraQuery::new();
        self.walk_ast(&mut query);

        SelectQuery {
            _query: query,
            fields: self.fields,
            _phantom: PhantomData,
        }
    }
}

impl<T> QueryBuilder for SelectBuilder<T> {
    fn walk_ast(&self, query: &mut CassandraQuery) {
        query.push_cql(&format!("SELECT * FROM {} ", &self.table));
    }
}

#[derive(Default)]
pub struct SelectClause<T> {
    _query: CassandraQuery,
    fields: CqlMap,
    _phantom: PhantomData<T>,
}

pub struct SelectCombine<T> {
    _query: CassandraQuery,
    fields: CqlMap,
    _phantom: PhantomData<T>,
}

impl<T> SelectCombine<T> {
    pub fn new(_query: CassandraQuery, fields: CqlMap) -> Self {
        Self {
            _query,
            fields,
            _phantom: PhantomData,
        }
    }

    pub fn and(mut clause: SelectClause<T>) -> SelectClause<T> {
        And::new().walk_ast(&mut clause._query);
        clause
    }

    pub fn or(mut clause: SelectClause<T>) -> SelectClause<T> {
        Or::new().walk_ast(&mut clause._query);
        clause
    }
}

impl<T> SelectClause<T> {
    fn new(mut query: CassandraQuery, fields: CqlMap) -> Self {
        WhereClause.walk_ast(&mut query);

        Self {
            _query: query,
            fields,
            _phantom: PhantomData,
        }
    }

    pub fn eq(mut self, source: String) -> Self {
        Eq::new(source).walk_ast(&mut self._query);
        self
    }

    pub fn gt(mut self, source: String) -> Self {
        Gt::new(source).walk_ast(&mut self._query);
        self
    }

    pub fn gte(mut self, source: String) -> Self {
        Gte::new(source).walk_ast(&mut self._query);
        self
    }

    pub fn lt(mut self, source: String) -> Self {
        Lt::new(source).walk_ast(&mut self._query);
        self
    }

    pub fn lte(mut self, source: String) -> Self {
        Lte::new(source).walk_ast(&mut self._query);
        self
    }

    pub fn limit(mut self, limit: u32) -> SelectQuery<T> {
        Limit::new(limit).walk_ast(&mut self._query);

        SelectQuery {
            _query: self._query,
            fields: self.fields,
            _phantom: PhantomData,
        }
    }

    pub fn build(self) -> SelectQuery<T> {
        SelectQuery {
            _query: self._query,
            fields: self.fields,
            _phantom: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct SelectQuery<T> {
    _query: CassandraQuery,
    fields: CqlMap,
    _phantom: PhantomData<T>,
}

impl<R, T: AsRef<str>> PartialEq<T> for SelectQuery<R> {
    fn eq(&self, other: &T) -> bool {
        self._query.eq(other)
    }
}
