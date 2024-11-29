use crate::impl_walk_ast_conds;

impl_walk_ast_conds!(And, "AND ");
impl_walk_ast_conds!(Or, "OR ");
impl_walk_ast_conds!(WhereClause, "WHERE ");
impl_walk_ast_conds!(Gt, ">", String);
impl_walk_ast_conds!(Gte, ">=", String);
impl_walk_ast_conds!(Lt, "<", String);
impl_walk_ast_conds!(Lte, "<=", String);
impl_walk_ast_conds!(Eq, "=", String);

pub struct Limit(u32);

impl Limit {
    pub fn new(l: u32) -> Self {
        Self(l)
    }
}

impl crate::query_builder::QueryBuilder for Limit {
    fn walk_ast(&self, query: &mut super::query::CassandraQuery) {
        query.push_cql(&format!("LIMIT {} ", self.0));
    }
}
