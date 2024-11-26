use crate::{
    consts, error,
    migrations::Conn,
    setup::{setup_migration, Schema},
};
use colored::Colorize;
use futures::{StreamExt, TryStreamExt};
use scylla::Session;
use std::path::PathBuf;
use std::{collections::HashMap, ops::Deref};
use traits::query::{client::Insertable, query::QueryInterface};

pub(crate) struct MigrationsToRun {
    dirs: Vec<PathBuf>,
}

impl MigrationsToRun {
    pub(crate) fn from_path(dir: PathBuf) -> error::CustomResult<Self> {
        let dirs = std::fs::read_dir(dir)?;

        let mut dirs = dirs
            .into_iter()
            .map(|s| s.map(|s| s.path()))
            .collect::<std::io::Result<Vec<_>>>()?;

        dirs.sort_unstable_by(|a, b| a.file_name().cmp(&b.file_name()));
        Ok(Self { dirs })
    }

    pub(crate) async fn run(self, conn: &Conn) -> error::CustomResult<()> {
        if Self::is_fresh_migration(conn).await {
            setup_migration(conn).await?;
        }

        let migrations_to_run = self.yet_to_run(conn).await?;

        for mut dir in self.dirs {
            let version = Self::extract_version(&dir)?;

            if migrations_to_run.contains(&version) {
                dir.push(consts::UP_CQL);

                let stmts = std::fs::read_to_string(&dir)?
                    .replace("\n", "")
                    .split_inclusive(";")
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();

                let announce = format!("Running migrations for {}", dir.to_string_lossy()).green();
                println!("{}", announce);

                Self::inner_run(conn, version, stmts).await?;
            }
        }
        Ok(())
    }

    pub(crate) async fn yet_to_run(&self, conn: &Conn) -> error::CustomResult<Vec<String>> {
        let dirs = self
            .dirs
            .iter()
            .flat_map(|s| Self::extract_version(s))
            .collect::<Vec<String>>();

        let query = "SELECT * from metadata.migration_metadata";

        let mut results = conn
            .query_iter(query, &[])
            .await?
            .rows_stream::<Schema>()
            .unwrap();

        let results = results
            .filter_map(|f| async {
                if let Ok(v) = f {
                    Some(v.version.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>()
            .await;

        let path_results = dirs
            .clone()
            .into_iter()
            .filter(|schema| !results.contains(schema))
            .collect::<Vec<String>>();

        Ok(path_results)
    }

    fn extract_version(path: &PathBuf) -> error::CustomResult<String> {
        path.file_name()
            .map(|st| {
                let s = st.to_string_lossy().to_string();
                s.split_once("_").map(|s| s.0.to_string())
            })
            .flatten()
            .ok_or(error::Error::MigrationPathError)
    }

    async fn inner_run(
        conn: &Conn,
        version: String,
        stmts: Vec<String>,
    ) -> error::CustomResult<()> {
        for stmt in stmts {
            conn.query_unpaged(stmt, &[]).await?;
        }

        Self::insert_metadata(conn, &version).await?;
        Ok(())
    }

    // TODO: Replace this with the implementation in the main library
    async fn insert_metadata(conn: &Conn, version: &str) -> error::CustomResult<()> {
        let now = time::OffsetDateTime::now_utc();
        let schema = Schema::new(version.to_string(), now);
        let create = schema.create().build();
        create.execute(&conn.conn).await?;

        Ok(())
    }

    async fn is_fresh_migration(conn: &Conn) -> bool {
        let query = consts::SIN_CHECK_MIGRATE;
        conn.query_unpaged(query, &[]).await.is_err()
    }
}

pub(crate) async fn run_migrations(dir: PathBuf, conn: &Conn) -> error::CustomResult<()> {
    let migrations = MigrationsToRun::from_path(dir)?;

    migrations.run(conn).await?;
    Ok(())
}
