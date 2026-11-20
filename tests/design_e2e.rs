use std::{fs, process::Command};
use tempfile::tempdir;

#[test]
fn design_options_require_two_distinct_known_modules() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("a.rs"), "pub fn value() -> Result<Vec<u8>, std::io::Error> { Err(std::io::Error::other(\"bad\")) }\n").unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_cargo-coupling"))
        .args(["coupling", "--agent", "--focus", "a", "--no-git"])
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(result.status.success());
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["alternatives"], serde_json::json!([]));
    assert!(!report["evidence"].as_array().unwrap().is_empty());
}

#[test]
fn impact_test_candidates_exclude_helpers_and_fixtures() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("a.rs"),
        r#"
pub fn value() -> u8 { 1 }
#[cfg(test)]
mod tests {
    fn helper() -> u8 { super::value() }
    struct Fixture;
    impl Fixture { fn setup() {} }
    #[test] fn actual() { assert_eq!(helper(), 1); }
    #[tokio::test] async fn async_case() { assert_eq!(helper(), 1); }
    #[test] #[ignore = "manual"] fn ignored_case() { assert_eq!(helper(), 1); }
}
"#,
    )
    .unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_cargo-coupling"))
        .args(["coupling", "--agent", "--focus", "a", "--no-git"])
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(result.status.success());
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    let names: Vec<_> = report["impact"][0]["tests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|candidate| candidate["item"]["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["actual", "async_case", "ignored_case"]);
    let tests = report["impact"][0]["tests"].as_array().unwrap();
    assert_eq!(tests[0]["item"]["test_kind"], "test");
    assert_eq!(tests[1]["item"]["test_kind"], "test");
    assert_eq!(tests[2]["item"]["test_kind"], "ignored");
    assert!(tests[2]["reason"].as_str().unwrap().contains("--ignored"));
}

#[test]
fn focused_options_prioritize_the_origin_before_alphabetical_neighbors() {
    let dir = tempdir().unwrap();
    fs::write(
        dir.path().join("z_focus.rs"),
        "pub fn value() -> u8 { 1 }\n",
    )
    .unwrap();
    let mut consumer = String::from("pub fn work() { crate::z_focus::value();");
    for index in 0..8 {
        fs::write(
            dir.path().join(format!("a_{index}.rs")),
            "pub fn value() -> u8 { 1 }\n",
        )
        .unwrap();
        consumer.push_str(&format!("crate::a_{index}::value();"));
    }
    consumer.push_str("}\n");
    fs::write(dir.path().join("consumer.rs"), consumer).unwrap();
    fs::write(
        dir.path().join(".coupling.toml"),
        "[subdomains]\ncore=['*.rs']\n",
    )
    .unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_cargo-coupling"))
        .args(["coupling", "--agent", "--focus", "z_focus", "--no-git"])
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(result.status.success());
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["alternatives"][0]["target"], "z_focus");
    assert!(report["omitted"]["alternatives"].as_u64().unwrap() > 0);
}

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
        dir.path().join(".coupling.toml"),
        "[subdomains]\ncore=['src/policy.rs','src/service.rs']\n",
    )
    .unwrap();
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
evidence = ["docs/quote-consistency.md"]
rejected_options = ["Separate delivery without a consistent price revision"]
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
    assert_eq!(
        design["decisions"][0]["evidence"][0],
        "docs/quote-consistency.md"
    );
    assert_eq!(
        design["decisions"][0]["rejected_options"][0],
        "Separate delivery without a consistent price revision"
    );
    let separation = design["alternatives"]
        .as_array()
        .unwrap()
        .iter()
        .find(|option| option["action"] == "separate-responsibilities")
        .expect("retain a separation option for comparison");
    assert_eq!(separation["status"], "requires-review");
    assert!(
        separation["constraints"]
            .as_array()
            .unwrap()
            .iter()
            .any(|constraint| constraint.as_str().unwrap().contains("quote-consistency"))
    );
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

    // With no planning context, CLI and design report must agree exactly.
    fs::remove_file(dir.path().join(".coupling-context.toml")).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_cargo-coupling"))
        .args([
            "coupling",
            "--design",
            "--json",
            "--no-git",
            "--max-deps",
            "0",
        ])
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(result.status.success());
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    let priorities = report["design"]["priorities"].as_array().unwrap();
    let hotspots = report["hotspots"].as_array().unwrap();
    assert!(!hotspots.is_empty());
    for (hotspot, priority) in hotspots.iter().zip(priorities) {
        assert_eq!(hotspot["module"], priority["module"]);
        assert_eq!(
            hotspot["score"].as_f64(),
            priority["priority_score"].as_f64()
        );
    }
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
    let result = Command::new(env!("CARGO_BIN_EXE_cargo-coupling"))
        .args([
            "coupling",
            "--agent",
            "--no-git",
            "--changed-since",
            "HEAD",
            "--impact-depth",
            "3",
        ])
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(result.status.success());
    let review: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    // These fixture lists fit within the handoff limits. Borrowed serialization
    // must preserve every field, including revision, depth and baseline notes.
    assert_eq!(review["baseline"], design["baseline"]);
    assert_eq!(review["impact"], design["impact"]);
    assert_eq!(review["omitted"]["baseline.findings"], 0);
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

