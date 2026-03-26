use std::path::PathBuf;

use super::track::Track;

/// Audio file management. Mirrors src/track/audio.{h,cpp}.
pub struct Audio<'a> {
    track: &'a Track,
    manemix_dir: String,
}

#[derive(Debug, serde::Serialize)]
pub struct AudioStatus {
    pub ready: bool,
    pub has_status: bool,
    pub status: String,
    pub is_mp3_source: bool,
    pub is_other_source: bool,
    pub extension: String,
}

impl<'a> Audio<'a> {
    pub fn new(track: &'a Track, manemix_dir: &str) -> Self {
        Audio { track, manemix_dir: manemix_dir.into() }
    }

    fn base(&self) -> PathBuf {
        PathBuf::from(format!("{}/tracks", self.manemix_dir))
    }

    pub fn mp3_path(&self) -> PathBuf { self.base().join(format!("{}.mp3", self.track.id)) }
    pub fn vorbis_path(&self) -> PathBuf { self.base().join(format!("{}.ogg", self.track.id)) }
    pub fn aac_path(&self) -> PathBuf { self.base().join(format!("{}.m4a", self.track.id)) }
    pub fn opus_path(&self) -> PathBuf { self.base().join(format!("{}.opus", self.track.id)) }

    pub fn original_path(&self) -> Option<PathBuf> {
        let prefix = format!("{}.orig.", self.track.id);
        let dir = self.base();
        std::fs::read_dir(&dir).ok()?
            .filter_map(|e| e.ok())
            .find(|e| e.file_name().to_string_lossy().starts_with(&prefix))
            .map(|e| e.path())
    }

    /// Build the status context, mirroring Audio::fill(Dict*).
    pub fn status(&self) -> AudioStatus {
        if let Some(orig) = self.original_path() {
            let ext = orig.extension()
                .map(|e| format!(".{}", e.to_string_lossy()))
                .unwrap_or_default();
            AudioStatus {
                ready: true,
                has_status: false,
                status: String::new(),
                is_mp3_source: ext == ".mp3",
                is_other_source: ext != ".mp3",
                extension: ext,
            }
        } else {
            let status = if self.mp3_path().exists() {
                "Transcoding into MP3..."
            } else if self.opus_path().exists() {
                "Transcoding into Opus..."
            } else if self.aac_path().exists() {
                "Transcoding into AAC..."
            } else if self.vorbis_path().exists() {
                "Transcoding into Vorbis..."
            } else {
                "Couldn't transcode."
            };
            AudioStatus {
                ready: false,
                has_status: true,
                status: status.into(),
                is_mp3_source: false,
                is_other_source: false,
                extension: String::new(),
            }
        }
    }

    /// Update ID3/Vorbis/MP4 tags using lofty.
    pub fn update_tags(&self) {
        use lofty::prelude::*;
        use lofty::config::WriteOptions;
        for path in [self.mp3_path(), self.vorbis_path(), self.aac_path(), self.opus_path()] {
            if !path.exists() { continue; }
            if let Ok(mut tagged) = lofty::read_from_path(&path) {
                if let Some(tag) = tagged.primary_tag_mut() {
                    tag.set_title(self.track.title.clone());
                    tag.set_artist(self.track.artist.name.clone());
                    let _ = tagged.save_to_path(&path, WriteOptions::default());
                }
            }
        }
    }

    /// Remove all audio files for this track (transcoded + original).
    pub fn unlink(&self) {
        let _ = std::fs::remove_file(self.mp3_path());
        let _ = std::fs::remove_file(self.vorbis_path());
        let _ = std::fs::remove_file(self.aac_path());
        let _ = std::fs::remove_file(self.opus_path());
        if let Some(orig) = self.original_path() {
            let _ = std::fs::remove_file(orig);
        }
    }
}
