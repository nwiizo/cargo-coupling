//! Invalid-input coverage for the unconfirmed analyzer panic report (#97).

use std::fs;

use cargo_coupling::analyzer::{
    AnalyzerError, CouplingAnalyzer, analyze_rust_file, analyze_rust_file_full,
};

#[test]
fn malformed_source_returns_a_parse_error() {
    for code in [
        "pub fn broken( {",
        "fn f() { let s = \"",
        "struct @Invalid;",
    ] {
        let mut analyzer = CouplingAnalyzer::new("broken".into(), "broken.rs".into());

        assert!(matches!(
            analyzer.analyze_file(code),
            Err(AnalyzerError::ParseError(message)) if !message.is_empty()
        ));
    }
}

#[test]
fn file_analysis_propagates_parse_errors() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("broken.rs");
    fs::write(&path, "pub fn broken( {").unwrap();

    assert!(matches!(
        analyze_rust_file(&path),
        Err(AnalyzerError::ParseError(_))
    ));
    assert!(matches!(
        analyze_rust_file_full(&path),
        Err(AnalyzerError::ParseError(_))
    ));
}

#[test]
fn file_analysis_propagates_invalid_utf8() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("broken.rs");
    fs::write(&path, [0xff, 0xfe]).unwrap();

    assert!(matches!(
        analyze_rust_file(&path),
        Err(AnalyzerError::IoError(error)) if error.kind() == std::io::ErrorKind::InvalidData
    ));
    assert!(matches!(
        analyze_rust_file_full(&path),
        Err(AnalyzerError::IoError(error)) if error.kind() == std::io::ErrorKind::InvalidData
    ));
}
