//! Decision outcomes for the model: repeatable evidence, unknowns, and change intent.

use std::{fs, path::Path, process::Command};

fn fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("provider.rs"),
        "pub fn value() -> u32 { 1 }\n",
    )
    .unwrap();
    fs::write(dir.path().join("consumer.rs"), "pub fn read() -> u32 { crate::provider::value() }\n#[test] fn reads_value() { assert_eq!(read(), 1); }\n").unwrap();
    fs::write(dir.path().join("unrelated.rs"), "pub fn other() {}\n").unwrap();
    fs::write(
        dir.path().join(".coupling.toml"),
        "[subdomains]\ngeneric=['provider.rs']\ncore=['consumer.rs']\n",
    )
    .unwrap();
    fs::write(dir.path().join("empty-git-config"), "").unwrap();
    dir
}

fn review(root: &Path, args: &[&str], threads: &str) -> Vec<u8> {
    let output = Command::new(env!("CARGO_BIN_EXE_cargo-coupling"))
        .args(["coupling"])
        .args(args)
        .arg(root)
        .env("RAYON_NUM_THREADS", threads)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", root.join("empty-git-config"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

#[test]
fn generic_provider_changes_and_unknown_owners_remain_review_questions() {
    let dir = fixture();
    let report: serde_json::Value = serde_json::from_slice(&review(
        dir.path(),
        &["--agent", "--focus", "provider", "--no-git"],
        "1",
    ))
    .unwrap();
    let questions = report["questions"].as_array().unwrap();
    assert!(questions.iter().any(|q| q["id"] == "future-change:provider"
        && q["reason"].as_str().unwrap().contains("implementation")));
    assert!(questions.iter().any(|q| q["id"] == "ownership:provider"));
    assert_eq!(report["focus"], serde_json::json!(["consumer", "provider"]));
    assert!(
        report["impact"][0]["tests"]
            .as_array()
            .unwrap()
            .iter()
            .any(|test| test["item"]["name"] == "reads_value")
    );
    assert_eq!(report["verification_status"], "not-run");

    fs::write(
        dir.path().join(".coupling-context.toml"),
        r#"
version=1
[[components]]
name='gateway'
modules=['provider','consumer']
owners=['@team']
planned_changes='Replace the provider while retaining the accepted response'
[[scenarios]]
name='provider-replacement'
description='Review an implementation change without reclassifying the generic function'
[[scenarios.changes]]
source='consumer'
target='provider'
volatility=1.0
"#,
    )
    .unwrap();
    let report: serde_json::Value = serde_json::from_slice(&review(
        dir.path(),
        &["--design", "--json", "--focus", "provider", "--no-git"],
        "1",
    ))
    .unwrap();
    let design = &report["design"];
    assert!(design["questions"].as_array().unwrap().is_empty());
    let scenario = &design["scenarios"][0];
    assert!(scenario["affected_edges"].as_u64().unwrap() > 0);
    assert_eq!(scenario["current_balance"], 1.0);
    assert!(scenario["hypothetical_balance"].as_f64().unwrap() < 1.0);
    assert!(
        design["edges"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|edge| edge["target"] == "provider")
            .all(|edge| edge["volatility"] == 0.0)
    );
}

#[test]
fn review_is_repeatable_and_focus_does_not_fabricate_a_git_change() {
    let dir = fixture();
    fs::write(dir.path().join(".coupling-context.toml"), "version=1\n[[decisions]]\nid='keep'\nmodules=['provider']\nreason='Preserve the interface'\nreview_on=['changed']\n").unwrap();
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
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_CONFIG_GLOBAL", dir.path().join("empty-git-config"))
                .status()
                .unwrap()
                .success()
        );
    }
    let args = ["--agent", "--focus", "provider", "--changed-since", "HEAD"];
    let unchanged: serde_json::Value =
        serde_json::from_slice(&review(dir.path(), &args, "1")).unwrap();
    assert_eq!(unchanged["decisions"][0]["status"], "retain");
    assert!(
        unchanged["decisions"][0]["triggered"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    fs::write(
        dir.path().join("provider.rs"),
        "pub fn value() -> u32 { 2 }\n",
    )
    .unwrap();
    let single_thread = review(dir.path(), &args, "1");
    for threads in ["4", "1", "4"] {
        assert_eq!(
            single_thread,
            review(dir.path(), &args, threads),
            "identical source, context and Git observations must produce identical agent JSON"
        );
    }
    let changed: serde_json::Value = serde_json::from_slice(&single_thread).unwrap();
    assert_eq!(changed["decisions"][0]["status"], "review");
    assert_eq!(
        changed["decisions"][0]["triggered"],
        serde_json::json!(["Changed"])
    );
    assert_eq!(
        changed["focus"],
        serde_json::json!(["consumer", "provider"])
    );
}
