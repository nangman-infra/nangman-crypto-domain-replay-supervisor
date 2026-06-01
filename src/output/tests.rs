use super::{local_output_path, write_local_outputs};
use crate::types::{
    ControlPlaneRunRecord, DomainReplaySupervisorReport, PRODUCER_APP, REPORT_SCHEMA_VERSION,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn minimal_report() -> DomainReplaySupervisorReport {
    DomainReplaySupervisorReport {
        schema_version: REPORT_SCHEMA_VERSION.to_owned(),
        supervisor_run_id: "domain_replay_supervisor_run_abc123".to_owned(),
        producer_app: PRODUCER_APP.to_owned(),
        producer_version: "0.1.0".to_owned(),
        created_at_ms: 1_700_000_000_000,
        manifest_id: "manifest_001".to_owned(),
        manifest_checksum: "checksum_001".to_owned(),
        changed_triggers: Vec::new(),
        domains_seen: 0,
        domains_selected: 0,
        workflow_commands_created: 0,
        input_keys_estimated: 0,
        skipped_domains: Vec::new(),
        workflow_commands: Vec::new(),
        control_plane_run_record: ControlPlaneRunRecord {
            schema_version: "control_plane_run_record_v1".to_owned(),
            control_plane_run_record_id: "control_plane_run_record_abc123".to_owned(),
            producer_app: PRODUCER_APP.to_owned(),
            producer_version: "0.1.0".to_owned(),
            created_at_ms: 1_700_000_000_000,
            run_type: "domain_replay_supervisor".to_owned(),
            manifest_id: "manifest_001".to_owned(),
            manifest_checksum: "checksum_001".to_owned(),
            changed_triggers: Vec::new(),
            domains_seen: 0,
            domains_selected: 0,
            workflow_command_ids: Vec::new(),
            authority_migration_record_ids: Vec::new(),
            output_report_key: None,
            checksum: "checksum_002".to_owned(),
        },
        authority_migration_records: Vec::new(),
        report_key: None,
        checksum: "checksum_003".to_owned(),
    }
}

fn unique_output_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!(
        "domain-replay-supervisor-output-{name}-{}-{nanos}",
        std::process::id()
    ))
}

#[test]
fn local_output_path_rejects_unsafe_ids() {
    let output_dir = Path::new("/tmp/domain-replay-output");
    for id in ["", ".", "..", "../escape", "nested/id", "line\nbreak"] {
        let error = local_output_path(output_dir, id, "report.json")
            .expect_err("unsafe id must be rejected")
            .to_string();
        assert!(
            error.contains("local output id"),
            "expected local output id error for {id:?}, got {error}"
        );
    }
}

#[test]
fn write_local_outputs_rejects_relative_output_dir() {
    let error = write_local_outputs(
        &minimal_report(),
        Path::new("relative-output"),
        "domain_replay_supervisor_run_abc123",
        "control_plane_run_record_abc123",
    )
    .expect_err("relative output dir must be rejected")
    .to_string();

    assert!(error.contains("absolute path"));
}

#[test]
fn write_local_outputs_rejects_ambiguous_absolute_output_dir() {
    let output_dir = std::env::temp_dir()
        .join("..")
        .join("domain-replay-ambiguous-output");
    let error = write_local_outputs(
        &minimal_report(),
        &output_dir,
        "domain_replay_supervisor_run_abc123",
        "control_plane_run_record_abc123",
    )
    .expect_err("ambiguous output dir must be rejected")
    .to_string();

    assert!(error.contains("relative path components"));
    assert!(
        !output_dir
            .join("domain_replay_supervisor_run_abc123.report.json")
            .exists()
    );
}

#[test]
fn write_local_outputs_keeps_files_inside_output_dir() {
    let output_dir = unique_output_dir("safe");
    let files = write_local_outputs(
        &minimal_report(),
        &output_dir,
        "domain_replay_supervisor_run_abc123",
        "control_plane_run_record_abc123",
    )
    .expect("safe output ids write successfully");

    assert_eq!(files.len(), 3);
    assert!(files.iter().all(|file| {
        file.starts_with(
            output_dir
                .to_str()
                .expect("temporary output path must be utf8"),
        )
    }));
    fs::remove_dir_all(&output_dir).ok();
}

#[cfg(unix)]
#[test]
fn write_local_outputs_rejects_symlink_destination() {
    use std::os::unix::fs::symlink;

    let output_dir = unique_output_dir("symlink");
    fs::create_dir_all(&output_dir).expect("create temp output dir");
    let target_path = output_dir.join("outside-target.json");
    fs::write(&target_path, "do-not-overwrite").expect("write symlink target");
    let report_path = output_dir.join("domain_replay_supervisor_run_abc123.report.json");
    symlink(&target_path, &report_path).expect("create report output symlink");

    let error = write_local_outputs(
        &minimal_report(),
        &output_dir,
        "domain_replay_supervisor_run_abc123",
        "control_plane_run_record_abc123",
    )
    .expect_err("symlink output destination must be rejected")
    .to_string();

    assert!(error.contains("symlink"));
    assert_eq!(
        fs::read_to_string(&target_path).expect("read symlink target"),
        "do-not-overwrite"
    );
    fs::remove_dir_all(&output_dir).ok();
}
