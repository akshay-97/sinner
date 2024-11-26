use crate::{
    data_types::types::{CqlMap, CqlType},
    nosql::interface::{CqlStore, NoSql},
    query::query::{Create, FindOne, QueryError, QueryInterface},
};
use scylla::{
    prepared_statement::PreparedStatement, serialize::value::SerializeValue, QueryResult, Session,
};

pub struct ScyllaQuery {
    query_string: String,
    binds: Vec<Box<dyn SerializeValue + Send>>,
}

impl ScyllaQuery {
    fn new(query_string: String, binds: Vec<Box<dyn SerializeValue + Send>>) -> Self {
        Self {
            query_string,
            binds,
        }
    }
}

pub struct ScyllaPreparedStatement {
    binds: Vec<Box<dyn SerializeValue + Send>>,
    prepared_statement: PreparedStatement,
}

impl ScyllaPreparedStatement {
    pub fn new(
        binds: Vec<Box<dyn SerializeValue + Send>>,
        prepared_statement: PreparedStatement,
    ) -> Self {
        Self {
            binds,
            prepared_statement,
        }
    }
}

#[async_trait::async_trait]
impl<'a> CqlStore for &'a Session {
    type Output = QueryResult;
    type Statement = ScyllaQuery;
    type StoreError = ();
    type Query = ScyllaPreparedStatement;

    async fn execute(self, statement: Self::Statement) -> Result<Self::Output, Self::StoreError> {
        let query = self.into_query(statement).await;

        let result = self
            .execute_unpaged(&query.prepared_statement, query.binds)
            .await
            .map_err(|_e| ())?; // TODO : manage and propogate errors
        Ok(result)
    }

    async fn into_query(&self, statement: Self::Statement) -> Self::Query {
        let prepared_statement = self
            .prepare(statement.query_string)
            .await
            .expect("query generation from scylla failed"); //// Prepare statements on all connections concurrently underneath
        ScyllaPreparedStatement::new(statement.binds, prepared_statement)
    }
}

// Query Interface implementation
#[async_trait::async_trait]
impl<'b, T: NoSql + Send> QueryInterface<&'b Session> for FindOne<T> {
    async fn execute(self, store: &'b Session) -> Result<Self::Output, QueryError> {
        let statement = <Self as QueryInterface<&'b Session>>::into_statement(self);
        let result = <&'b Session as CqlStore>::execute(store, statement)
            .await
            .map_err(|_e| QueryError::E01)?;

        <Self as QueryInterface<&'b Session>>::into_output(result).ok_or(QueryError::E02)
    }

    fn into_output(query_output: <&'b Session as CqlStore>::Output) -> Option<Self::Output> {
        let iter = query_output.into_rows_result().ok()?;
        let res = iter.first_row::<CqlMap>().ok().and_then(
            |cql_map: std::collections::HashMap<String, CqlType>| {
                T::from_cql(&CqlType::Row(cql_map)).ok()
            },
        )?;
        Some(res)
    }

    fn into_statement(self) -> <&'b Session as CqlStore>::Statement {
        let mut res: Vec<Box<dyn SerializeValue + Send + 'static>> =
            Vec::with_capacity(self.binds.len());
        let _binds = self
            .binds
            .into_iter()
            .for_each(|(_col, val)| res.push(Box::new(val)));

        ScyllaQuery::new(self.query, res)
    }
}

#[async_trait::async_trait]
impl<'b, T: NoSql + Send> QueryInterface<&'b Session> for Create<T> {
    async fn execute(self, store: &'b Session) -> Result<Self::Output, QueryError> {
        let statement = <Self as QueryInterface<&'b Session>>::into_statement(self);

        let result = <&'b Session as CqlStore>::execute(store, statement)
            .await
            .map_err(|_e| QueryError::E01)?; //TODO: add error context here

        <Self as QueryInterface<&'b Session>>::into_output(result).ok_or(QueryError::E02)
        // TODO: add error context here
    }

    fn into_output(_query_output: <&'b Session as CqlStore>::Output) -> Option<Self::Output> {
        Some(true) // TODO: verify result
    }

    fn into_statement(self) -> <&'b Session as CqlStore>::Statement {
        if let CqlType::Row(bind_map) = T::to_cql(self.model) {
            let mut res_binds: Vec<Box<dyn SerializeValue + Send + 'static>> =
                Vec::with_capacity(bind_map.len());
            let _ = bind_map
                .into_iter()
                .map(|(_, val)| res_binds.push(Box::new(val)));
            ScyllaQuery::new(T::insert_statement().to_string(), res_binds)
        } else {
            panic!("fix me")
        }
        // TODO generate query string in query object
    }
}