#[test]
fn agent_review_traces_declared_rules_without_inventing_code_dependencies() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("a.rs"), "pub fn price() -> u32 { 1 }\n").unwrap();
    fs::write(
        dir.path().join("b.rs"),
        "pub fn quote() -> u32 { 1 }\n#[test] fn quote_test() { assert_eq!(quote(), 1); }\n",
    )
    .unwrap();
    fs::write(dir.path().join("unrelated.rs"), "pub fn other() {}\n").unwrap();
    fs::write(dir.path().join(".coupling-context.toml"), "version=1\n[[rules]]\nid='pricing'\nmodules=['a','b']\ndescription='Both implement the same pricing rule'\n").unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_cargo-coupling"))
        .args(["coupling", "--agent", "--focus", "a", "--no-git"])
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let review: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(review["format"], "cargo-coupling/agent-review");
    assert_eq!(review["schema_version"], 1);
    assert_eq!(review["focus"], serde_json::json!(["a", "b"]));
    let impact = &review["impact"][0];
    assert_eq!(impact["origin"], "a");
    assert!(
        impact["reachability"]["paths"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let declared = &impact["contextual_reachability"]["paths"][0];
    assert_eq!(declared["module"], "b");
    assert_eq!(
        declared["steps"][0]["rule_ids"],
        serde_json::json!(["pricing"])
    );
    assert!(
        impact["tests"]
            .as_array()
            .unwrap()
            .iter()
            .any(|test| test["item"]["name"] == "quote_test")
    );
    assert!(!review["questions"].as_array().unwrap().is_empty());
    assert!(review["omitted"].is_object());
    assert!(!review["coverage"].as_array().unwrap().is_empty());
    assert_eq!(review["verification_status"], "not-run");

    let missing = Command::new(env!("CARGO_BIN_EXE_cargo-coupling"))
        .args(["coupling", "--agent", "--focus", "missing", "--no-git"])
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(!missing.status.success());
}

#[test]
fn agent_review_discloses_nested_omissions_and_preserves_verification_references() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("a.rs"), "pub fn price() {}\n").unwrap();
    let mut tests = String::from("pub fn quote() {}\n");
    for number in 0..25 {
        tests.push_str(&format!("#[test] fn quote_{number}() {{ quote(); }}\n"));
    }
    fs::write(dir.path().join("b.rs"), tests).unwrap();
    fs::write(dir.path().join(".coupling-context.toml"), "version=1\n[[rules]]\nid='pricing'\nmodules=['a','b']\ndescription='Shared price'\nverification=['b::quote_0']\n").unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_cargo-coupling"))
        .args(["coupling", "--agent", "--focus", "a", "--no-git"])
        .arg(dir.path())
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["impact"][0]["tests"].as_array().unwrap().len(), 20);
    assert_eq!(report["omitted"]["impact[0].tests"], 5);
    assert_eq!(report["shared_rules"][0]["verification"][0], "b::quote_0");
    assert_eq!(report["verification_status"], "not-run");
    assert!(
        !report["questions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|question| question["id"] == "verify-rule:pricing")
    );
}
