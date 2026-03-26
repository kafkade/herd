use anyhow::{Context, Result};
use chrono::Utc;
use log::debug;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::monitor::Rect;
use crate::window::WindowInfo;

const MAX_SNAPSHOTS: usize = 10;

#[derive(Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub version: u32,
    pub timestamp: String,
    pub target_display: u32,
    pub windows: Vec<WindowSnapshot>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowSnapshot {
    pub hwnd: isize,
    pub title: String,
    pub rect: Rect,
    pub monitor_index: u32,
}

fn snapshots_dir() -> Result<PathBuf> {
    let local_app_data = dirs::data_local_dir().context("Could not find local app data dir")?;
    let dir = local_app_data.join("herd").join("snapshots");
    fs::create_dir_all(&dir).context("Could not create snapshots directory")?;
    Ok(dir)
}

pub fn save_snapshot(
    windows: &[WindowInfo],
    monitor_indices: &[u32],
    target_display: u32,
) -> Result<PathBuf> {
    let dir = snapshots_dir()?;

    let snapshot = Snapshot {
        version: 1,
        timestamp: Utc::now().to_rfc3339(),
        target_display,
        windows: windows
            .iter()
            .zip(monitor_indices.iter())
            .map(|(w, &mon_idx)| WindowSnapshot {
                hwnd: w.hwnd,
                title: w.title.clone(),
                rect: w.rect,
                monitor_index: mon_idx,
            })
            .collect(),
    };

    let filename = format!("{}.json", Utc::now().timestamp_millis());
    let path = dir.join(&filename);

    let json = serde_json::to_string_pretty(&snapshot)?;
    fs::write(&path, json)?;

    debug!("Saved snapshot to {}", path.display());

    prune_snapshots(&dir)?;

    Ok(path)
}

pub fn load_latest_snapshot() -> Result<Option<(Snapshot, PathBuf)>> {
    let dir = match snapshots_dir() {
        Ok(d) => d,
        Err(_) => return Ok(None),
    };

    let mut entries: Vec<_> = fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == "json")
        })
        .collect();

    entries.sort_by_key(|e| e.file_name());

    if let Some(latest) = entries.last() {
        let content = fs::read_to_string(latest.path())?;
        let snapshot: Snapshot = serde_json::from_str(&content)?;
        debug!("Loaded snapshot from {}", latest.path().display());

        Ok(Some((snapshot, latest.path())))
    } else {
        Ok(None)
    }
}

/// Remove a consumed snapshot file after successful restore.
pub fn remove_snapshot(path: &PathBuf) -> Result<()> {
    fs::remove_file(path).context("Failed to remove snapshot file")?;
    debug!("Removed snapshot: {}", path.display());
    Ok(())
}

fn prune_snapshots(dir: &PathBuf) -> Result<()> {
    let mut entries: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .is_some_and(|ext| ext == "json")
        })
        .collect();

    entries.sort_by_key(|e| e.file_name());

    while entries.len() > MAX_SNAPSHOTS {
        if let Some(oldest) = entries.first() {
            debug!("Pruning old snapshot: {}", oldest.path().display());
            fs::remove_file(oldest.path())?;
            entries.remove(0);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_serialization() {
        let snapshot = Snapshot {
            version: 1,
            timestamp: "2026-03-25T16:30:00Z".to_string(),
            target_display: 1,
            windows: vec![WindowSnapshot {
                hwnd: 12345,
                title: "Test Window".to_string(),
                rect: Rect {
                    left: 100,
                    top: 50,
                    right: 900,
                    bottom: 650,
                },
                monitor_index: 2,
            }],
        };

        let json = serde_json::to_string_pretty(&snapshot).unwrap();
        let deserialized: Snapshot = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.version, 1);
        assert_eq!(deserialized.windows.len(), 1);
        assert_eq!(deserialized.windows[0].title, "Test Window");
        assert_eq!(deserialized.windows[0].rect.left, 100);
    }
}
