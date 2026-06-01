use super::fixtures::domain;
use crate::planning::validate_manifest;
use crate::types::{AuthorityMigrationInput, DomainReplayManifest, MANIFEST_SCHEMA_VERSION};

fn manifest() -> DomainReplayManifest {
    DomainReplayManifest {
        schema_version: MANIFEST_SCHEMA_VERSION.to_owned(),
        manifest_id: "manifest_001".to_owned(),
        domains: vec![domain()],
        authority_migration_records: vec![AuthorityMigrationInput {
            authority_scope: "control-plane-lite:domain-replay-supervisor".to_owned(),
            from_app: Some("legacy-supervisor".to_owned()),
            to_app: "domain-replay-supervisor-app".to_owned(),
            migration_reason: "centralized_replay_authority".to_owned(),
            effective_at_ms: Some(7_200_000),
        }],
    }
}

#[test]
fn manifest_validation_accepts_complete_manifest() {
    validate_manifest(&manifest()).expect("complete manifest should validate");
}

#[test]
fn manifest_validation_rejects_top_level_contract_errors() {
    let mut wrong_schema = manifest();
    wrong_schema.schema_version = "wrong".to_owned();
    assert!(
        validate_manifest(&wrong_schema)
            .unwrap_err()
            .to_string()
            .contains("schema_version")
    );

    let mut missing_id = manifest();
    missing_id.manifest_id = " ".to_owned();
    assert!(
        validate_manifest(&missing_id)
            .unwrap_err()
            .to_string()
            .contains("manifest_id")
    );

    let mut empty_domains = manifest();
    empty_domains.domains.clear();
    assert!(
        validate_manifest(&empty_domains)
            .unwrap_err()
            .to_string()
            .contains("at least one domain")
    );
}

#[test]
fn manifest_validation_rejects_incomplete_domain_specs() {
    let mut manifest = manifest();
    manifest.domains[0].input_prefixes.clear();

    let error = validate_manifest(&manifest).unwrap_err().to_string();

    assert!(error.contains("missing required fields"));
    assert!(error.contains("intel-candidate"));
}

#[test]
fn manifest_validation_rejects_incomplete_authority_migrations() {
    let mut missing_scope = manifest();
    missing_scope.authority_migration_records[0].authority_scope = " ".to_owned();
    assert!(
        validate_manifest(&missing_scope)
            .unwrap_err()
            .to_string()
            .contains("authority_migration_records require")
    );

    let mut negative_effective_at = manifest();
    negative_effective_at.authority_migration_records[0].effective_at_ms = Some(-1);
    assert!(
        validate_manifest(&negative_effective_at)
            .unwrap_err()
            .to_string()
            .contains("effective_at_ms must be non-negative")
    );
}
