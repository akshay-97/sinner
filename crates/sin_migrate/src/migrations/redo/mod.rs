use crate::{conn::Conn, error::CustomResult};
use std::path::PathBuf;

pub(crate) mod redo_impl;

pub async fn redo_migrations(dir: PathBuf, conn: Conn) -> CustomResult<()> {
    let redo = redo_impl::MigrationsRedo::new(dir)?;

    redo.redo(&conn).await?;

    Ok(())
}
