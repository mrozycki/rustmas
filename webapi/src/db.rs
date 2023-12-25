use std::{error::Error, str::FromStr, sync::Arc};

use sqlx::{sqlite::SqliteConnectOptions, ConnectOptions, Executor, Row, SqliteConnection};
use tokio::sync::Mutex;

pub struct Db {
    conn: Arc<Mutex<SqliteConnection>>,
}

impl Db {
    pub async fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        let conn = SqliteConnectOptions::from_str(path)?
            .disable_statement_logging()
            .connect()
            .await?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub async fn set_parameters(
        &self,
        animation_id: &str,
        parameters: &serde_json::Value,
    ) -> Result<(), Box<dyn Error>> {
        let query =
            sqlx::query("INSERT INTO animation_parameters(animation, parameters) VALUES (?, ?) ON CONFLICT(animation) DO UPDATE SET parameters=excluded.parameters;")
                .bind(animation_id)
                .bind(parameters.to_string());
        self.conn.lock().await.execute(query).await?;
        Ok(())
    }

    pub async fn get_parameters(
        &self,
        animation_id: &str,
    ) -> Result<Option<serde_json::Value>, Box<dyn Error>> {
        let query = sqlx::query("SELECT parameters FROM animation_parameters WHERE animation = ?;")
            .bind(animation_id);

        let result = self
            .conn
            .lock()
            .await
            .fetch_optional(query)
            .await?
            .map(|row| row.try_get::<String, &str>("parameters"))
            .transpose()?
            .map(|s| serde_json::from_str::<serde_json::Value>(&s))
            .transpose()?;

        Ok(result)
    }
}
