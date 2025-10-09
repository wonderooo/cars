use crate::config::CONFIG;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;
use std::sync::LazyLock;

pub mod models;
pub mod schema;

pub type PgPool = Pool<AsyncPgConnection>;

pub fn init_pg_pool() -> PgPool {
    let db_url = format!(
        "postgres://{user}:{password}@{host}/{db_name}",
        user = CONFIG.postgres.user,
        password = CONFIG.postgres.password,
        host = CONFIG.postgres.host,
        db_name = CONFIG.postgres.db_name
    );
    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(db_url);
    Pool::builder(config).build().expect("build pool")
}

pub static PG_POOL: LazyLock<PgPool> = LazyLock::new(|| init_pg_pool());
