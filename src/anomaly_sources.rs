use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// Finds source anomaly documents in one directory in stable filename order.
pub fn anomaly_sources(directory: &Path) -> io::Result<Vec<PathBuf>> {
    let entries = fs::read_dir(directory)?.collect::<io::Result<Vec<_>>>()?;
    let mut sources = Vec::new();

    for entry in entries {
        if !entry.file_type()?.is_file() {
            continue;
        }

        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with("AN-") && name.ends_with(".md") {
            sources.push(entry.path());
        }
    }

    sources.sort_by(|left, right| left.file_name().cmp(&right.file_name()));
    Ok(sources)
}
