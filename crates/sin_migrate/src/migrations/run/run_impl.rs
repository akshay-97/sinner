use crate::{
    consts, error,
    migrations::Conn,
    setup::{setup_migration, Schema},
    utils,
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
        let is_fresh_migration = utils::is_fresh_migration(conn).await;

        if is_fresh_migration {
            setup_migration(conn).await?;
        }

        let migrations_to_run = self.yet_to_run(conn).await?;

        for mut dir in self.dirs {
            let version = utils::extract_version(&dir)?;

            if is_fresh_migration || migrations_to_run.contains(&version) {
                dir.push(consts::UP_CQL);

                let announce = format!("Running migrations for {}", dir.to_string_lossy()).green();
                println!("{}", announce);

                utils::run_cql_queries(&dir, conn, &version, true).await?;
            }
        }
        Ok(())
    }

    pub(crate) async fn yet_to_run(&self, conn: &Conn) -> error::CustomResult<Vec<String>> {
        let dirs = self
            .dirs
            .iter()
            .flat_map(|s| utils::extract_version(s))
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
                    Some(v)
                } else {
                    None
                }
            })
            .collect::<Vec<Schema>>()
            .await;

        let results = dirs
            .into_iter()
            .zip(results.into_iter())
            .filter(|(version, schema)| !schema.is_run || !version.eq(&schema.version))
            .map(|(_, schema)| schema.version)
            .collect::<Vec<String>>();

        Ok(results)
    }
}

pub(crate) async fn run_migrations(dir: PathBuf, conn: &Conn) -> error::CustomResult<()> {
    let migrations = MigrationsToRun::from_path(dir)?;

    migrations.run(conn).await?;
    Ok(())
}
