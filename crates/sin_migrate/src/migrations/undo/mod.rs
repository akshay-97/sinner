pub mod undo_impl;

use crate::{conn::Conn, error::CustomResult};
use std::path::PathBuf;

pub async fn undo_migrations(dir: PathBuf, conn: Conn) -> CustomResult<()> {
    let redo = undo_impl::MigrationsUndo::new(dir)?;

    redo.undo(&conn).await?;

    Ok(())
}
