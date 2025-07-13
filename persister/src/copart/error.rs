use browser::error::BrowserError;
use diesel_async::pooled_connection::deadpool::PoolError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PersisterError {
    #[error("browser error: {0}")]
    BrowserError(#[from] BrowserError),
    #[error("pg pool error: {0}")]
    PgPoolError(#[from] PoolError),
    #[error("diesel error: {0}")]
    DieselError(#[from] diesel::result::Error),
}
