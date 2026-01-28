//! BLAKE3-based deterministic ID generation for SAF entities.
//!
//! All AIR entities use `u128` IDs derived from BLAKE3 hashes, ensuring
//! deterministic identification across runs (FR-AIR-002).

/// Compute a deterministic `u128` ID from a domain tag and arbitrary data.
///
/// The domain tag prevents collisions between different entity kinds
/// that might share the same data bytes.
pub fn make_id(domain: &str, data: &[u8]) -> u128 {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    hasher.update(b":");
    hasher.update(data);
    let hash = hasher.finalize();
    let bytes: &[u8; 16] = hash.as_bytes()[..16].try_into().unwrap_or_else(|_| {
        // BLAKE3 always produces 32+ bytes; this branch is unreachable.
        unreachable!("BLAKE3 hash is always at least 16 bytes")
    });
    u128::from_le_bytes(*bytes)
}

/// Format a `u128` ID as a lowercase hex string with `0x` prefix.
pub fn id_to_hex(id: u128) -> String {
    format!("0x{id:032x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_id_is_deterministic() {
        let a = make_id("test", b"hello");
        let b = make_id("test", b"hello");
        assert_eq!(a, b);
    }

    #[test]
    fn different_domains_produce_different_ids() {
        let a = make_id("fn", b"main");
        let b = make_id("bb", b"main");
        assert_ne!(a, b);
    }

    #[test]
    fn id_to_hex_format() {
        let id = make_id("test", b"hello");
        let hex = id_to_hex(id);
        assert!(hex.starts_with("0x"));
        // 0x prefix + 32 hex chars = 34 total
        assert_eq!(hex.len(), 34);
    }
}
