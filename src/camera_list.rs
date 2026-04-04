use anyhow::{Context, Result};
use nokhwa::query;
use nokhwa::utils::{ApiBackend, CameraIndex};

/// Cameras as `(index, human-readable name)` for the dashboard and for `--device`.
pub fn list_cameras() -> Result<Vec<(CameraIndex, String)>> {
    let infos = query(ApiBackend::Auto).context("query cameras (v4l2 / permissions?)")?;
    Ok(infos
        .into_iter()
        .map(|info| (info.index().clone(), info.human_name()))
        .collect())
}
