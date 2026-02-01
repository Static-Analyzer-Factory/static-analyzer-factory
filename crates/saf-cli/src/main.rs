mod commands;
mod driver;
mod help;

use clap::Parser;
use commands::{Cli, Commands};
use tracing_subscriber::prelude::*;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let saf_layer = saf_core::logging::subscriber::init();
    if cli.json_errors {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_filter(tracing_subscriber::EnvFilter::new("info")),
            )
            .with(saf_layer)
            .init();
    } else {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_filter(tracing_subscriber::EnvFilter::new("info")),
            )
            .with(saf_layer)
            .init();
    }

    match cli.command {
        Commands::Index(args) => commands::index(&args),
        Commands::Run(args) => commands::run(&args),
        Commands::Query(args) => commands::query(&args),
        Commands::Export(args) => commands::export(&args),
        Commands::Schema(args) => commands::schema(&args),
        Commands::Specs(args) => commands::specs(&args),
        Commands::Incremental(args) => commands::incremental(&args),
        Commands::Help(args) => help::print_help(args.topic.as_deref()),
    }
}
