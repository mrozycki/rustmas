use std::{collections::HashMap, error::Error};

use sqlx::{Executor, Row};
use webapi_model::ParameterValue;

use crate::db::SharedDbConnection;

#[derive(Debug, Clone)]
pub struct Storage {
    conn: SharedDbConnection,
}

impl Storage {
    pub fn new(conn: SharedDbConnection) -> Self {
        Self { conn }
    }

    pub async fn save(
        &self,
        animation_id: &str,
        parameters: &HashMap<String, ParameterValue>,
    ) -> Result<(), Box<dyn Error>> {
        let query =
            sqlx::query("INSERT INTO animation_parameters(animation, parameters) VALUES (?, ?) ON CONFLICT(animation) DO UPDATE SET parameters=excluded.parameters;")
                .bind(animation_id)
                .bind(serde_json::to_string(parameters).unwrap());
        self.conn.lock().await.execute(query).await?;
        Ok(())
    }

    pub async fn fetch(
        &self,
        animation_id: &str,
    ) -> Result<Option<HashMap<String, ParameterValue>>, Box<dyn Error>> {
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
            .map(|s| serde_json::from_str::<HashMap<String, ParameterValue>>(&s))
            .transpose()?;

        Ok(result)
    }
}
