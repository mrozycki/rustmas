use std::{collections::HashMap, error::Error, str::FromStr, sync::Arc};

use sqlx::{sqlite::SqliteConnectOptions, ConnectOptions, Executor, Row, SqliteConnection};
use tokio::sync::Mutex;
use webapi_model::ParameterValue;

pub struct Db {
    conn: Arc<Mutex<SqliteConnection>>,
}

impl Db {
    pub async fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        let mut conn = SqliteConnectOptions::from_str(path)?
            .disable_statement_logging()
            .connect()
            .await?;

        sqlx::migrate!("../migrations").run(&mut conn).await?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub async fn set_parameters(
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

    pub async fn get_parameters(
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
