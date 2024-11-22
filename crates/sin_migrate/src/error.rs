use scylla::transport::errors::QueryError;
use thiserror::Error;

pub(crate) type CustomResult<T> = Result<T, Error>;

#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("Error while running the query in the database {0:?}")]
    QueryError(#[from] QueryError),
    #[error("Error while running the IO operation {0:?}")]
    IoError(#[from] std::io::Error),
    #[error("Error parsing the migration path please use `generate` command to generate migration folder")]
    MigrationPathError,
}
