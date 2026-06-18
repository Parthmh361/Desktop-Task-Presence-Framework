pub mod repository;

use std::path::PathBuf;
use std::sync::Arc;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;

pub use repository::*;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new() -> Result<Arc<Self>, sqlx::Error> {
        let db_path = Self::db_path();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        // TODO Phase 2: SQLCipher encryption at rest
        let options = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Arc::new(Self { pool }))
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    fn db_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".dtpf")
            .join("tasks.db")
    }
}
