use crate::keys::{
    authority_migration_records_key, checksum_json, control_plane_run_record_key, help_text,
    supervisor_commands_key, supervisor_report_key,
};
use crate::types::{
    AUTHORITY_MIGRATION_RECORD_SCHEMA_VERSION, CONTROL_PLANE_RUN_RECORD_SCHEMA_VERSION,
    REPORT_SCHEMA_VERSION,
};

#[test]
fn supervisor_output_keys_are_partitioned_and_prefix_normalized() {
    assert_eq!(
        supervisor_report_key("/domain-replay-supervisor/", 7_200_000, "run_001"),
        format!(
            "domain-replay-supervisor/schema={REPORT_SCHEMA_VERSION}/dt=1970-01-01/hour=02/supervisor_run_id=run_001/report.json"
        )
    );
    assert_eq!(
        supervisor_commands_key("/domain-replay-supervisor/", 7_200_000, "run_001"),
        format!(
            "domain-replay-supervisor/schema={REPORT_SCHEMA_VERSION}/dt=1970-01-01/hour=02/supervisor_run_id=run_001/commands.jsonl"
        )
    );
}

#[test]
fn control_plane_keys_use_their_own_schema_partition() {
    assert_eq!(
        control_plane_run_record_key("control-plane", 7_200_000, "control_run_001"),
        format!(
            "control-plane/schema={CONTROL_PLANE_RUN_RECORD_SCHEMA_VERSION}/dt=1970-01-01/hour=02/control_plane_run_record_id=control_run_001/record.json"
        )
    );
    assert_eq!(
        authority_migration_records_key("control-plane", 7_200_000, "control_run_001"),
        format!(
            "control-plane/schema={AUTHORITY_MIGRATION_RECORD_SCHEMA_VERSION}/dt=1970-01-01/hour=02/control_plane_run_record_id=control_run_001/part-000001.jsonl"
        )
    );
}

#[test]
fn checksum_json_is_stable_for_same_payload() {
    let one = checksum_json(&serde_json::json!({"domain": "intel-candidate", "count": 1})).unwrap();
    let two = checksum_json(&serde_json::json!({"domain": "intel-candidate", "count": 1})).unwrap();

    assert_eq!(one, two);
    assert_eq!(one.len(), 64);
}

#[test]
fn help_text_names_required_contract_inputs() {
    let text = help_text();

    assert!(text.contains("domain-replay-supervisor-app"));
    assert!(text.contains("--manifest-file"));
    assert!(text.contains("--changed-trigger"));
    assert!(text.contains("--output-s3-bucket"));
}
