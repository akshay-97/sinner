use crate::{consts, migrations::Conn};
use scylla::statement::{
    batch::{Batch, BatchStatement, BatchType},
    query::Query,
};

use std::path::PathBuf;

pub(crate) struct MigrationsToRun {
    dirs: Vec<PathBuf>,
}

impl MigrationsToRun {
    pub(crate) fn from_path(dir: PathBuf) -> std::io::Result<Self> {
        let dirs = std::fs::read_dir(dir)?;

        let mut dirs = dirs
            .into_iter()
            .map(|s| s.map(|s| s.path()))
            .collect::<std::io::Result<Vec<_>>>()?;

        dirs.sort_unstable_by(|a, b| a.file_name().cmp(&b.file_name()));
        Ok(Self { dirs })
    }

    pub(crate) async fn run(self, conn: Conn) -> std::io::Result<()> {
        for mut dir in self.dirs {
            dir.push(consts::UP_CQL);

            let stmts = std::fs::read_to_string(dir)?
                .split(';')
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            let query = stmts
                .into_iter()
                .map(|s| BatchStatement::Query(Query::new(s)))
                .collect::<Vec<BatchStatement>>();

            let batch = Batch::new_with_statements(BatchType::Logged, query);
            conn.batch(&batch, ((),)).await.map_err(|err| {
                dbg!(err);
                std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "Connection Refused")
            })?;
        }
        Ok(())
    }
}

pub(crate) async fn run_migrations(dir: PathBuf, conn: Conn) -> std::io::Result<()> {
    let migrations = MigrationsToRun::from_path(dir)?;

    migrations.run(conn).await?;
    Ok(())
}
