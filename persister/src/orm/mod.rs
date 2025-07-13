use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;
use std::sync::LazyLock;

pub mod models;
pub mod schema;

pub type PgPool = Pool<AsyncPgConnection>;

pub static PG_POOL: LazyLock<PgPool> = LazyLock::new(|| {
    dotenvy::from_filename("persister/.env").ok();

    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(
        std::env::var("DATABASE_URL").expect("database url not set"),
    );
    Pool::builder(config).build().expect("build pool")
});
