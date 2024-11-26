use colored::Colorize;
use std::path::PathBuf;

use crate::{
    conn::Conn,
    consts,
    error::{CustomResult, Error},
    utils,
};

pub(crate) struct MigrationsRedo(PathBuf);

impl MigrationsRedo {
    pub(crate) fn new(dir: PathBuf) -> CustomResult<Self> {
        let dirs = std::fs::read_dir(dir)?;

        let mut dirs = dirs
            .into_iter()
            .map(|s| s.map(|s| s.path()))
            .collect::<std::io::Result<Vec<_>>>()?;

        dirs.sort_unstable_by(|a, b| a.file_name().cmp(&b.file_name()));

        Ok(Self(
            dirs.first().ok_or(Error::MigrationPathError)?.to_path_buf(),
        ))
    }

    pub(crate) async fn redo(mut self, conn: &Conn) -> CustomResult<()> {
        let version = utils::extract_version(&self.0)?;
        self.0.push(consts::DOWN_CQL);
        let announce = format!("Reverting migrations for {}", self.0.to_string_lossy()).green();

        println!("{}", announce);
        utils::run_cql_queries(&self.0, conn, &version, false).await?;

        self.0.pop();
        self.0.push(consts::UP_CQL);

        let announce = format!("Re-running migrations for {}", self.0.to_string_lossy()).green();
        println!("{}", announce);

        utils::run_cql_queries(&self.0, conn, &version, true).await?;

        Ok(())
    }
}
