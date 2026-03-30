use super::*;
use chrono::NaiveDate;

fn date(value: &str) -> NaiveDate {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").unwrap()
}

#[test]
fn test_node_eol_versions() {
    let advisory = node::node_advisory_at("16.20.0", date("2026-03-28"));
    assert_eq!(advisory.status, AdvisoryStatus::Eol);
    assert!(advisory.is_warning());
    assert!(advisory.message.is_some());
    assert!(advisory.recommendation.is_some());
    assert_eq!(
        advisory.recommendation.as_deref(),
        Some("upgrade to node@24 (current LTS)")
    );
}

#[test]
fn test_node_current_lts() {
    let advisory = node::node_advisory_at("24.0.0", date("2026-03-28"));
    assert_eq!(advisory.status, AdvisoryStatus::Current);
    assert!(!advisory.is_warning());
}

#[test]
fn test_node_older_lts() {
    let advisory = node::node_advisory_at("22.0.0", date("2026-03-28"));
    assert_eq!(advisory.status, AdvisoryStatus::LtsAvailable);
    assert!(advisory.is_warning());
    assert_eq!(
        advisory.message.as_deref(),
        Some("node@22 is in maintenance mode")
    );
    assert_eq!(
        advisory.recommendation.as_deref(),
        Some("consider upgrading to node@24 (current LTS)")
    );
}

#[test]
fn test_node_maintenance_lts_before_eol() {
    let advisory = node::node_advisory_at("20.20.2", date("2026-03-28"));
    assert_eq!(advisory.status, AdvisoryStatus::LtsAvailable);
    assert!(advisory.is_warning());
    assert_eq!(
        advisory.recommendation.as_deref(),
        Some("consider upgrading to node@24 (current LTS)")
    );
}

#[test]
fn test_node_current_release_when_no_active_lts_exists() {
    let advisory = node::node_advisory_at("24.2.0", date("2025-07-01"));
    assert_eq!(advisory.status, AdvisoryStatus::Current);
    assert!(!advisory.is_warning());
}

#[test]
fn test_java_eol_versions() {
    let advisory = java::java_advisory("10.0.0");
    assert_eq!(advisory.status, AdvisoryStatus::Eol);
    assert!(advisory.is_warning());
}

#[test]
fn test_java_current_lts() {
    let advisory = java::java_advisory("21.0.0");
    assert_eq!(advisory.status, AdvisoryStatus::Current);
    assert!(!advisory.is_warning());
}

#[test]
fn test_java_older_lts() {
    let advisory = java::java_advisory("17.0.0");
    assert_eq!(advisory.status, AdvisoryStatus::LtsAvailable);
    assert!(advisory.is_warning());
}

#[test]
fn test_python_eol_versions() {
    let advisory = python::python_advisory("3.7.0");
    assert_eq!(advisory.status, AdvisoryStatus::Eol);
    assert!(advisory.is_warning());
}

#[test]
fn test_python_current() {
    let advisory = python::python_advisory("3.12.0");
    assert_eq!(advisory.status, AdvisoryStatus::Current);
    assert!(!advisory.is_warning());
}

#[test]
fn test_python2_eol() {
    let advisory = python::python_advisory("2.7.18");
    assert_eq!(advisory.status, AdvisoryStatus::Eol);
    assert!(advisory.is_warning());
}

#[test]
fn test_unsupported_tool() {
    let advisory = get_advisory("go", "1.21.0");
    assert_eq!(advisory.status, AdvisoryStatus::Unknown);
    assert!(!advisory.is_warning());
}
