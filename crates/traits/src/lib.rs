pub mod clients;
pub mod data_types;
pub mod nosql;
pub mod query;
pub mod query_builder;

#[macro_export]
macro_rules! impl_walk_ast_conds {
    ($name: ident, $cql: literal) => {
        pub(crate) struct $name;

        impl $name {
            pub(crate) fn new() -> Self {
                Self
            }
        }

        impl $crate::query_builder::QueryBuilder for $name {
            fn walk_ast(&self, query: &mut $crate::query_builder::query::CassandraQuery) {
                query.push_cql($cql)
            }
        }
    };
    ($name: ident, $cql: literal, $source: ty) => {
        pub(crate) struct $name {
            source: $source,
        }

        impl $name {
            pub(crate) fn new(source: $source) -> Self {
                Self { source }
            }
        }
        impl $crate::query_builder::QueryBuilder for $name {
            fn walk_ast(&self, query: &mut $crate::query_builder::query::CassandraQuery) {
                query.push_cql(&format!("{} {} ? ", self.source, $cql))
            }
        }
    };
}
