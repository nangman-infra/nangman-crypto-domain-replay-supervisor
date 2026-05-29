use super::*;

fn domain() -> DomainRuntimeSpec {
    DomainRuntimeSpec {
        domain_id: "intel-candidate".to_owned(),
        target_app: "intel-candidate-app".to_owned(),
        target_mode: "replay".to_owned(),
        input_bucket: "input-bucket".to_owned(),
        input_region: "ap-northeast-2".to_owned(),
        input_profile: None,
        input_prefixes: vec![
            "structured-intel-packet/schema=structured_intel_packet_v1/".to_owned(),
        ],
        input_suffixes: vec![".json".to_owned(), ".jsonl".to_owned()],
        max_keys_per_prefix: 100,
        replay_triggers: vec!["scoring_policy_version_changed".to_owned()],
        output_bucket: "output-bucket".to_owned(),
        output_region: "ap-northeast-2".to_owned(),
        output_report_prefix: "candidate-replay-report".to_owned(),
        command_template: vec![
            "/usr/local/bin/intel-candidate-replay-worker".to_owned(),
            "--input-s3-bucket".to_owned(),
            "{input_bucket}".to_owned(),
            "--output-s3-bucket".to_owned(),
            "{output_bucket}".to_owned(),
            "{repeat_input_prefixes}".to_owned(),
        ],
    }
}

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

#[test]
fn command_template_is_rendered() {
    let command = render_command_template(&domain());
    assert_eq!(
        command,
        vec![
            "/usr/local/bin/intel-candidate-replay-worker",
            "--input-s3-bucket",
            "input-bucket",
            "--output-s3-bucket",
            "output-bucket",
            "--replay-input-prefix",
            "structured-intel-packet/schema=structured_intel_packet_v1/"
        ]
    );
}

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
