//! Subprocess runner for invoking `saf run --bench-config`.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use saf_cli::bench_types::{BenchConfig, BenchResult};

/// Find the `saf` binary. Prefers the same target directory as our own binary.
fn find_saf_binary() -> Result<PathBuf> {
    // Look for saf in the same directory as this binary
    let self_path = std::env::current_exe()?;
    let dir = self_path.parent().unwrap_or(Path::new("."));
    let saf_path = dir.join("saf");
    if saf_path.exists() {
        return Ok(saf_path);
    }
    // Fall back to PATH
    Ok(PathBuf::from("saf"))
}

/// Run `saf run --bench-config` as a subprocess.
///
/// Writes `config` to `config_path`, invokes `saf`, reads result from
/// `result_path`. Returns the parsed `BenchResult`.
///
/// # Errors
///
/// Returns an error if config serialization, subprocess execution, or
/// result deserialization fails.
pub fn run_saf_bench(
    input: &Path,
    config: &BenchConfig,
    config_path: &Path,
    result_path: &Path,
) -> Result<BenchResult> {
    // Write bench config
    let config_json = serde_json::to_string_pretty(config)?;
    std::fs::write(config_path, &config_json)?;

    let saf = find_saf_binary()?;
    let output = Command::new(&saf)
        .arg("run")
        .arg(input)
        .arg("--bench-config")
        .arg(config_path)
        .arg("--output")
        .arg(result_path)
        .output()
        .with_context(|| format!("Failed to execute: {}", saf.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Process failed — return error result
        return Ok(BenchResult {
            success: false,
            error: Some(format!(
                "saf exited with status {}: {}",
                output.status,
                stderr.trim()
            )),
            ..Default::default()
        });
    }

    // Parse result
    let result_json = std::fs::read_to_string(result_path)
        .with_context(|| format!("Failed to read result: {}", result_path.display()))?;
    let result: BenchResult =
        serde_json::from_str(&result_json).with_context(|| "Failed to parse bench result JSON")?;

    Ok(result)
}
