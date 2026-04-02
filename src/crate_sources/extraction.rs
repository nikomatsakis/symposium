//! Crate extraction and download

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use tar::Archive;

/// Handles extraction of .crate files to local cache
pub struct CrateExtractor;

impl CrateExtractor {
    pub fn new() -> Self {
        Self
    }

    /// Extract a cached .crate file to the extraction cache
    pub async fn extract_crate_to_cache(
        &self,
        crate_path: &Path,
        extraction_path: &Path,
    ) -> Result<PathBuf> {
        let file = fs::File::open(crate_path)
            .with_context(|| format!("failed to open {}", crate_path.display()))?;
        self.extract_from_reader(file, extraction_path)?;
        Ok(extraction_path.to_path_buf())
    }

    /// Download a crate from crates.io and extract it
    pub async fn download_and_extract_crate(
        &self,
        crate_name: &str,
        version: &str,
        extraction_path: &Path,
    ) -> Result<PathBuf> {
        let download_url =
            format!("https://static.crates.io/crates/{crate_name}/{crate_name}-{version}.crate",);

        let response = reqwest::get(&download_url)
            .await
            .with_context(|| format!("failed to download {crate_name} v{version}"))?;

        if !response.status().is_success() {
            anyhow::bail!(
                "failed to download {crate_name} v{version}: HTTP {}",
                response.status()
            );
        }

        let bytes = response.bytes().await?;
        self.extract_from_reader(std::io::Cursor::new(bytes), extraction_path)?;
        Ok(extraction_path.to_path_buf())
    }

    /// Extract from any reader to the specified directory
    fn extract_from_reader<R: Read>(&self, reader: R, extraction_path: &Path) -> Result<()> {
        fs::create_dir_all(extraction_path)?;

        let gz_decoder = GzDecoder::new(reader);
        let mut archive = Archive::new(gz_decoder);

        archive
            .unpack(extraction_path)
            .context("failed to extract crate archive")?;

        // Flatten: .crate archives contain a single top-level directory (name-version/)
        self.flatten_extraction(extraction_path)?;

        Ok(())
    }

    /// If the extraction contains a single top-level directory, move its contents up
    fn flatten_extraction(&self, extraction_path: &Path) -> Result<()> {
        let entries: Vec<_> =
            fs::read_dir(extraction_path)?.collect::<std::result::Result<Vec<_>, _>>()?;

        if entries.len() == 1 && entries[0].file_type()?.is_dir() {
            let inner_dir = entries[0].path();

            for inner_entry in fs::read_dir(&inner_dir)? {
                let inner_entry = inner_entry?;
                let src = inner_entry.path();
                let dst = extraction_path.join(inner_entry.file_name());

                if src.is_dir() {
                    self.move_dir(&src, &dst)?;
                } else {
                    fs::rename(&src, &dst)?;
                }
            }

            fs::remove_dir(&inner_dir)?;
        }

        Ok(())
    }

    /// Recursively move a directory
    fn move_dir(&self, src: &Path, dst: &Path) -> Result<()> {
        fs::create_dir_all(dst)?;

        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                self.move_dir(&src_path, &dst_path)?;
            } else {
                fs::rename(&src_path, &dst_path)?;
            }
        }

        fs::remove_dir(src)?;
        Ok(())
    }
}
