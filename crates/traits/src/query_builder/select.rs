use super::conds::*;
use std::marker::PhantomData;

use crate::{
    data_types::types::CqlMap,
    nosql::interface::NoSql,
    query_builder::{query::CassandraQuery, QueryBuilder},
};

pub struct SelectBuilder<T: NoSql> {
    table: String,
    keyspace: String,
    fields: CqlMap,
    _phantom: PhantomData<T>,
}

impl<T: NoSql> SelectBuilder<T> {
    pub fn new(fields: CqlMap) -> Self {
        Self {
            table: T::table_name().to_string(),
            keyspace: T::keyspace().to_string(),
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

impl<T: NoSql> QueryBuilder for SelectBuilder<T> {
    fn walk_ast(&self, query: &mut CassandraQuery) {
        query.push_cql(&format!(
            "SELECT * FROM {}.{} ",
            &self.keyspace, &self.table
        ));
    }
}

#[derive(Default)]
pub struct SelectClause<T> {
    _query: CassandraQuery,
    fields: CqlMap,
    _phantom: PhantomData<T>,
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

    pub fn and(mut self) -> Self {
        And::new().walk_ast(&mut self._query);
        self
    }

    pub fn or(mut self) -> Self {
        Or::new().walk_ast(&mut self._query);
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

impl<T> SelectQuery<T> {
    pub fn query(&self) -> String {
        self._query.query()
    }

    pub fn binds(&self) -> CqlMap {
        self.fields.clone()
    }
}

impl<R, T: AsRef<str>> PartialEq<T> for SelectQuery<R> {
    fn eq(&self, other: &T) -> bool {
        self._query.eq(other)
    }
}
