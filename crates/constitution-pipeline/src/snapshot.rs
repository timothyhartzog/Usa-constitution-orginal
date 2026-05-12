use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::paths::PipelinePaths;

/// A stable file inventory record for canonical corpus artifacts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnapshotRecord {
    pub path: String,
    pub bytes: u64,
    /// Stable FNV-1a 64-bit checksum. This is for change detection, not crypto.
    pub fnv1a64: String,
}

/// A rebuildable inventory of corpus files.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Snapshot {
    pub version: u32,
    pub root: String,
    pub records: Vec<SnapshotRecord>,
}

impl Snapshot {
    pub fn build(paths: &PipelinePaths) -> io::Result<Self> {
        let mut records = Vec::new();
        for input in paths.canonical_inputs() {
            if input.exists() {
                walk_dir(paths, input, &mut records)?;
            }
        }
        records.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(Self {
            version: 1,
            root: paths.root.display().to_string(),
            records,
        })
    }

    pub fn write_pretty(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let bytes = serde_json::to_vec_pretty(self)?;
        fs::write(path, bytes)
    }

    pub fn read(path: &Path) -> io::Result<Self> {
        let bytes = fs::read(path)?;
        serde_json::from_slice(&bytes).map_err(io::Error::other)
    }
}

fn walk_dir(
    paths: &PipelinePaths,
    dir: &Path,
    records: &mut Vec<SnapshotRecord>,
) -> io::Result<()> {
    let mut entries = fs::read_dir(dir)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            walk_dir(paths, &path, records)?;
        } else if file_type.is_file() {
            records.push(record_file(paths, &path)?);
        }
    }
    Ok(())
}

fn record_file(paths: &PipelinePaths, path: &Path) -> io::Result<SnapshotRecord> {
    let metadata = fs::metadata(path)?;
    let checksum = fnv1a64_file(path)?;
    Ok(SnapshotRecord {
        path: normalize_path(paths.display_from_root(path)),
        bytes: metadata.len(),
        fnv1a64: format!("{checksum:016x}"),
    })
}

fn normalize_path(path: &Path) -> String {
    path.components()
        .map(|part| part.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn fnv1a64_file(path: &Path) -> io::Result<u64> {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;

    let mut file = fs::File::open(PathBuf::from(path))?;
    let mut hash = OFFSET;
    let mut buf = [0_u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        for byte in &buf[..n] {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(PRIME);
        }
    }
    Ok(hash)
}
