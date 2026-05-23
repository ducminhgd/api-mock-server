pub mod collection_shares;
pub mod collections;
pub mod endpoints;
pub mod groups;
pub mod users;

use sqlx::AnyPool;

pub async fn connect(database_url: &str) -> Result<AnyPool, sqlx::Error> {
    sqlx::any::install_default_drivers();
    AnyPool::connect(database_url).await
}

pub async fn migrate(pool: &AnyPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}
