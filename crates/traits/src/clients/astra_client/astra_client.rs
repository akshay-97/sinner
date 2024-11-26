use crate::{
    data_types::types::{CqlType, IntoValue, ToCqlRow},
    nosql::interface::{CqlStore, NoSql},
    query::query::{Create, FindOne, QueryError, QueryInterface},
};
use stargate_grpc::{query::QueryBuilder, Query};

#[async_trait::async_trait]
impl<'a> CqlStore for &'a mut stargate_grpc::StargateClient {
    type Output = tonic::Response<stargate_grpc::proto::Response>;
    type Statement = AstrStatement;
    type StoreError = ();
    type Query = QueryBuilder;

    async fn execute(self, statement: Self::Statement) -> Result<Self::Output, Self::StoreError> {
        let query = Self::into_query(&self, statement).await.build();
        self.execute_query(query).await.map_err(|_| ())
    }

    async fn into_query(&self, statement: Self::Statement) -> Self::Query {
        let mut query = Query::builder()
            .keyspace(statement.keyspace)
            .query(statement.query_str.as_str());

        let mut enumer = statement.binds.into_iter();
        for (el, value) in enumer.next() {
            query = query.bind_name(el.as_str(), value.into());
        }
        query
    }
}

pub struct AstrStatement {
    query_str: String,
    binds: Vec<(String, Box<dyn IntoValue + Send>)>,
    keyspace: &'static str,
}

impl AstrStatement {
    pub fn new(
        query_str: String,
        binds: Vec<(String, Box<dyn IntoValue + Send>)>,
        keyspace: &'static str,
    ) -> Self {
        Self {
            query_str,
            binds,
            keyspace,
        }
    }
}

// Queries

#[async_trait::async_trait]
impl<'b, T: NoSql + Send> QueryInterface<&'b mut stargate_grpc::StargateClient> for FindOne<T> {
    async fn execute(
        self,
        store: &'b mut stargate_grpc::StargateClient,
    ) -> Result<Self::Output, QueryError> {
        let statement =
            <Self as QueryInterface<&'b mut stargate_grpc::StargateClient>>::into_statement(self);
        let result = <&'b mut stargate_grpc::StargateClient as CqlStore>::execute(store, statement)
            .await
            .map_err(|_e| QueryError::E01)?; //TODO: add error context here

        <Self as QueryInterface<&'b mut stargate_grpc::StargateClient>>::into_output(result)
            .ok_or(QueryError::E02) // TODO: add error context here
    }

    fn into_output(
        query_output: <&'b mut stargate_grpc::StargateClient as CqlStore>::Output,
    ) -> Option<Self::Output> {
        query_output
            .try_into()
            .map(|r: stargate_grpc::ResultSet| r)
            .ok()?
            .to_row_iter()
            .into_iter()
            .next()
            .and_then(|e| T::from_cql(&CqlType::Row(e)).ok())
    }

    fn into_statement(self) -> <&'b mut stargate_grpc::StargateClient as CqlStore>::Statement {
        let mut res_binds: Vec<(String, Box<dyn IntoValue + Send + 'static>)> =
            Vec::with_capacity(self.binds.len());
        let _ = self
            .binds
            .into_iter()
            .map(|(key, val)| res_binds.push((key, Box::new(val))));
        AstrStatement::new(self.query, res_binds, T::keyspace()) // TODO generate query string in query object
    }
}

#[async_trait::async_trait]
impl<'b, T: NoSql + Send> QueryInterface<&'b mut stargate_grpc::StargateClient> for Create<T> {
    async fn execute(
        self,
        store: &'b mut stargate_grpc::StargateClient,
    ) -> Result<Self::Output, QueryError> {
        let statement =
            <Self as QueryInterface<&'b mut stargate_grpc::StargateClient>>::into_statement(self);
        let result = <&'b mut stargate_grpc::StargateClient as CqlStore>::execute(store, statement)
            .await
            .map_err(|_e| QueryError::E01)?; //TODO: add error context here

        <Self as QueryInterface<&'b mut stargate_grpc::StargateClient>>::into_output(result)
            .ok_or(QueryError::E02) // TODO: add error context here
    }

    fn into_output(
        _query_output: <&'b mut stargate_grpc::StargateClient as CqlStore>::Output,
    ) -> Option<Self::Output> {
        Some(true) // TODO: verify result
    }

    fn into_statement(self) -> <&'b mut stargate_grpc::StargateClient as CqlStore>::Statement {
        if let CqlType::Row(bind_map) = T::to_cql(self.model) {
            let mut res_binds: Vec<(String, Box<dyn IntoValue + Send + 'static>)> =
                Vec::with_capacity(bind_map.len());
            let _ = bind_map
                .into_iter()
                .map(|(key, val)| res_binds.push((key, Box::new(val))));
            AstrStatement::new(T::insert_statement().to_string(), res_binds, T::keyspace())
        } else {
            panic!("fix me")
        }
        // TODO generate query string in query object
    }
}
