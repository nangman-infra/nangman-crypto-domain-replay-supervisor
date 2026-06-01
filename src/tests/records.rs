use super::fixtures::domain;
use crate::planning::{
    ControlPlaneRunRecordInput, build_authority_migration_records, build_control_plane_run_record,
    build_workflow_command,
};
use crate::types::{
    AUTHORITY_MIGRATION_RECORD_SCHEMA_VERSION, AuthorityMigrationInput,
    CONTROL_PLANE_RUN_RECORD_SCHEMA_VERSION,
};
use std::collections::BTreeSet;

#[test]
fn control_plane_run_record_captures_commands_and_authority_migrations() {
    let changed_triggers = BTreeSet::from(["scoring_policy_version_changed".to_owned()]);
    let command = build_workflow_command(
        "supervisor_run_001",
        &domain(),
        &changed_triggers,
        3,
        1_700_000_000_000,
    );
    let migrations = build_authority_migration_records(
        &[AuthorityMigrationInput {
            authority_scope: "control-plane-lite:domain-replay-supervisor".to_owned(),
            from_app: None,
            to_app: "domain-replay-supervisor-app".to_owned(),
            migration_reason: "initial_owner".to_owned(),
            effective_at_ms: None,
        }],
        "control_run_001",
        1_700_000_000_000,
        "manifest_001",
        "checksum_001",
    );
    let record = build_control_plane_run_record(ControlPlaneRunRecordInput {
        control_plane_run_record_id: "control_run_001",
        created_at_ms: 1_700_000_000_000,
        manifest_id: "manifest_001",
        manifest_checksum: "checksum_001",
        changed_triggers: &changed_triggers,
        domains_seen: 1,
        workflow_commands: &[command],
        authority_migration_records: &migrations,
        output_report_key: Some("domain-replay-supervisor/report.json".to_owned()),
    })
    .unwrap();

    assert_eq!(
        record.schema_version,
        CONTROL_PLANE_RUN_RECORD_SCHEMA_VERSION
    );
    assert_eq!(record.domains_seen, 1);
    assert_eq!(record.domains_selected, 1);
    assert_eq!(record.workflow_command_ids.len(), 1);
    assert_eq!(record.authority_migration_record_ids.len(), 1);
    assert!(!record.checksum.is_empty());
    assert_eq!(
        migrations[0].schema_version,
        AUTHORITY_MIGRATION_RECORD_SCHEMA_VERSION
    );
    assert_eq!(migrations[0].control_plane_run_record_id, "control_run_001");
}
