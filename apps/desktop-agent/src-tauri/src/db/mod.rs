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
        Self::open_at(Self::db_path()).await
    }

    async fn open_at(db_path: PathBuf) -> Result<Arc<Self>, sqlx::Error> {
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
        crate::paths::db_path()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    async fn temp_database() -> (Arc<Database>, PathBuf) {
        let db_path = std::env::temp_dir().join(format!("dtpf-test-{}.db", Uuid::new_v4()));
        let db = Database::open_at(db_path.clone()).await.expect("open temp db");
        (db, db_path)
    }

    #[tokio::test]
    async fn create_task_and_list_active() {
        let (db, db_path) = temp_database().await;

        let task = db
            .create_task(&NewTask {
                source_app_id: "demo-app".into(),
                title: "Test sticky".into(),
                body: Some("Hello".into()),
                priority: 1,
                color: "#FFE066".into(),
                position_x: Some(10),
                position_y: Some(20),
                monitor_id: None,
                remind_at: None,
                metadata: None,
            })
            .await
            .expect("create task");

        assert_eq!(task.title, "Test sticky");
        assert_eq!(task.status, "active");

        let active = db.list_active_tasks().await.expect("list active");
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, task.id);

        let count = db.count_active_tasks().await.expect("count");
        assert_eq!(count, 1);

        std::fs::remove_file(db_path).ok();
    }

    #[tokio::test]
    async fn register_app_and_lookup_by_token() {
        let (db, db_path) = temp_database().await;

        db.register_app(&NewAppRegistration {
            app_id: "app-1".into(),
            app_name: "Demo".into(),
            origin: "http://localhost:5173".into(),
            token: "secret-token".into(),
        })
        .await
        .expect("register app");

        let by_id = db
            .get_app_by_id("app-1")
            .await
            .expect("lookup by id")
            .expect("registration");
        assert_eq!(by_id.app_name, "Demo");

        let by_token = db
            .get_app_by_token("secret-token")
            .await
            .expect("lookup by token")
            .expect("registration");
        assert_eq!(by_token.app_id, "app-1");

        std::fs::remove_file(db_path).ok();
    }
}
