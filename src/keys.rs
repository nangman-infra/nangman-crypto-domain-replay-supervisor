use super::*;

pub(crate) fn supervisor_report_key(
    prefix: &str,
    created_at_ms: i64,
    supervisor_run_id: &str,
) -> String {
    partitioned_key(prefix, created_at_ms, supervisor_run_id, "report.json")
}

pub(crate) fn supervisor_commands_key(
    prefix: &str,
    created_at_ms: i64,
    supervisor_run_id: &str,
) -> String {
    partitioned_key(prefix, created_at_ms, supervisor_run_id, "commands.jsonl")
}

pub(crate) fn control_plane_run_record_key(
    prefix: &str,
    created_at_ms: i64,
    control_plane_run_record_id: &str,
) -> String {
    let part = time_part(created_at_ms);
    format!(
        "{}/schema={}/dt={}/hour={:02}/control_plane_run_record_id={}/record.json",
        prefix.trim_matches('/'),
        CONTROL_PLANE_RUN_RECORD_SCHEMA_VERSION,
        part.event_date,
        part.hour,
        path_segment(control_plane_run_record_id)
    )
}

pub(crate) fn authority_migration_records_key(
    prefix: &str,
    created_at_ms: i64,
    control_plane_run_record_id: &str,
) -> String {
    let part = time_part(created_at_ms);
    format!(
        "{}/schema={}/dt={}/hour={:02}/control_plane_run_record_id={}/part-000001.jsonl",
        prefix.trim_matches('/'),
        AUTHORITY_MIGRATION_RECORD_SCHEMA_VERSION,
        part.event_date,
        part.hour,
        path_segment(control_plane_run_record_id)
    )
}

pub(crate) fn partitioned_key(
    prefix: &str,
    created_at_ms: i64,
    supervisor_run_id: &str,
    leaf: &str,
) -> String {
    let part = time_part(created_at_ms);
    format!(
        "{}/schema={}/dt={}/hour={:02}/supervisor_run_id={}/{}",
        prefix.trim_matches('/'),
        REPORT_SCHEMA_VERSION,
        part.event_date,
        part.hour,
        path_segment(supervisor_run_id),
        leaf
    )
}

pub(crate) fn checksum_json<T: Serialize>(value: &T) -> AppResult<String> {
    Ok(sha256_hex(serde_json::to_vec(value)?))
}

pub(crate) fn log_event(event: &str, fields: serde_json::Value) {
    let mut value = json!({
        "schema_version": "domain_replay_supervisor_log_v1",
        "producer_app": PRODUCER_APP,
        "timestamp_ms": now_ms(),
        "level": "info",
        "event": event,
    });
    if let (Some(target), Some(source)) = (value.as_object_mut(), fields.as_object()) {
        for (key, field_value) in source {
            target.insert(key.clone(), field_value.clone());
        }
    }
    println!("{value}");
}

pub(crate) fn print_help() {
    println!("{}", help_text());
}

pub(crate) fn help_text() -> &'static str {
    r#"domain-replay-supervisor-app
Usage:
  domain-replay-supervisor-app \
    --manifest-file /Volumes/WD/Developments/nangman-crypto/domains/runtime/domain-replay-manifest.dev.json \
    --changed-trigger scoring_policy_version_changed \
    --output-dir /Volumes/WD/Developments/nangman-crypto/data/replay-supervisor \
    --output-s3-bucket nangman-crypto-dev-control-plane-<account-suffix> \
    --output-s3-prefix domain-replay-supervisor

This app does not implement domain logic. It scans durable input prefixes,
creates workflow_command records, and writes a durable supervisor report."#
}

#[allow(dead_code)]
fn _assert_absolute_path(_: &Path) {}
