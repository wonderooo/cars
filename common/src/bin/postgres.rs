use common::persistence::PG_POOL;
use diesel_async::AsyncMigrationHarness;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("src/persistence/migrations");

#[tokio::main]
async fn main() {
    let conn = PG_POOL.get().await.expect("failed to get pg connection");
    let mut harness = AsyncMigrationHarness::new(conn);
    harness
        .run_pending_migrations(MIGRATIONS)
        .expect("failed to run migrations");
}
