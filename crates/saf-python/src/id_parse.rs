//! Centralized ID parsing utilities for Python bindings.
//!
//! This module consolidates all hex string → ID type conversions used
//! across the Python bindings, eliminating code duplication.

use pyo3::prelude::*;
use saf_core::ids::{BlockId, FunctionId, InstId, LocId, ValueId};

/// Parse a hex string (with optional `0x` prefix) into a `u128`.
pub(crate) fn parse_hex(s: &str) -> PyResult<u128> {
    let hex = s.strip_prefix("0x").unwrap_or(s);
    u128::from_str_radix(hex, 16)
        .map_err(|_| pyo3::exceptions::PyValueError::new_err(format!("Invalid hex ID: {s}")))
}

/// Parse a hex string into a `BlockId`.
pub(crate) fn parse_block_id(s: &str) -> PyResult<BlockId> {
    Ok(BlockId::new(parse_hex(s)?))
}

/// Parse a hex string into a `FunctionId`.
pub(crate) fn parse_function_id(s: &str) -> PyResult<FunctionId> {
    Ok(FunctionId::new(parse_hex(s)?))
}

/// Parse a hex string into a `ValueId`.
pub(crate) fn parse_value_id(s: &str) -> PyResult<ValueId> {
    Ok(ValueId::new(parse_hex(s)?))
}

/// Parse a hex string into an `InstId`.
pub(crate) fn parse_inst_id(s: &str) -> PyResult<InstId> {
    Ok(InstId::new(parse_hex(s)?))
}

/// Parse a hex string into a `LocId`.
pub(crate) fn parse_loc_id(s: &str) -> PyResult<LocId> {
    Ok(LocId::new(parse_hex(s)?))
}

/// Parse a hex string into a `MemAccessId`.
pub(crate) fn parse_mem_access_id(s: &str) -> PyResult<saf_analysis::mssa::MemAccessId> {
    Ok(saf_analysis::mssa::MemAccessId::new(parse_hex(s)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_with_prefix() {
        assert_eq!(parse_hex("0x1234").unwrap(), 0x1234);
    }

    #[test]
    fn test_parse_hex_without_prefix() {
        assert_eq!(parse_hex("abcd").unwrap(), 0xabcd);
    }

    #[test]
    fn test_parse_value_id() {
        let id = parse_value_id("0x123456789abcdef0").unwrap();
        assert_eq!(id.raw(), 0x123456789abcdef0);
    }

    #[test]
    fn test_parse_block_id() {
        let id = parse_block_id("0xdeadbeef").unwrap();
        assert_eq!(id.raw(), 0xdeadbeef);
    }

    #[test]
    fn test_parse_function_id() {
        let id = parse_function_id("0xcafebabe").unwrap();
        assert_eq!(id.raw(), 0xcafebabe);
    }

    #[test]
    fn test_parse_inst_id() {
        let id = parse_inst_id("0xfeedface").unwrap();
        assert_eq!(id.raw(), 0xfeedface);
    }

    #[test]
    fn test_parse_loc_id() {
        let id = parse_loc_id("0xbaadf00d").unwrap();
        assert_eq!(id.raw(), 0xbaadf00d);
    }
}
