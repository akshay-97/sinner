#[derive(Default, Debug)]
pub struct CassandraQuery {
    query: String,
}

impl CassandraQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_cql(&mut self, cql: &str) {
        self.query.push_str(cql);
    }

    pub fn query(&self) -> String {
        self.query.to_string()
    }
}

impl<T: AsRef<str>> PartialEq<T> for CassandraQuery {
    fn eq(&self, other: &T) -> bool {
        self.query.eq(other.as_ref())
    }
}
