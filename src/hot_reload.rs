use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use anyhow::Result;
use tokio::sync::Mutex;

use crate::structure::HarFile;

pub async fn run(path: PathBuf, current_state: Arc<Mutex<HarFile>>) -> Result<()> {
    let mut last_modified = fetch_last_modified(&path).await?;
    tracing::info!(
        "Starting the hot-reloading, current last modified is {:?}",
        last_modified
    );

    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;

        let modified = fetch_last_modified(&path).await?;

        if modified > last_modified {
            last_modified = modified;

            reload_file(&path, &current_state).await?;
        }
    }
}

async fn fetch_last_modified(path: &Path) -> Result<SystemTime> {
    let metadata = tokio::fs::metadata(path).await?;
    let modified = metadata.modified()?;

    Ok(modified)
}

async fn reload_file(path: &Path, current_state: &Arc<Mutex<HarFile>>) -> Result<()> {
    tracing::info!("{:?} has changed, reloading the state", path);

    let raw = tokio::fs::read_to_string(path).await?;
    let harfile: HarFile = serde_json::from_str(&raw)?;

    *current_state.lock().await = harfile;

    Ok(())
}
