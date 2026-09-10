// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2025 Fezzik the Giant

//! Persisted playback state (queue, track, position, paused) across restarts.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::api::models::Track;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackSession {
    pub queue: Vec<Track>,
    pub queue_index: usize,
    pub position: f64,
    pub paused: bool,
    #[serde(default)]
    pub source_playlist_uuid: Option<String>,
}

impl PlaybackSession {
    pub fn path() -> PathBuf {
        crate::api::auth::config_path().with_file_name("session.json")
    }

    pub fn load() -> Option<Self> {
        let data = std::fs::read_to_string(Self::path()).ok()?;
        serde_json::from_str(&data).ok()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn clear() {
        let _ = std::fs::remove_file(Self::path());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::test_support::track;

    #[test]
    fn session_roundtrip() {
        let session = PlaybackSession {
            queue: vec![track(10), track(20)],
            queue_index: 1,
            position: 45.5,
            paused: true,
            source_playlist_uuid: Some("uuid-123".to_string()),
        };

        let json = serde_json::to_string(&session).unwrap();
        let loaded: PlaybackSession = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.queue.len(), 2);
        assert_eq!(loaded.queue_index, 1);
        assert_eq!(loaded.position, 45.5);
        assert!(loaded.paused);
        assert_eq!(loaded.source_playlist_uuid.as_deref(), Some("uuid-123"));
    }

    #[test]
    fn session_corrupted_json_fails() {
        assert!(serde_json::from_str::<PlaybackSession>("{not valid json").is_err());
    }
}
