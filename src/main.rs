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
    CompiledConfig, IssueThresholds, ManifestContext, Severity, TextReportOptions,
    VolatilityAnalyzer, analyze_external_dependencies, analyze_history, analyze_ref,
    analyze_workspace_with_config, build_manifest,
    cli_output::{
        CheckConfig, generate_baseline_diff_output, generate_check_output,
        generate_external_dependencies_output, generate_history_output, generate_hotspots_output,
        generate_impact_output, generate_json_output, generate_json_output_with_diff,
        generate_ratchet_check_output, parse_grade, parse_severity,
    },
    diff_ref_analysis, generate_ai_output_with_thresholds, generate_report_with_options,
    generate_summary_with_options, load_compiled_config, load_lock_versions_near,
    web::{DEFAULT_HISTORY_MAX_POINTS, ServerConfig, start_server},
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

    /// Exclude test code from analysis (#[test], #[cfg(test)], mod tests)
    #[arg(long)]
    exclude_tests: bool,

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

    // === Job-focused CLI options ===
    /// Show top N refactoring hotspots (default: 5). Use --hotspots or --hotspots=N
    #[arg(long, value_name = "N", num_args = 0..=1, require_equals = true, default_missing_value = "5")]
    hotspots: Option<usize>,

    /// Show third-party crate coupling breadth and scattered usage risks
    #[arg(long)]
    deps: bool,

    /// Analyze change impact for a specific module
    #[arg(long, value_name = "MODULE")]
    impact: Option<String>,

    /// Trace dependencies for a specific function/type (e.g., "analyze_file" or "BalanceScore")
    #[arg(long, value_name = "ITEM")]
    trace: Option<String>,

    /// Show coupling health over git history (default: 12 samples). Window set by --git-months
    #[arg(long, value_name = "N", num_args = 0..=1, require_equals = true, default_missing_value = "12")]
    history: Option<usize>,

    /// Compare current issues against a baseline git ref (commit, branch, or tag)
    #[arg(long, value_name = "GIT_REF")]
    baseline: Option<String>,

    /// Run quality gate check (returns non-zero exit code on failure)
    #[arg(long)]
    check: bool,

    /// Minimum grade for --check (A, B, C, D, F). Default: C
    #[arg(long, value_name = "GRADE", requires = "check")]
    min_grade: Option<String>,

    /// Maximum critical issues for --check. Default: 0
    #[arg(long, value_name = "N", requires = "check")]
    max_critical: Option<usize>,

    /// Maximum circular dependencies for --check. Default: 0
    #[arg(long, value_name = "N", requires = "check")]
    max_circular: Option<usize>,

    /// Fail --check on any issue at this severity or higher (critical, high, medium, low)
    #[arg(long, value_name = "SEVERITY", requires = "check")]
    fail_on: Option<String>,

    /// Output in JSON format (machine-readable)
    #[arg(long)]
    json: bool,

    /// Show all issues including Low severity (default: only Medium/High/Critical)
    #[arg(long)]
    all: bool,

    /// Show the full structural blind-spot list in text output
    #[arg(long)]
    blind_spots: bool,

    /// Show explanations in Japanese (日本語で解説を表示)
    #[arg(long, visible_alias = "jp")]
    japanese: bool,
}

