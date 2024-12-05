use crate::{conn::Conn, consts, error, setup::Schema};
use std::collections::BTreeMap;
use std::path::PathBuf;

use traits::query::{
    client::{Insertable, Selectable},
    query::QueryInterface,
};

pub fn parse_cql_statements(file: &PathBuf) -> error::CustomResult<Vec<String>> {
    Ok(std::fs::read_to_string(&file)?
        .replace(consts::PLACEHOLDER, "")
        .replace("\n", "")
        .split_inclusive(";")
        .map(|s| s.to_string())
        .collect::<Vec<String>>())
}

pub async fn is_fresh_migration(conn: &Conn) -> bool {
    Schema::select_all()
        .limit(1)
        .build()
        .execute(&conn.conn)
        .await
        .is_err()
}

pub fn get_migration_tree(dir: std::fs::ReadDir) -> error::CustomResult<BTreeMap<String, PathBuf>> {
    let mut tree = BTreeMap::<String, PathBuf>::new();

    for dir in dir.into_iter() {
        let path = dir?.path();
        let version = extract_version(&path)?;
        tree.insert(version, path);
    }

    Ok(tree)
}

pub async fn run_cql_queries(
    file: &PathBuf,
    conn: &Conn,
    version: &str,
    is_run: bool,
) -> error::CustomResult<()> {
    let cql_statements = parse_cql_statements(file)?;

    for statement in cql_statements {
        conn.query_unpaged(statement, &[]).await?;
    }
    insert_metadata(conn, version, is_run).await?;
    Ok(())
}

pub fn extract_version(path: &PathBuf) -> error::CustomResult<String> {
    path.file_name()
        .map(|st| {
            let s = st.to_string_lossy().to_string();
            s.split_once("_").map(|s| s.0.to_string())
        })
        .flatten()
        .ok_or(error::Error::MigrationPathError)
}

async fn insert_metadata(conn: &Conn, version: &str, is_run: bool) -> error::CustomResult<()> {
    let now = time::OffsetDateTime::now_utc();
    let schema = Schema::new(version.to_string(), now, is_run);
    let create = schema.create().build();
    create.execute(&conn.conn).await?;

    Ok(())
}
