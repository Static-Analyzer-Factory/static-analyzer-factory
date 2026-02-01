//! Algorithm trace generator for SAF interactive tutorials.
//!
//! Generates pre-computed JSON traces that drive the algorithm animation
//! stepper in the tutorials application.

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use serde::Deserialize;

/// Generate algorithm animation traces from LLVM IR files.
#[derive(Parser)]
#[command(
    name = "saf-trace",
    about = "Algorithm trace generator for tutorial animations"
)]
struct Cli {
    /// Algorithm to trace: pta, ifds, absint, mssa
    #[arg(short, long)]
    algorithm: Option<String>,

    /// Input LLVM IR file (.ll or .bc)
    #[arg(short, long)]
    input: Option<PathBuf>,

    /// Output JSON file path
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Generate all traces from traces.toml manifest
    #[arg(long)]
    all: bool,

    /// Output directory for --all mode
    #[arg(long, default_value = "tutorials/public/content/algorithms")]
    output_dir: PathBuf,

    /// Validate existing trace files against current analysis output
    #[arg(long)]
    validate: bool,
}

/// A single trace entry from the manifest.
#[derive(Deserialize)]
struct TraceEntry {
    algorithm: String,
    input: PathBuf,
    output: PathBuf,
    title: String,
}

/// The traces manifest format.
#[derive(Deserialize)]
struct TracesManifest {
    trace: Vec<TraceEntry>,
}

fn main() -> Result<()> {
    // Initialize tracing (same pattern as saf-bench)
    use tracing_subscriber::prelude::*;
    let saf_layer = saf_core::logging::subscriber::init();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_filter(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
                ),
        )
        .with(saf_layer)
        .init();

    let cli = Cli::parse();

    if cli.all {
        run_all(&cli)?;
    } else if cli.validate {
        tracing::info!("Validation mode not yet implemented");
    } else if let Some(ref algorithm) = cli.algorithm {
        run_single(algorithm, &cli)?;
    } else {
        anyhow::bail!("Specify --algorithm or --all. Run with --help for usage.");
    }

    Ok(())
}

fn run_all(cli: &Cli) -> Result<()> {
    let manifest_path = PathBuf::from("crates/saf-trace/traces.toml");
    let manifest_str = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("Failed to read manifest: {}", manifest_path.display()))?;
    let manifest: TracesManifest =
        toml::from_str(&manifest_str).context("Failed to parse traces.toml")?;

    tracing::info!("Found {} trace entries in manifest", manifest.trace.len());

    for entry in &manifest.trace {
        let output_path = cli.output_dir.join(&entry.output);
        tracing::info!(
            algorithm = %entry.algorithm,
            input = %entry.input.display(),
            output = %output_path.display(),
            "Generating trace: {}",
            entry.title
        );
        run_single(
            &entry.algorithm,
            &Cli {
                algorithm: Some(entry.algorithm.clone()),
                input: Some(entry.input.clone()),
                output: Some(output_path),
                all: false,
                output_dir: cli.output_dir.clone(),
                validate: false,
            },
        )?;
    }

    Ok(())
}

fn run_single(algorithm: &str, cli: &Cli) -> Result<()> {
    let input = cli
        .input
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("--input is required when using --algorithm"))?;

    let output = cli
        .output
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("--output is required when using --algorithm"))?;

    match algorithm {
        "pta" => {
            tracing::info!("PTA trace generation not yet implemented");
            tracing::info!("  input: {}", input.display());
            tracing::info!("  output: {}", output.display());
        }
        "ifds" => {
            tracing::info!("IFDS trace generation not yet implemented");
        }
        "absint" => {
            tracing::info!("Abstract interpretation trace generation not yet implemented");
        }
        "mssa" => {
            tracing::info!("Memory SSA trace generation not yet implemented");
        }
        other => {
            anyhow::bail!("Unknown algorithm '{other}'. Valid options: pta, ifds, absint, mssa");
        }
    }

    Ok(())
}
