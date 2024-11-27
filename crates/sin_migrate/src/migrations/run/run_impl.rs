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
use std::{
    collections::{BTreeMap, HashMap},
    ops::Deref,
};
use traits::query::{client::Insertable, query::QueryInterface};

pub(crate) struct MigrationsToRun {
    tree: BTreeMap<String, PathBuf>,
}

impl MigrationsToRun {
    pub(crate) fn from_path(dir: PathBuf) -> error::CustomResult<Self> {
        let dirs = std::fs::read_dir(dir)?;
        let tree = utils::get_migration_tree(dirs)?;

        Ok(Self { tree })
    }

    pub(crate) async fn run(self, conn: &Conn) -> error::CustomResult<()> {
        let is_fresh_migration = utils::is_fresh_migration(conn).await;

        if is_fresh_migration {
            setup_migration(conn).await?;
        }

        let migrations_to_run = self.yet_to_run(conn).await?;

        for (version, mut dir) in self.tree {
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
        let dirs = self.tree.keys();

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

        let migrations_not_run = results.iter().filter(|s| !s.is_run).map(|s| &s.version);

        let fresh_migrations = dirs
            .into_iter()
            .filter(|version| !results.iter().any(|s| s.version.eq(*version)));

        let mut results = fresh_migrations
            .chain(migrations_not_run)
            .map(|s| s.clone())
            .collect::<Vec<String>>();

        results.sort_unstable_by(|a, b| a.cmp(b));
        Ok(results)
    }
}

pub(crate) async fn run_migrations(dir: PathBuf, conn: &Conn) -> error::CustomResult<()> {
    let migrations = MigrationsToRun::from_path(dir)?;

    migrations.run(conn).await?;
    Ok(())
}
