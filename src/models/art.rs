use std::path::{Path, PathBuf};

/// Cover art management. Mirrors src/track/art.{h,cpp}.
pub struct Art {
    tid: i32,
    manemix_dir: String,
}

impl Art {
    pub fn new(manemix_dir: &str, tid: i32) -> Option<Self> {
        let art = Art { tid, manemix_dir: manemix_dir.into() };
        if art.full_path().exists() { Some(art) } else { None }
    }

    pub fn exists(manemix_dir: &str, tid: i32) -> bool {
        Path::new(&format!("{}/art/{}", manemix_dir, tid)).exists()
    }

    pub fn full_path(&self) -> PathBuf {
        PathBuf::from(format!("{}/art/{}", self.manemix_dir, self.tid))
    }

    pub fn medium_path(&self) -> PathBuf {
        PathBuf::from(format!("{}/art/medium/{}.jpg", self.manemix_dir, self.tid))
    }

    pub fn thumb_path(&self) -> PathBuf {
        PathBuf::from(format!("{}/art/thumb/{}.png", self.manemix_dir, self.tid))
    }

    /// Generate medium and thumbnail images using the `image` crate.
    pub fn make_thumbs(&self) -> Result<(), image::ImageError> {
        let img = image::open(self.full_path())?;
        let (w, h) = (img.width(), img.height());

        // Medium: scale down if > 480px tall and not a GIF
        if h > 480 {
            let medium = if w > 1000 {
                img.resize(1000, (1000.0 / w as f64 * h as f64) as u32, image::imageops::FilterType::Lanczos3)
            } else {
                img.clone()
            };
            medium.save(self.medium_path())?;
        }

        // Thumbnail: 64px tall
        if h > 64 {
            let aspect = w as f64 / h as f64;
            let thumb = img.resize((64.0 * aspect) as u32, 64, image::imageops::FilterType::Lanczos3);
            thumb.save(self.thumb_path())?;
        }

        Ok(())
    }

    /// MIME type of the full-size art based on magic bytes.
    pub fn mime(&self) -> &'static str {
        detect_format(&self.full_path())
    }
}

fn detect_format(path: &Path) -> &'static str {
    let bytes = std::fs::read(path).unwrap_or_default();
    match bytes.get(0..4) {
        Some([0xff, 0xd8, ..]) => "image/jpeg",
        Some([0x89, b'P', b'N', b'G']) => "image/png",
        Some([b'G', b'I', b'F', b'8']) => "image/gif",
        _ => "application/octet-stream",
    }
}
