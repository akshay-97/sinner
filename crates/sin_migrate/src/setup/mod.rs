pub(crate) mod schema;

pub use schema::*;

use colored::Colorize;

use crate::{conn::Conn, consts, error, migrations::run::run_migrations};
use std::path::PathBuf;

pub async fn setup_migration(conn: &Conn) -> error::CustomResult<()> {
    let announce = "Setting up migrations..".green();
    println!("{}", announce);

    let stmts = consts::SIN_SETUP_UP
        .replace(consts::PLACEHOLDER, "")
        .replace("\n", "")
        .split_inclusive(";")
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    for stmt in stmts {
        conn.query_unpaged(stmt, &[]).await?;
    }

    Ok(())
}
