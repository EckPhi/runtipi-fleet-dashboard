use crate::model::CachedInventory;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct CacheStore {
    root: PathBuf,
}

impl CacheStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub async fn load_all(&self) -> Result<Vec<CachedInventory>> {
        tokio::fs::create_dir_all(&self.root).await?;
        let mut entries = tokio::fs::read_dir(&self.root).await?;
        let mut out = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            if entry.path().extension().and_then(|x| x.to_str()) != Some("json") {
                continue;
            }
            let bytes = tokio::fs::read(entry.path()).await?;
            let cached = serde_json::from_slice(&bytes)
                .with_context(|| format!("parse cache {}", entry.path().display()))?;
            out.push(cached);
        }
        Ok(out)
    }

    pub async fn save(&self, value: &CachedInventory) -> Result<()> {
        tokio::fs::create_dir_all(&self.root).await?;
        let safe_id: String = value
            .inventory
            .server
            .id
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        anyhow::ensure!(!safe_id.is_empty(), "unsafe empty server id");
        atomic_write(
            &self.root.join(format!("{safe_id}.json")),
            &serde_json::to_vec_pretty(value)?,
        )
        .await
    }
}

pub async fn write_if_changed(path: &Path, bytes: &[u8]) -> Result<bool> {
    if matches!(tokio::fs::read(path).await, Ok(ref current) if current.as_slice() == bytes) {
        return Ok(false);
    }
    atomic_write(path, bytes).await?;
    Ok(true)
}

async fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().context("output path has no parent")?;
    tokio::fs::create_dir_all(parent).await?;
    let file_name = path
        .file_name()
        .and_then(|x| x.to_str())
        .context("invalid output filename")?;
    let tmp = parent.join(format!(".{file_name}.tmp"));
    tokio::fs::write(&tmp, bytes)
        .await
        .with_context(|| format!("write temporary file {}", tmp.display()))?;
    tokio::fs::rename(&tmp, path)
        .await
        .with_context(|| format!("replace {}", path.display()))?;
    Ok(())
}
