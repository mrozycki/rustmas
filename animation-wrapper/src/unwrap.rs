use std::{
    fs::File,
    io::{BufReader, Read, Seek},
    path::Path,
};

use tar::Archive;

use crate::config::{PluginConfig, PluginManifest};

#[derive(Debug, thiserror::Error)]
pub enum PluginUnwrapError {
    #[error("Cannot open plugin file: {0}")]
    CannotOpenFile(#[from] std::io::Error),

    #[error("CRAB is missing manifest.json entry")]
    MissingManifest,

    #[error("Invalid manifest: {0}")]
    InvalidManifest(#[from] serde_json::error::Error),

    #[error("CRAB is missing plugin.wasm entry")]
    MissingWasm,

    #[error("Invalid CRAB file name")]
    InvalidFilename,
}

fn manifest_from_crab(path: &Path) -> Result<PluginManifest, PluginUnwrapError> {
    let reader = BufReader::new(File::open(path)?);
    let mut archive = Archive::new(reader);
    let mut entries = archive.entries_with_seek()?;

    let entry_reader = entries
        .find(|e| {
            e.as_ref().is_ok_and(|e| {
                e.path()
                    .is_ok_and(|p| p.to_str().is_some_and(|p| p == "manifest.json"))
            })
        })
        .ok_or(PluginUnwrapError::MissingManifest)??;

    Ok(serde_json::from_reader(entry_reader)?)
}

fn animation_id_from_crab(path: &Path) -> Result<String, PluginUnwrapError> {
    Ok(path
        .file_name()
        .ok_or(PluginUnwrapError::InvalidFilename)?
        .to_string_lossy()
        .trim_end_matches(".crab")
        .to_string())
}

pub fn unwrap_plugin<P: AsRef<Path>>(path: &P) -> Result<PluginConfig, PluginUnwrapError> {
    fn inner(path: &Path) -> Result<PluginConfig, PluginUnwrapError> {
        let animation_id = animation_id_from_crab(path)?;
        let manifest = manifest_from_crab(path)?;
        let path = path.to_owned();

        Ok(PluginConfig {
            animation_id,
            manifest,
            path,
        })
    }
    inner(path.as_ref())
}

pub fn reader_from_crab<P: AsRef<Path>>(path: &P) -> Result<impl Read, PluginUnwrapError> {
    let (start, size) = {
        let reader = BufReader::new(File::open(path)?);
        let mut archive = Archive::new(reader);
        let mut entries = archive.entries_with_seek()?;

        let wasm_entry = entries
            .find(|e| {
                e.as_ref().is_ok_and(|e| {
                    e.path()
                        .is_ok_and(|p| p.to_str().is_some_and(|p| p == "plugin.wasm"))
                })
            })
            .ok_or(PluginUnwrapError::MissingWasm)??;

        (wasm_entry.raw_file_position(), wasm_entry.size())
    };

    let mut reader = BufReader::new(File::open(path)?);
    reader.seek(std::io::SeekFrom::Start(start))?;

    Ok(LimitedReader::new(reader, size as usize))
}

pub struct LimitedReader<R>
where
    R: Read,
{
    inner: R,
    limit: usize,
    position: usize,
}

impl<R> LimitedReader<R>
where
    R: Read,
{
    fn new(reader: R, limit: usize) -> Self {
        Self {
            inner: reader,
            limit,
            position: 0,
        }
    }
}

impl<R> Read for LimitedReader<R>
where
    R: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let left = self.limit - self.position;
        let to_read = left.min(buf.len());
        let buf = &mut buf[0..to_read];
        let read = self.inner.read(buf)?;
        self.position += read;
        Ok(read)
    }
}
