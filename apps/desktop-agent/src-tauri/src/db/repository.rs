use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::Database;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaskRow {
    pub id: String,
    pub source_app_id: String,
    pub title: String,
    pub body: Option<String>,
    pub status: String,
    pub priority: i64,
    pub color: String,
    pub position_x: Option<i64>,
    pub position_y: Option<i64>,
    pub monitor_id: Option<String>,
    pub remind_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
    pub synced_at: Option<i64>,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTask {
    pub source_app_id: String,
    pub title: String,
    pub body: Option<String>,
    pub priority: i64,
    pub color: String,
    pub position_x: Option<i64>,
    pub position_y: Option<i64>,
    pub monitor_id: Option<String>,
    pub remind_at: Option<i64>,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskUpdate {
    pub title: Option<String>,
    pub body: Option<String>,
    pub status: Option<String>,
    pub priority: Option<i64>,
    pub color: Option<String>,
    pub remind_at: Option<i64>,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AppRegistration {
    pub app_id: String,
    pub app_name: String,
    pub origin: String,
    pub token: String,
    pub created_at: i64,
    pub last_seen_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewAppRegistration {
    pub app_id: String,
    pub app_name: String,
    pub origin: String,
    pub token: String,
}

impl Database {
    pub async fn create_task(&self, new_task: &NewTask) -> Result<TaskRow, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().timestamp();

        sqlx::query_as::<_, TaskRow>(
            r#"
            INSERT INTO tasks (
                id, source_app_id, title, body, status, priority, color,
                position_x, position_y, monitor_id, remind_at,
                created_at, updated_at, metadata
            ) VALUES (?, ?, ?, ?, 'active', ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(&id)
        .bind(&new_task.source_app_id)
        .bind(&new_task.title)
        .bind(&new_task.body)
        .bind(new_task.priority)
        .bind(&new_task.color)
        .bind(new_task.position_x)
        .bind(new_task.position_y)
        .bind(&new_task.monitor_id)
        .bind(new_task.remind_at)
        .bind(now)
        .bind(now)
        .bind(&new_task.metadata)
        .fetch_one(self.pool())
        .await
    }

    pub async fn get_task(&self, id: &str) -> Result<Option<TaskRow>, sqlx::Error> {
        sqlx::query_as::<_, TaskRow>("SELECT * FROM tasks WHERE id = ?")
            .bind(id)
            .fetch_optional(self.pool())
            .await
    }

    pub async fn list_active_tasks(&self) -> Result<Vec<TaskRow>, sqlx::Error> {
        sqlx::query_as::<_, TaskRow>(
            "SELECT * FROM tasks WHERE status = 'active' ORDER BY created_at ASC",
        )
        .fetch_all(self.pool())
        .await
    }

    pub async fn list_tasks_for_app(&self, app_id: &str) -> Result<Vec<TaskRow>, sqlx::Error> {
        sqlx::query_as::<_, TaskRow>(
            "SELECT * FROM tasks WHERE source_app_id = ? AND status = 'active' ORDER BY created_at ASC",
        )
        .bind(app_id)
        .fetch_all(self.pool())
        .await
    }

    pub async fn update_task(
        &self,
        id: &str,
        update: &TaskUpdate,
    ) -> Result<Option<TaskRow>, sqlx::Error> {
        let existing = match self.get_task(id).await? {
            Some(t) => t,
            None => return Ok(None),
        };

        let now = Utc::now().timestamp();
        let title = update.title.as_ref().unwrap_or(&existing.title);
        let body = update.body.as_ref().or(existing.body.as_ref());
        let status = update.status.as_ref().unwrap_or(&existing.status);
        let priority = update.priority.unwrap_or(existing.priority);
        let color = update.color.as_ref().unwrap_or(&existing.color);
        let remind_at = update.remind_at.or(existing.remind_at);
        let metadata = update.metadata.as_ref().or(existing.metadata.as_ref());

        sqlx::query_as::<_, TaskRow>(
            r#"
            UPDATE tasks SET
                title = ?, body = ?, status = ?, priority = ?, color = ?,
                remind_at = ?, metadata = ?, updated_at = ?
            WHERE id = ?
            RETURNING *
            "#,
        )
        .bind(title)
        .bind(body)
        .bind(status)
        .bind(priority)
        .bind(color)
        .bind(remind_at)
        .bind(metadata)
        .bind(now)
        .bind(id)
        .fetch_optional(self.pool())
        .await
    }

    pub async fn delete_task(&self, id: &str) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM tasks WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn update_position(
        &self,
        id: &str,
        x: i32,
        y: i32,
    ) -> Result<Option<TaskRow>, sqlx::Error> {
        let now = Utc::now().timestamp();
        sqlx::query_as::<_, TaskRow>(
            "UPDATE tasks SET position_x = ?, position_y = ?, updated_at = ? WHERE id = ? RETURNING *",
        )
        .bind(x as i64)
        .bind(y as i64)
        .bind(now)
        .bind(id)
        .fetch_optional(self.pool())
        .await
    }

    pub async fn register_app(&self, app: &NewAppRegistration) -> Result<(), sqlx::Error> {
        let now = Utc::now().timestamp();
        sqlx::query(
            r#"
            INSERT INTO app_registrations (app_id, app_name, origin, token, created_at, last_seen_at)
            VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(app_id) DO UPDATE SET
                app_name = excluded.app_name,
                origin = excluded.origin,
                token = excluded.token,
                last_seen_at = excluded.last_seen_at
            "#,
        )
        .bind(&app.app_id)
        .bind(&app.app_name)
        .bind(&app.origin)
        .bind(&app.token)
        .bind(now)
        .bind(now)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn get_app_by_id(&self, app_id: &str) -> Result<Option<AppRegistration>, sqlx::Error> {
        sqlx::query_as::<_, AppRegistration>("SELECT * FROM app_registrations WHERE app_id = ?")
            .bind(app_id)
            .fetch_optional(self.pool())
            .await
    }

    pub async fn get_app_by_token(&self, token: &str) -> Result<Option<AppRegistration>, sqlx::Error> {
        sqlx::query_as::<_, AppRegistration>("SELECT * FROM app_registrations WHERE token = ?")
            .bind(token)
            .fetch_optional(self.pool())
            .await
    }

    pub async fn list_registered_apps(&self) -> Result<Vec<AppRegistration>, sqlx::Error> {
        sqlx::query_as::<_, AppRegistration>(
            "SELECT * FROM app_registrations ORDER BY app_name ASC",
        )
        .fetch_all(self.pool())
        .await
    }

    pub async fn touch_app(&self, app_id: &str) -> Result<(), sqlx::Error> {
        let now = Utc::now().timestamp();
        sqlx::query("UPDATE app_registrations SET last_seen_at = ? WHERE app_id = ?")
            .bind(now)
            .bind(app_id)
            .execute(self.pool())
            .await?;
        Ok(())
    }

    pub async fn count_active_tasks(&self) -> Result<i64, sqlx::Error> {
        let row: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM tasks WHERE status = 'active'")
                .fetch_one(self.pool())
                .await?;
        Ok(row.0)
    }

    pub async fn tasks_due_for_reminder(&self, now: i64) -> Result<Vec<TaskRow>, sqlx::Error> {
        sqlx::query_as::<_, TaskRow>(
            "SELECT * FROM tasks WHERE status = 'active' AND remind_at IS NOT NULL AND remind_at <= ?",
        )
        .bind(now)
        .fetch_all(self.pool())
        .await
    }
}
