pub mod collection_shares;
pub mod collections;
pub mod endpoints;
pub mod groups;
pub mod users;

use sqlx::AnyPool;

pub async fn connect(database_url: &str) -> Result<AnyPool, sqlx::Error> {
    sqlx::any::install_default_drivers();
    ensure_sqlite_file(database_url).await?;
    AnyPool::connect(database_url).await
}

pub async fn migrate(pool: &AnyPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}

/// For SQLite URLs, creates the parent directory and the database file if they
/// do not already exist. No-op for other database backends.
async fn ensure_sqlite_file(database_url: &str) -> Result<(), sqlx::Error> {
    // Strip the "sqlite://" scheme; anything else is not SQLite.
    let path_str = match database_url.strip_prefix("sqlite://") {
        Some(p) => p,
        None => return Ok(()),
    };

    // In-memory databases need no file.
    if path_str == ":memory:" || path_str.is_empty() {
        return Ok(());
    }

    let path = std::path::Path::new(path_str);

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                sqlx::Error::Io(std::io::Error::new(
                    e.kind(),
                    format!(
                        "failed to create database directory {}: {e}",
                        parent.display()
                    ),
                ))
            })?;
        }
    }

    if !path.exists() {
        tokio::fs::File::create(path).await.map_err(|e| {
            sqlx::Error::Io(std::io::Error::new(
                e.kind(),
                format!("failed to create database file {}: {e}", path.display()),
            ))
        })?;
    }

    Ok(())
}
