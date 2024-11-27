use colored::Colorize;
use std::path::PathBuf;

use crate::{
    conn::Conn,
    consts,
    error::{CustomResult, Error},
    utils,
};

pub(crate) struct MigrationsUndo(String, PathBuf);

impl MigrationsUndo {
    pub(crate) fn new(dir: PathBuf) -> CustomResult<Self> {
        let dirs = std::fs::read_dir(dir)?;
        let mut tree = utils::get_migration_tree(dirs)?;
        let first = tree.pop_last().ok_or(Error::MigrationPathError)?;

        Ok(Self(first.0, first.1))
    }

    pub(crate) async fn undo(mut self, conn: &Conn) -> CustomResult<()> {
        self.1.push(consts::DOWN_CQL);
        let announce = format!("Reverting migrations for {}", self.1.to_string_lossy()).green();

        println!("{}", announce);
        utils::run_cql_queries(&self.1, conn, &self.0, false).await?;
        Ok(())
    }
}
