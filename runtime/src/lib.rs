//! Loading and validation only. No dispatch, grants, source execution or network I/O.
mod entry;
mod json;
mod urls;

use serde_json::Value;
use std::{
    fmt,
    fs::{self, File},
    io::Read,
    path::Path,
    sync::LazyLock,
};

/// Host-facing categories, not portable Source API error codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadError {
    InvalidManifest,
    UnsupportedVersion,
    UnsupportedEngine,
    /// Reserved: no schema-valid declarative source requires host services.
    UnavailableService,
    InvalidEntry,
    ResourceLimit,
}
impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidManifest => "invalid manifest",
            Self::UnsupportedVersion => "unsupported specification version",
            Self::UnsupportedEngine => "unsupported engine",
            Self::UnavailableService => "unavailable required host service",
            Self::InvalidEntry => "invalid entry",
            Self::ResourceLimit => "load resource limit",
        })
    }
}
impl std::error::Error for LoadError {}
pub type LoadResult<T> = Result<T, LoadError>;

/// Finite host policy. Values may be lowered from the default hard ceilings.
#[derive(Debug, Clone, Copy)]
pub struct LoadLimits {
    pub manifest_bytes: usize,
    pub entry_bytes: usize,
    /// Combined manifest and entry input, not unrelated files in the directory.
    pub package_bytes: usize,
    pub json_depth: usize,
}
impl Default for LoadLimits {
    fn default() -> Self {
        Self {
            manifest_bytes: 64 * 1024,
            entry_bytes: 8 * 1024 * 1024,
            package_bytes: 8 * 1024 * 1024 + 64 * 1024,
            json_depth: 64,
        }
    }
}

/// Immutable validated data, not yet an invocable instance or permission grant.
/// Debug deliberately omits source data, which may contain credentials in headers.
pub struct LoadedSource {
    manifest: Value,
    entry: Value,
    origins: Vec<String>,
}
impl LoadedSource {
    pub fn manifest(&self) -> &Value {
        &self.manifest
    }
    pub fn entry(&self) -> &Value {
        &self.entry
    }
    pub fn requested_origins(&self) -> &[String] {
        &self.origins
    }
}
impl fmt::Debug for LoadedSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LoadedSource { validated: true }")
    }
}

static SCHEMA: LazyLock<jsonschema::Validator> = LazyLock::new(|| {
    let schema: Value =
        serde_json::from_str(include_str!("../../spec/schema/manifest.schema.json"))
            .expect("repository schema must be valid JSON");
    jsonschema::draft202012::new(&schema).expect("repository schema must compile")
});

fn read_contained(
    root: &Path,
    relative: &str,
    limit: usize,
    failure: LoadError,
) -> LoadResult<Vec<u8>> {
    let path = fs::canonicalize(root.join(relative)).map_err(|_| failure)?;
    // Path components, not a string prefix. The host must keep this tree stable
    // during loading; canonicalize/open is not a hostile-writer sandbox.
    if !path.starts_with(root) || !path.is_file() {
        return Err(failure);
    }
    let file = File::open(path).map_err(|_| failure)?;
    let metadata = file.metadata().map_err(|_| failure)?;
    if !metadata.is_file() {
        return Err(failure);
    }
    if metadata.len() > limit as u64 {
        return Err(LoadError::ResourceLimit);
    }
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| failure)?;
    if bytes.len() > limit {
        return Err(LoadError::ResourceLimit);
    }
    Ok(bytes)
}

/// Load from a host-controlled directory kept stable throughout this call.
/// See runtime/README.md for limits, diagnostics, filesystem and parser policy.
pub fn load(root: impl AsRef<Path>, limits: LoadLimits) -> LoadResult<LoadedSource> {
    let caps = LoadLimits::default();
    for (value, cap) in [
        (limits.manifest_bytes, caps.manifest_bytes),
        (limits.entry_bytes, caps.entry_bytes),
        (limits.package_bytes, caps.package_bytes),
        (limits.json_depth, caps.json_depth),
    ] {
        if value == 0 || value > cap {
            return Err(LoadError::ResourceLimit);
        }
    }
    let root = fs::canonicalize(root).map_err(|_| LoadError::InvalidManifest)?;
    if !root.is_dir() {
        return Err(LoadError::InvalidManifest);
    }
    let bytes = read_contained(
        &root,
        "manifest.json",
        limits.manifest_bytes.min(limits.package_bytes),
        LoadError::InvalidManifest,
    )?;
    let manifest = json::parse(&bytes, LoadError::InvalidManifest, limits.json_depth)?;
    if let Some(version) = manifest.get("specVersion").and_then(Value::as_str)
        && version != "0.1"
    {
        return Err(LoadError::UnsupportedVersion);
    }
    if let Some(engine) = manifest.get("engine").and_then(Value::as_str)
        && engine != "declarative"
    {
        return Err(LoadError::UnsupportedEngine);
    }
    if !SCHEMA.is_valid(&manifest) {
        return Err(LoadError::InvalidManifest);
    }
    let origins = urls::origins(manifest["permissions"]["network"].as_array().unwrap())
        .map_err(|_| LoadError::InvalidManifest)?;
    // The authoritative schema establishes these types and an empty host list.
    let entry_path = manifest["entry"].as_str().unwrap();
    let operations = manifest["capabilities"]["operations"].as_array().unwrap();
    let input = read_contained(
        &root,
        entry_path,
        limits.entry_bytes.min(limits.package_bytes - bytes.len()),
        LoadError::InvalidEntry,
    )?;
    let entry = json::parse(&input, LoadError::InvalidEntry, limits.json_depth)?;
    entry::validate(&entry, operations).map_err(|_| LoadError::InvalidEntry)?;
    Ok(LoadedSource {
        manifest,
        entry,
        origins,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authoritative_schema_and_all_manifest_fixtures_agree() {
        let schema: Value =
            serde_json::from_str(include_str!("../../spec/schema/manifest.schema.json")).unwrap();
        jsonschema::meta::validate(&schema).unwrap();
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../conformance");
        for (directory, valid, count) in [("valid", true, 7), ("invalid", false, 34)] {
            let mut checked = 0;
            for file in fs::read_dir(root.join(directory).join("manifests")).unwrap() {
                let path = file.unwrap().path();
                if path.extension().and_then(|v| v.to_str()) != Some("json") {
                    continue;
                }
                let value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                assert_eq!(SCHEMA.is_valid(&value), valid, "{}", path.display());
                checked += 1;
            }
            assert_eq!(checked, count);
        }
    }
}
