use std::{path::PathBuf, str::FromStr};

use animation_wrapper::config::{PluginConfig, PluginManifest};
use anyhow::anyhow;
use log::warn;

use crate::db::SharedDbConnection;

pub struct DbAnimation {
    pub animation_id: String,
    pub path: PathBuf,
    pub manifest: PluginManifest,
}

#[derive(Debug, Clone)]
pub struct Storage {
    conn: SharedDbConnection,
}

impl Storage {
    pub fn new(conn: SharedDbConnection) -> Self {
        Self { conn }
    }

    pub async fn install(&self, config: &PluginConfig) -> anyhow::Result<()> {
        let path = config.path.to_string_lossy();
        let manifest = serde_json::to_string(&config.manifest)?;
        sqlx::query!(
            "INSERT INTO animation_plugins(id, path, manifest) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
            config.animation_id,
            path,
            manifest
        )
        .execute(&mut *self.conn.lock().await)
        .await?;

        Ok(())
    }

    pub async fn fetch_by_id(&self, animation_id: &str) -> anyhow::Result<Option<DbAnimation>> {
        sqlx::query!(
            "SELECT id, path, manifest FROM animation_plugins WHERE id = $1",
            animation_id
        )
        .fetch_optional(&mut *self.conn.lock().await)
        .await?
        .map(|r| {
            let manifest = serde_json::from_slice(&r.manifest)
                .map_err(|_| anyhow!("Invalid manifest for animation with id {}", r.id))?;
            Ok(DbAnimation {
                animation_id: r.id,
                path: PathBuf::from_str(&r.path).unwrap(), // Infallible, safe to unwrap
                manifest,
            })
        })
        .transpose()
    }

    pub async fn fetch_all(&self) -> anyhow::Result<Vec<DbAnimation>> {
        let animations = sqlx::query!("SELECT id, path, manifest FROM animation_plugins")
            .fetch_all(&mut *self.conn.lock().await)
            .await?
            .into_iter()
            .filter_map(|r| {
                let manifest = serde_json::from_slice(&r.manifest)
                    .inspect_err(|e| {
                        warn!(
                            "Invalid manifest for animation with id {}, skipping: {e}",
                            r.id
                        )
                    })
                    .ok()?;
                Some(DbAnimation {
                    animation_id: r.id,
                    path: PathBuf::from_str(&r.path).unwrap(), // Infallible, safe to unwrap
                    manifest,
                })
            })
            .collect();

        Ok(animations)
    }

    pub async fn delete(&self, animation_id: &str) -> anyhow::Result<String> {
        let path = sqlx::query!(
            "DELETE FROM animation_plugins WHERE id = $1 RETURNING path",
            animation_id
        )
        .fetch_one(&mut *self.conn.lock().await)
        .await?
        .path;

        Ok(path)
    }
}
