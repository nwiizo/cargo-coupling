use std::{fs, process::Command};
use tempfile::tempdir;

#[test]
fn design_report_connects_declared_context_to_observed_modules() {
    let dir = tempdir().unwrap();
    fs::create_dir(dir.path().join("src")).unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname='design-fixture'\nversion='0.1.0'\nedition='2024'\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("src/lib.rs"),
        "pub mod policy; pub mod service;\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("src/policy.rs"),
        "pub fn price(n: u32) -> u32 { n * 2 }\n",
    )
    .unwrap();
    fs::write(dir.path().join("src/service.rs"), "pub fn quote(n: u32) -> u32 { crate::policy::price(n) }\n#[test] fn quote_test() { assert_eq!(quote(2), 4); }\n").unwrap();
    fs::write(
        dir.path().join(".coupling-context.toml"),
        r#"
version = 1
[[components]]
name = "pricing"
modules = ["*policy"]
owners = ["@pricing"]
planned_changes = "Introduce regional pricing"
business_value = 3
effort_days = 2
release_unit = "shop"
[[components]]
name = "quotes"
modules = ["*service"]
owners = ["@checkout"]
release_unit = "shop"
[[rules]]
id = "regional-pricing"
modules = ["*policy", "*service"]
description = "Both must agree on the regional pricing rule"
[[relationships]]
id = "quote-consistency"
source = "*service"
target = "*policy"
kind = "transaction"
evidence = "A quote must use one consistent price revision"
[[decisions]]
id = "keep-together"
modules = ["*policy", "*service"]
reason = "Both ship as one unit"
review_on = ["planned-change", "cross-team"]
[[scenarios]]
name = "Keep knowledge local"
description = "A responsibility move, not an implemented change"
[[scenarios.changes]]
source = "*service"
target = "*policy"
distance = 0.0
[[scenarios]]
name = "Outside this scope"
description = "No observation is not a zero balance"
[[scenarios.changes]]
source = "missing"
target = "missing"
distance = 0.0
"#,
    )
    .unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_cargo-coupling"))
        .args(["coupling", "--design", "--json", "--no-git"])
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    let design = &report["design"];
    assert!(design["scenarios"][0]["affected_edges"].as_u64().unwrap() > 0);
    assert!(design["scenarios"][0]["hypothetical_balance"].is_number());
    assert_eq!(design["scenarios"][1]["affected_edges"], 0);
    assert!(design["scenarios"][1]["hypothetical_balance"].is_null());
    assert!(
        design["hierarchy"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["level"] == "item")
    );
    assert!(
        design["shared_reasons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["origin"] == "declared")
    );
    assert_eq!(design["schema_version"], 1);
    assert!(
        design["coordination"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["cross_team"] == true)
    );
    assert!(
        design["lifecycle"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["unit"] == "shop")
    );
    assert!(
        design["runtime"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["kind"] == "transaction")
    );
    assert!(
        design["decisions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["status"] == "review")
    );
    assert!(
        design["priorities"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["planned_changes"] == "Introduce regional pricing")
    );
}

#[test]
fn git_change_reaches_three_hops_and_current_tests_while_retaining_deleted_item_revision() {
    let dir = tempdir().unwrap();
    fs::create_dir(dir.path().join("src")).unwrap();
    fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname='change-fixture'\nversion='0.1.0'\nedition='2024'\n",
    )
    .unwrap();
    for (file, source) in [
        ("lib.rs", "pub mod a; pub mod b; pub mod c; pub mod d;\n"),
        (
            "a.rs",
            "pub fn origin() -> u32 { 1 }\npub fn removed() {}\n",
        ),
        ("b.rs", "pub fn step() -> u32 { crate::a::origin() }\n"),
        ("c.rs", "pub fn step() -> u32 { crate::b::step() }\n"),
        (
            "d.rs",
            "pub fn run() -> u32 { crate::c::step() }\n#[test] fn chain_test() { let _value = run(); }\n",
        ),
    ] {
        fs::write(dir.path().join("src").join(file), source).unwrap();
    }
    for args in [
        vec!["init", "-q"],
        vec!["add", "."],
        vec![
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "-qm",
            "baseline",
        ],
    ] {
        assert!(
            Command::new("git")
                .current_dir(dir.path())
                .args(args)
                .status()
                .unwrap()
                .success()
        );
    }
    fs::write(
        dir.path().join("src/a.rs"),
        "pub fn origin() -> u32 { 2 }\n",
    )
    .unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_cargo-coupling"))
        .args([
            "coupling",
            "--design",
            "--json",
            "--no-git",
            "--changed-since",
            "HEAD",
            "--impact-depth",
            "3",
        ])
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    let design = &report["design"];
    let origin = design["impact"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["origin"].as_str().unwrap().rsplit("::").next() == Some("a"))
        .expect("changed a module must be present");
    assert!(
        origin["reachability"]["paths"]
            .as_array()
            .unwrap()
            .iter()
            .any(
                |path| path["module"].as_str().unwrap().rsplit("::").next() == Some("d")
                    && path["path"].as_array().unwrap().len() == 4
            ),
        "{origin:#}"
    );
    assert!(
        origin["tests"]
            .as_array()
            .unwrap()
            .iter()
            .any(|test| test["item"]["name"] == "chain_test")
    );
    assert!(
        design["changes"]["files"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|file| file["items"].as_array().unwrap())
            .any(|item| item["name"] == "removed"
                && item["revision"] == design["changes"]["baseline"])
    );
    assert!(design["baseline"]["reference_commit"].is_string());
    assert_eq!(
        design["baseline"]["settings_fingerprint"],
        design["provenance"]["config_fingerprint"]
    );
}

#[test]
fn malformed_context_fails_instead_of_silently_ignoring_design_facts() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("lib.rs"), "pub fn f() {}\n").unwrap();
    fs::write(
        dir.path().join(".coupling-context.toml"),
        "version=1\nunknown_option=true\n",
    )
    .unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_cargo-coupling"))
        .args(["coupling", "--design", "--no-git"])
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("unknown field"));
}