fn main() {
    match run() {
        Ok(exit_code) => process::exit(exit_code),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn run() -> Result<i32, Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let Commands::Coupling(args) = cli.command;

    run_coupling(args)
}

fn run_coupling(args: Args) -> Result<i32, Box<dyn std::error::Error>> {
    warn_on_output_mode_conflicts(&args);

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

    // Apply CLI flags to config (CLI takes precedence over config file)
    if args.exclude_tests {
        config.set_exclude_tests(true);
    }

    if args.verbose && config.exclude_tests {
        eprintln!("Test code will be excluded from analysis");
    }

    if args.verbose && config.prelude_module_count() > 0 {
        eprintln!(
            "Prelude modules configured: {} pattern(s)",
            config.prelude_module_count()
        );
    }

    // Create custom thresholds - CLI args override config, which overrides defaults.
    // Computed early so both the history timeline and the snapshot analysis share them.
    let thresholds = IssueThresholds {
        max_dependencies: args.max_deps.unwrap_or(config.thresholds.max_dependencies),
        max_dependents: args
            .max_dependents
            .unwrap_or(config.thresholds.max_dependents),
        strict_mode: !args.all, // Default is strict (hide Low), --all shows everything
        japanese: args.japanese,
        exclude_tests: config.exclude_tests,
        prelude_module_count: config.prelude_module_count(),
        ..IssueThresholds::default()
    };

    if args.verbose {
        eprintln!(
            "Thresholds: max_deps={}, max_dependents={}",
            thresholds.max_dependencies, thresholds.max_dependents
        );
    }

    // --history: time-series coupling health across git revisions. Independent of
    // the snapshot analysis below, so handle it here and return early.
    if let Some(max_points) = args.history {
        if args.baseline.is_some() {
            return Err(invalid_cli_input("--baseline cannot be combined with --history").into());
        }
        if max_points == 0 {
            return Err(invalid_cli_input("--history must be greater than 0").into());
        }
        if args.no_git {
            eprintln!(
                "Warning: --history requires git history; ignoring --no-git for history analysis."
            );
        }
        eprintln!(
            "Analyzing coupling history ({} months, up to {} samples)...",
            args.git_months, max_points
        );

        let report = analyze_history(
            &args.path,
            &config,
            &thresholds,
            args.git_months,
            max_points,
        )
        .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?;

        let mut writer: Box<dyn Write> = match &args.output {
            Some(path) => Box::new(BufWriter::new(File::create(path)?)),
            None => Box::new(stdout()),
        };
        generate_history_output(&report, args.json, max_points, &mut writer)?;
        return Ok(0);
    }

    // Print analysis header
    eprintln!("Analyzing project at '{}'...", args.path.display());

    // Analyze the project (uses cargo metadata for better accuracy)
    let analysis_start = Instant::now();
    let mut metrics = analyze_workspace_with_config(&args.path, &config)?;
    let analysis_time = analysis_start.elapsed();

    // Analyze git history for volatility (if not disabled)
    let mut git_used = false;
    if !args.no_git {
        if args.verbose {
            eprintln!("Analyzing git history ({} months)...", args.git_months);
        }

        let mut volatility = VolatilityAnalyzer::new(args.git_months);
        match volatility.analyze(&args.path) {
            Ok(()) => {
                git_used = true;
                if args.verbose {
                    let stats = volatility.statistics();
                    eprintln!(
                        "Git analysis: {} files, {} total changes",
                        stats.total_files, stats.total_changes
                    );
                }

                // Analyze temporal coupling (co-change patterns)
                match volatility.analyze_temporal_coupling(&args.path) {
                    Ok(temporal) => {
                        if args.verbose && !temporal.is_empty() {
                            eprintln!(
                                "Temporal coupling: {} co-changing file pairs detected",
                                temporal.len()
                            );
                        }
                        metrics.temporal_couplings = temporal;
                    }
                    Err(e) => {
                        if args.verbose {
                            eprintln!("Warning: Temporal coupling analysis failed: {}", e);
                        }
                    }
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

    // Apply volatility overrides from config (includes subdomain classification)
    if config.has_subdomain_config() && args.verbose {
        eprintln!("DDD subdomain classification configured (affects volatility)");
    }
    if config.has_volatility_overrides() || config.has_subdomain_config() {
        let override_count = metrics.apply_config_volatility_overrides(&mut config);
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

    let manifest = build_manifest(&ManifestContext {
        git_used,
        tests_excluded: config.exclude_tests,
        parse_failures: metrics.parse_failures,
        skipped_crates: metrics.skipped_crates.clone(),
    });

    // Web visualization mode
    if args.web {
        let server_config = ServerConfig {
            port: args.port,
            open_browser: !args.no_open,
            api_endpoint: args.api_endpoint.clone(),
            analysis_path: args.path.clone(),
            analysis_config: config,
            git_months: args.git_months,
            history_max_points: DEFAULT_HISTORY_MAX_POINTS,
            no_git: args.no_git,
        };

        // Run the web server using tokio runtime
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(start_server(metrics, thresholds, server_config))
            .map_err(|e| -> Box<dyn std::error::Error> { e })?;

        return Ok(0);
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

    // Job-focused CLI modes (mutually exclusive with other modes)

    // --baseline: compare current issues against a git ref. With --check this
    // is a ratchet gate that fails only for new issues at the configured severity.
    if let Some(baseline_ref) = &args.baseline {
        let baseline = analyze_ref(
            &args.path,
            &config,
            &thresholds,
            baseline_ref,
            args.git_months,
            !args.no_git,
        )
        .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?;
        let current_report =
            cargo_coupling::analyze_project_balance_with_thresholds(&metrics, &thresholds);
        let diff = diff_ref_analysis(&baseline, &current_report);

        if args.json {
            generate_json_output_with_diff(&metrics, &thresholds, &manifest, &diff, &mut writer)?;
        } else if args.check {
            let fail_on = ratchet_fail_on_from_args(&args)?;
            let exit_code =
                generate_ratchet_check_output(&diff, baseline_ref, fail_on, &mut writer)?;
            return Ok(exit_code);
        } else {
            generate_baseline_diff_output(&diff, baseline_ref, &mut writer)?;
        }
        return Ok(0);
    }

    // --deps: Show third-party dependency coupling exposure
    if args.deps {
        let versions = load_lock_versions_near(&args.path);
        let report = analyze_external_dependencies(&metrics, &versions);
        generate_external_dependencies_output(&report, args.json, args.japanese, &mut writer)?;
        return Ok(0);
    }

    // --json: Machine-readable JSON output
    if args.json {
        generate_json_output(&metrics, &thresholds, &manifest, &mut writer)?;
        return Ok(0);
    }

    // --check: Quality gate check (returns exit code)
    if args.check {
        let check_config = check_config_from_args(&args)?;
        let exit_code = generate_check_output(&metrics, &thresholds, &check_config, &mut writer)?;
        return Ok(exit_code);
    }

    // --hotspots: Show top refactoring targets
    if let Some(limit) = args.hotspots {
        generate_hotspots_output(&metrics, &thresholds, limit, args.verbose, &mut writer)?;
        return Ok(0);
    }

    // --impact: Analyze impact of a specific module
    if let Some(module_name) = &args.impact {
        let found = generate_impact_output(&metrics, module_name, &mut writer)?;
        if !found {
            return Ok(1);
        }
        return Ok(0);
    }

    // --trace: Trace dependencies for a specific function/type
    if let Some(item_name) = &args.trace {
        let found =
            cargo_coupling::cli_output::generate_trace_output(&metrics, item_name, &mut writer)?;
        if !found {
            return Ok(1);
        }
        return Ok(0);
    }

    // Default modes
    if args.ai {
        generate_ai_output_with_thresholds(&metrics, &thresholds, &manifest, &mut writer)?;
    } else if args.summary {
        generate_summary_with_options(
            &metrics,
            &thresholds,
            &manifest,
            args.blind_spots || args.all,
            &mut writer,
        )?;
    } else {
        generate_report_with_options(
            &metrics,
            &thresholds,
            &manifest,
            TextReportOptions {
                show_structural_blind_spots: args.blind_spots || args.all,
                show_all_temporal_couplings: args.all,
            },
            &mut writer,
        )?;
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

    Ok(0)
}

fn warn_on_output_mode_conflicts(args: &Args) {
    if let Some((used, ignored)) = output_mode_conflict(args) {
        eprintln!("Warning: using {}; ignoring {}.", used, ignored.join(", "));
    }
}

fn output_mode_conflict(args: &Args) -> Option<(&'static str, Vec<&'static str>)> {
    let mut modes = Vec::new();

    if args.history.is_some() {
        modes.push("--history");
    }
    if args.web {
        modes.push("--web");
    }
    if args.json && args.history.is_none() && !args.deps {
        modes.push("--json");
    }
    if args.deps {
        modes.push("--deps");
    }
    if args.check {
        modes.push("--check");
    }
    if args.hotspots.is_some() {
        modes.push("--hotspots");
    }
    if args.impact.is_some() {
        modes.push("--impact");
    }
    if args.trace.is_some() {
        modes.push("--trace");
    }
    if args.ai {
        modes.push("--ai");
    }
    if args.summary {
        modes.push("--summary");
    }

    (modes.len() > 1).then(|| (modes[0], modes[1..].to_vec()))
}

fn check_config_from_args(args: &Args) -> Result<CheckConfig, std::io::Error> {
    let has_gate_flag = args.min_grade.is_some()
        || args.max_critical.is_some()
        || args.max_circular.is_some()
        || args.fail_on.is_some();

    if !has_gate_flag {
        return Ok(CheckConfig::default());
    }

    let min_grade = match args.min_grade.as_deref() {
        Some(value) => Some(parse_grade(value).ok_or_else(|| {
            invalid_cli_input(format!(
                "invalid --min-grade '{}'; expected S, A, B, C, D, or F",
                value
            ))
        })?),
        None => None,
    };

    let fail_on = match args.fail_on.as_deref() {
        Some(value) => Some(parse_severity(value).ok_or_else(|| {
            invalid_cli_input(format!(
                "invalid --fail-on '{}'; expected critical, high, medium, or low",
                value
            ))
        })?),
        None => None,
    };

    Ok(CheckConfig {
        min_grade,
        max_critical: args.max_critical,
        max_circular: args.max_circular,
        fail_on,
    })
}

fn ratchet_fail_on_from_args(args: &Args) -> Result<Severity, std::io::Error> {
    match args.fail_on.as_deref() {
        Some(value) => parse_severity(value).ok_or_else(|| {
            invalid_cli_input(format!(
                "invalid --fail-on '{}'; expected critical, high, medium, or low",
                value
            ))
        }),
        None => Ok(Severity::High),
    }
}

fn invalid_cli_input(message: impl Into<String>) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, message.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cargo_coupling::{HealthGrade, Severity};
    use std::path::Path;

    fn base_args(path: PathBuf) -> Args {
        Args {
            path,
            output: None,
            summary: false,
            ai: false,
            git_months: 6,
            no_git: true,
            exclude_tests: false,
            config: None,
            verbose: false,
            timing: false,
            jobs: None,
            max_deps: None,
            max_dependents: None,
            web: false,
            port: 3000,
            no_open: true,
            api_endpoint: None,
            hotspots: None,
            deps: false,
            impact: None,
            trace: None,
            history: None,
            baseline: None,
            check: false,
            min_grade: None,
            max_critical: None,
            max_circular: None,
            fail_on: None,
            json: false,
            all: false,
            blind_spots: false,
            japanese: false,
        }
    }

    fn write_files(src: &Path, circular: bool) {
        std::fs::create_dir_all(src).unwrap();
        std::fs::write(src.join("lib.rs"), "pub mod a;\npub mod b;\n").unwrap();
        if circular {
            std::fs::write(src.join("a.rs"), "use crate::b::B;\npub struct A(pub B);\n").unwrap();
            std::fs::write(src.join("b.rs"), "use crate::a::A;\npub struct B(pub A);\n").unwrap();
        } else {
            std::fs::write(src.join("a.rs"), "pub struct A;\n").unwrap();
            std::fs::write(
                src.join("b.rs"),
                "use crate::a::A;\npub fn take(_a: A) {}\n",
            )
            .unwrap();
        }
    }

    #[test]
    fn bare_check_uses_documented_defaults() {
        let mut args = base_args(PathBuf::from("src"));
        args.check = true;

        let config = check_config_from_args(&args).unwrap();

        assert_eq!(config.min_grade, Some(HealthGrade::C));
        assert_eq!(config.max_critical, Some(0));
        assert_eq!(config.max_circular, Some(0));
        assert_eq!(config.fail_on, None);
    }

    #[test]
    fn check_with_any_gate_flag_keeps_only_specified_gates() {
        let mut args = base_args(PathBuf::from("src"));
        args.check = true;
        args.fail_on = Some("high".to_string());

        let config = check_config_from_args(&args).unwrap();

        assert_eq!(config.min_grade, None);
        assert_eq!(config.max_critical, None);
        assert_eq!(config.max_circular, None);
        assert_eq!(config.fail_on, Some(Severity::High));
    }

    #[test]
    fn invalid_check_grade_and_severity_are_errors() {
        let mut args = base_args(PathBuf::from("src"));
        args.check = true;
        args.min_grade = Some("ZZZ".to_string());
        assert!(
            check_config_from_args(&args)
                .unwrap_err()
                .to_string()
                .contains("invalid --min-grade")
        );

        args.min_grade = None;
        args.fail_on = Some("bogus".to_string());
        assert!(
            check_config_from_args(&args)
                .unwrap_err()
                .to_string()
                .contains("invalid --fail-on")
        );
    }

    #[test]
    fn ratchet_defaults_to_high_and_parses_fail_on() {
        let mut args = base_args(PathBuf::from("src"));
        args.check = true;
        args.baseline = Some("HEAD~1".to_string());

        assert_eq!(ratchet_fail_on_from_args(&args).unwrap(), Severity::High);

        args.fail_on = Some("medium".to_string());
        assert_eq!(ratchet_fail_on_from_args(&args).unwrap(), Severity::Medium);

        args.fail_on = Some("bogus".to_string());
        assert!(
            ratchet_fail_on_from_args(&args)
                .unwrap_err()
                .to_string()
                .contains("invalid --fail-on")
        );
    }

    #[test]
    fn check_output_file_is_flushed_on_pass_and_fail() {
        let passing = tempfile::tempdir().unwrap();
        let passing_src = passing.path().join("src");
        write_files(&passing_src, false);
        let passing_output = passing.path().join("pass.txt");

        let mut pass_args = base_args(passing_src);
        pass_args.check = true;
        pass_args.min_grade = Some("F".to_string());
        pass_args.max_critical = Some(usize::MAX);
        pass_args.max_circular = Some(usize::MAX);
        pass_args.output = Some(passing_output.clone());

        assert_eq!(run_coupling(pass_args).unwrap(), 0);
        assert!(
            std::fs::metadata(&passing_output).unwrap().len() > 0,
            "passing check output should be flushed before returning"
        );

        let failing = tempfile::tempdir().unwrap();
        let failing_src = failing.path().join("src");
        write_files(&failing_src, true);
        let failing_output = failing.path().join("fail.txt");

        let mut fail_args = base_args(failing_src);
        fail_args.check = true;
        fail_args.output = Some(failing_output.clone());

        assert_eq!(run_coupling(fail_args).unwrap(), 1);
        assert!(
            std::fs::metadata(&failing_output).unwrap().len() > 0,
            "failing check output should be flushed before returning"
        );
    }

    #[test]
    fn history_zero_is_rejected_before_git_analysis() {
        let tmp = tempfile::tempdir().unwrap();
        let mut args = base_args(tmp.path().to_path_buf());
        args.history = Some(0);

        let error = run_coupling(args).unwrap_err().to_string();

        assert!(error.contains("--history must be greater than 0"));
    }

    #[test]
    fn json_output_reports_no_git_and_excluded_tests() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        write_files(&src, false);
        let output = tmp.path().join("manifest.json");

        let mut args = base_args(src);
        args.json = true;
        args.exclude_tests = true;
        args.output = Some(output.clone());

        assert_eq!(run_coupling(args).unwrap(), 0);

        let text = std::fs::read_to_string(output).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
        let notes = parsed["analysis_manifest"]["notes"].as_array().unwrap();

        assert!(notes.iter().any(|note| {
            note.as_str()
                .is_some_and(|note| note.contains("Git history was not analyzed"))
        }));
        assert!(notes.iter().any(|note| {
            note.as_str()
                .is_some_and(|note| note.contains("Test code was excluded"))
        }));
    }

    #[test]
    fn json_manifest_reports_parse_failures() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        write_files(&src, false);
        std::fs::write(src.join("broken.rs"), "pub fn broken( {").unwrap();
        let output = tmp.path().join("manifest.json");

        let mut args = base_args(src);
        args.json = true;
        args.output = Some(output.clone());

        assert_eq!(run_coupling(args).unwrap(), 0);

        let text = std::fs::read_to_string(output).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&text).unwrap();
        let notes = parsed["analysis_manifest"]["notes"].as_array().unwrap();

        assert!(notes.iter().any(|note| {
            note.as_str()
                .is_some_and(|note| note.contains("1 source file(s) failed to parse"))
        }));
    }

    #[test]
    fn default_text_manifest_is_concise_and_blind_spots_are_opt_in() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        write_files(&src, false);

        let default_output = tmp.path().join("default.txt");
        let mut default_args = base_args(src.clone());
        default_args.output = Some(default_output.clone());
        assert_eq!(run_coupling(default_args).unwrap(), 0);
        let default_text = std::fs::read_to_string(default_output).unwrap();
        assert!(default_text.contains("4 structural blind spots not analyzed"));
        assert!(default_text.contains("Git history was not analyzed"));
        assert!(!default_text.contains("Dynamic connascence (Execution"));

        let blind_spots_output = tmp.path().join("blind-spots.txt");
        let mut blind_spots_args = base_args(src.clone());
        blind_spots_args.blind_spots = true;
        blind_spots_args.output = Some(blind_spots_output.clone());
        assert_eq!(run_coupling(blind_spots_args).unwrap(), 0);
        let blind_spots_text = std::fs::read_to_string(blind_spots_output).unwrap();
        assert!(blind_spots_text.contains("Dynamic connascence (Execution"));

        let all_output = tmp.path().join("all.txt");
        let mut all_args = base_args(src);
        all_args.all = true;
        all_args.output = Some(all_output.clone());
        assert_eq!(run_coupling(all_args).unwrap(), 0);
        let all_text = std::fs::read_to_string(all_output).unwrap();
        assert!(all_text.contains("Dynamic connascence (Execution"));
    }

    #[test]
    fn output_mode_conflicts_follow_existing_precedence() {
        let mut args = base_args(PathBuf::from("src"));
        args.json = true;
        args.check = true;
        assert_eq!(
            output_mode_conflict(&args),
            Some(("--json", vec!["--check"]))
        );

        args.json = false;
        args.check = false;
        args.summary = true;
        args.ai = true;
        assert_eq!(
            output_mode_conflict(&args),
            Some(("--ai", vec!["--summary"]))
        );

        args.history = Some(8);
        args.json = true;
        args.ai = false;
        args.summary = false;
        assert_eq!(output_mode_conflict(&args), None);
    }
}
