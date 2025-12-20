//! cargo-coupling CLI - Coupling Analysis Tool
//!
//! Analyzes Rust projects for coupling patterns and generates reports.
//! Features parallel processing for large codebases.
//!
//! Usage:
//!   cargo coupling [OPTIONS] [PATH]
//!   cargo-coupling [OPTIONS] [PATH]

use std::fs::File;
use std::io::{BufWriter, Write, stdout};
use std::path::PathBuf;
use std::process;
use std::time::Instant;

use clap::{Parser, Subcommand};

use cargo_coupling::{
    CompiledConfig, IssueThresholds, VolatilityAnalyzer, analyze_workspace,
    generate_ai_output_with_thresholds, generate_report_with_thresholds,
    generate_summary_with_thresholds, load_compiled_config,
    web::{ServerConfig, start_server},
};

/// cargo-coupling - Measure the "right distance" in your Rust code
#[derive(Parser, Debug)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Analyze coupling in a Rust project
    Coupling(Args),
}

#[derive(Parser, Debug)]
struct Args {
    /// Path to the project or directory to analyze
    #[arg(default_value = "./src")]
    path: PathBuf,

    /// Output file for the report (default: stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Show summary only (no detailed report)
    #[arg(short, long)]
    summary: bool,

    /// AI-friendly output format for use with coding agents (Claude, Copilot, etc.)
    #[arg(long)]
    ai: bool,

    /// Analyze git history for volatility (months to look back)
    #[arg(long, default_value = "6")]
    git_months: usize,

    /// Skip git history analysis
    #[arg(long)]
    no_git: bool,

    /// Config file path (default: search for .coupling.toml)
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Show timing information
    #[arg(long)]
    timing: bool,

    /// Number of threads for parallel processing (default: all CPU cores)
    #[arg(long, short = 'j', value_name = "N")]
    jobs: Option<usize>,

    // === Threshold options ===
    /// Max outgoing dependencies before flagging as High Efferent Coupling
    #[arg(long)]
    max_deps: Option<usize>,

    /// Max incoming dependencies before flagging as High Afferent Coupling
    #[arg(long)]
    max_dependents: Option<usize>,

    // === Web visualization options ===
    /// Start web server for interactive visualization
    #[arg(long)]
    web: bool,

    /// Port for web server (default: 3000)
    #[arg(long, default_value = "3000")]
    port: u16,

    /// Don't open browser automatically when starting web server
    #[arg(long)]
    no_open: bool,

    /// API endpoint URL for frontend (useful for separate deployments)
    #[arg(long)]
    api_endpoint: Option<String>,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let Commands::Coupling(args) = cli.command;

    // Detect available CPU cores
    let available_cores = std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(1);

    // Configure thread pool
    let num_threads = args.jobs.unwrap_or(available_cores);
    if num_threads != available_cores || args.jobs.is_some() {
        rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build_global()
            .unwrap_or_else(|e| eprintln!("Warning: Could not set thread count: {}", e));
    }

    if args.verbose || args.timing {
        eprintln!(
            "Using {} thread(s) for parallel processing ({} CPU cores available)",
            num_threads, available_cores
        );
    }

    let total_start = Instant::now();

    // Load configuration file
    let config_path = args.config.as_ref().unwrap_or(&args.path);
    let mut config = match load_compiled_config(config_path) {
        Ok(config) => {
            if args.verbose && config.has_volatility_overrides() {
                eprintln!("Loaded configuration from .coupling.toml");
            }
            config
        }
        Err(e) => {
            if args.verbose {
                eprintln!("Note: No config file loaded: {}", e);
            }
            CompiledConfig::empty()
        }
    };

    // Print analysis header
    eprintln!("Analyzing project at '{}'...", args.path.display());

    // Analyze the project (uses cargo metadata for better accuracy)
    let analysis_start = Instant::now();
    let mut metrics = analyze_workspace(&args.path)?;
    let analysis_time = analysis_start.elapsed();

    // Analyze git history for volatility (if not disabled)
    if !args.no_git {
        if args.verbose {
            eprintln!("Analyzing git history ({} months)...", args.git_months);
        }

        let mut volatility = VolatilityAnalyzer::new(args.git_months);
        match volatility.analyze(&args.path) {
            Ok(()) => {
                if args.verbose {
                    let stats = volatility.statistics();
                    eprintln!(
                        "Git analysis: {} files, {} total changes",
                        stats.total_files, stats.total_changes
                    );
                }

                // Copy file changes to project metrics (must be after statistics())
                metrics.file_changes = volatility.file_changes;

                // Update volatility for all couplings based on git history
                metrics.update_volatility_from_git();
            }
            Err(e) => {
                if args.verbose {
                    eprintln!("Warning: Git analysis failed: {}", e);
                }
            }
        }
    }

    // Apply volatility overrides from config
    if config.has_volatility_overrides() {
        let mut override_count = 0;
        for coupling in &mut metrics.couplings {
            // Use the target path for volatility lookup
            if let Some(override_vol) = config.get_volatility_override(&coupling.target) {
                coupling.volatility = override_vol;
                override_count += 1;
            }
        }
        if args.verbose && override_count > 0 {
            eprintln!(
                "Applied {} volatility overrides from config",
                override_count
            );
        }
    }

    if args.timing {
        eprintln!(
            "Analysis complete: {} files, {} modules (took {:.2?})\n",
            metrics.total_files,
            metrics.module_count(),
            analysis_time
        );
    } else {
        eprintln!(
            "Analysis complete: {} files, {} modules\n",
            metrics.total_files,
            metrics.module_count()
        );
    }

    // Create custom thresholds - CLI args override config, which overrides defaults
    let thresholds = IssueThresholds {
        max_dependencies: args.max_deps.unwrap_or(config.thresholds.max_dependencies),
        max_dependents: args
            .max_dependents
            .unwrap_or(config.thresholds.max_dependents),
        ..IssueThresholds::default()
    };

    if args.verbose {
        eprintln!(
            "Thresholds: max_deps={}, max_dependents={}",
            thresholds.max_dependencies, thresholds.max_dependents
        );
    }

    // Web visualization mode
    if args.web {
        let server_config = ServerConfig {
            port: args.port,
            open_browser: !args.no_open,
            api_endpoint: args.api_endpoint.clone(),
        };

        // Run the web server using tokio runtime
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(start_server(metrics, thresholds, server_config))
            .map_err(|e| -> Box<dyn std::error::Error> { e })?;

        return Ok(());
    }

    // Generate output
    let output: Box<dyn Write> = match &args.output {
        Some(path) => {
            let file = File::create(path)?;
            Box::new(BufWriter::new(file))
        }
        None => Box::new(stdout()),
    };

    let mut writer = output;

    if args.ai {
        generate_ai_output_with_thresholds(&metrics, &thresholds, &mut writer)?;
    } else if args.summary {
        generate_summary_with_thresholds(&metrics, &thresholds, &mut writer)?;
    } else {
        generate_report_with_thresholds(&metrics, &thresholds, &mut writer)?;
    }

    // Notify about output file
    if let Some(path) = &args.output {
        eprintln!("Report written to: {}", path.display());
    }

    // Show total timing
    if args.timing {
        let total_time = total_start.elapsed();
        let files_per_sec = metrics.total_files as f64 / total_time.as_secs_f64();
        eprintln!(
            "Total time: {:.2?} ({:.1} files/sec)",
            total_time, files_per_sec
        );
    }

    Ok(())
}
