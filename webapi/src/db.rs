use std::{str::FromStr, sync::Arc};

use sqlx::{sqlite::SqliteConnectOptions, ConnectOptions, SqliteConnection};
use tokio::sync::{Mutex, MutexGuard};

use crate::config::RustmasConfig;

#[derive(Debug, Clone)]
pub struct SharedDbConnection {
    inner: Arc<Mutex<SqliteConnection>>,
}

impl SharedDbConnection {
    pub async fn from_config(config: &RustmasConfig) -> anyhow::Result<Self> {
        let mut conn = SqliteConnectOptions::from_str(&config.database_path.to_string_lossy())?
            .disable_statement_logging()
            .connect()
            .await?;

        sqlx::migrate!("../migrations").run(&mut conn).await?;

        Ok(Self {
            inner: Arc::new(Mutex::new(conn)),
        })
    }

    pub async fn lock(&self) -> MutexGuard<'_, SqliteConnection> {
        self.inner.lock().await
    }
}
