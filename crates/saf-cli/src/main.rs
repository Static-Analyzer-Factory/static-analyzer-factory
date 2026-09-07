mod commands;
mod driver;
mod help;

use clap::Parser;
use commands::{Cli, Commands};
use tracing_subscriber::prelude::*;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let saf_layer = saf_core::logging::subscriber::init();
    // Diagnostics (incl. frontend "unsupported instruction" warnings) go to
    // STDERR so that stdout stays clean — `saf verify` requires verdict-only
    // stdout for BenchExec, and other commands write their real output to stdout.
    if cli.json_errors {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_writer(std::io::stderr)
                    .with_filter(tracing_subscriber::EnvFilter::new("info")),
            )
            .with(saf_layer)
            .init();
    } else {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(std::io::stderr)
                    .with_filter(tracing_subscriber::EnvFilter::new("info")),
            )
            .with(saf_layer)
            .init();
    }

    match cli.command {
        Commands::Index(args) => commands::index(&args),
        Commands::Run(args) => commands::run(&args),
        Commands::Verify(args) => commands::verify(&args),
        Commands::EmitCorrectnessWitness(args) => commands::emit_correctness_witness(&args),
        Commands::ProveNoOverflow(args) => commands::prove_no_overflow_cmd(&args),
        Commands::ProveUnreachable(args) => commands::prove_unreachable_cmd(&args),
        Commands::Query(args) => commands::query(&args),
        Commands::Export(args) => commands::export(&args),
        Commands::Schema(args) => commands::schema(&args),
        Commands::Specs(args) => commands::specs(&args),
        Commands::Incremental(args) => commands::incremental(&args),
        Commands::Help(args) => help::print_help(args.topic.as_deref()),
    }
}
