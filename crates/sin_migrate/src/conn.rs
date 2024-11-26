use scylla::{Session, SessionBuilder};

pub(crate) struct Conn {
    pub(crate) conn: Session,
}

impl Conn {
    pub(crate) async fn from_url(
        node: String,
        username: Option<String>,
        password: Option<String>,
        keyspace: String,
    ) -> Self {
        let mut session = SessionBuilder::new()
            .known_node(node)
            .use_keyspace(keyspace, false);

        if let Some(username) = username {
            session = session.user(username, password.expect("Password shouldn't be empty"));
        }

        Self {
            conn: session.build().await.expect("Unable to reach the database"),
        }
    }
}

impl std::ops::Deref for Conn {
    type Target = Session;

    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}
