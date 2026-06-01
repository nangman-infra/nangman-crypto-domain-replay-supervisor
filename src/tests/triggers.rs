use super::fixtures::domain;
use crate::planning::domain_matches_triggers;
use std::collections::BTreeSet;

#[test]
fn trigger_matching_allows_force_all() {
    let domain = domain();
    assert!(domain_matches_triggers(
        &domain,
        &BTreeSet::from(["force_all".to_owned()])
    ));
}

#[test]
fn trigger_matching_requires_overlap() {
    let domain = domain();
    assert!(domain_matches_triggers(
        &domain,
        &BTreeSet::from(["scoring_policy_version_changed".to_owned()])
    ));
    assert!(!domain_matches_triggers(
        &domain,
        &BTreeSet::from(["unrelated".to_owned()])
    ));
}
