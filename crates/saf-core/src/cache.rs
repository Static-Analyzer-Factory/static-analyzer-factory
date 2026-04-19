//! Fingerprint-keyed disk cache for [`AirBundle`] results.
//!
//! Provides opt-in filesystem caching to avoid re-parsing/ingesting
//! the same input files. Consumers choose when to use it — it is
//! not wired into any frontend automatically.

use std::path::{Path, PathBuf};

use crate::air::AirBundle;

/// Simple filesystem cache for [`AirBundle`] results.
///
/// Stores and retrieves [`AirBundle`] instances keyed by a BLAKE3
/// fingerprint of the input. Cache entries are stored as JSON files
/// in the configured directory.
pub struct BundleCache {
    cache_dir: PathBuf,
}

impl BundleCache {
    /// Create a new cache backed by the given directory.
    ///
    /// The directory will be created on first write if it does not exist.
    #[must_use]
    pub fn new(cache_dir: impl Into<PathBuf>) -> Self {
        Self {
            cache_dir: cache_dir.into(),
        }
    }

    /// Try to load a cached bundle for the given fingerprint.
    ///
    /// Returns `None` if no cache entry exists or if deserialization fails.
    #[must_use]
    pub fn get(&self, fingerprint: &[u8]) -> Option<AirBundle> {
        let key = hex_encode(fingerprint);
        let path = self.cache_dir.join(format!("{key}.air.json"));
        let data = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&data).ok()
    }

    /// Store a bundle under the given fingerprint.
    ///
    /// Creates the cache directory if it does not exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the cache directory cannot be created or the
    /// bundle cannot be serialized/written.
    pub fn put(&self, fingerprint: &[u8], bundle: &AirBundle) -> Result<(), std::io::Error> {
        std::fs::create_dir_all(&self.cache_dir)?;
        let key = hex_encode(fingerprint);
        let path = self.cache_dir.join(format!("{key}.air.json"));
        let data = serde_json::to_string(bundle).map_err(std::io::Error::other)?;
        std::fs::write(path, data)
    }

    /// Remove a cached entry for the given fingerprint.
    ///
    /// Returns `true` if a file was removed, `false` if it did not exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be removed.
    pub fn remove(&self, fingerprint: &[u8]) -> Result<bool, std::io::Error> {
        let key = hex_encode(fingerprint);
        let path = self.cache_dir.join(format!("{key}.air.json"));
        if path.exists() {
            std::fs::remove_file(path)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get the cache directory path.
    #[must_use]
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }
}

/// Encode bytes as lowercase hex string (no `0x` prefix).
fn hex_encode(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut s, b| {
            let _ = write!(s, "{b:02x}");
            s
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::air::AirModule;
    use crate::ids::ModuleId;

    /// Helper to create a minimal `AirBundle` for tests.
    fn minimal_bundle() -> AirBundle {
        let module = AirModule::new(ModuleId::derive(b"cache_test"));
        AirBundle::new("test", module)
    }

    #[test]
    fn hex_encode_empty() {
        assert_eq!(hex_encode(&[]), "");
    }

    #[test]
    fn hex_encode_bytes() {
        assert_eq!(hex_encode(&[0xde, 0xad, 0xbe, 0xef]), "deadbeef");
    }

    #[test]
    fn cache_miss_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let cache = BundleCache::new(dir.path());
        assert!(cache.get(b"nonexistent").is_none());
    }

    #[test]
    fn cache_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let cache = BundleCache::new(dir.path());

        let bundle = minimal_bundle();
        let fingerprint = b"test_fingerprint_123";

        cache.put(fingerprint, &bundle).unwrap();
        let retrieved = cache.get(fingerprint).expect("should find cached bundle");
        assert_eq!(retrieved, bundle);
    }

    #[test]
    fn cache_remove() {
        let dir = tempfile::tempdir().unwrap();
        let cache = BundleCache::new(dir.path());

        let bundle = minimal_bundle();
        let fingerprint = b"removable";

        cache.put(fingerprint, &bundle).unwrap();
        assert!(cache.get(fingerprint).is_some());

        assert!(cache.remove(fingerprint).unwrap());
        assert!(cache.get(fingerprint).is_none());

        // Removing again returns false
        assert!(!cache.remove(fingerprint).unwrap());
    }

    #[test]
    fn cache_creates_directory() {
        let dir = tempfile::tempdir().unwrap();
        let cache_dir = dir.path().join("nested").join("cache");
        let cache = BundleCache::new(&cache_dir);

        let bundle = minimal_bundle();
        cache.put(b"test", &bundle).unwrap();

        assert!(cache_dir.exists());
    }
}
