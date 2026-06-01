use super::selection::DomainSelection;
use crate::keys::checksum_json;
use crate::planning::{
    ControlPlaneRunRecordInput, build_authority_migration_records, build_control_plane_run_record,
};
use crate::types::{
    Args, DomainReplayManifest, DomainReplaySupervisorReport, PRODUCER_APP, REPORT_SCHEMA_VERSION,
    RunSummary,
};
use intel_candidate_app::error::AppResult;

pub(super) struct SupervisorReportInput<'a> {
    pub(super) manifest: &'a DomainReplayManifest,
    pub(super) manifest_checksum: &'a str,
    pub(super) args: &'a Args,
    pub(super) supervisor_run_id: &'a str,
    pub(super) control_plane_run_record_id: &'a str,
    pub(super) created_at_ms: i64,
    pub(super) selection: DomainSelection,
}

pub(super) fn build_supervisor_report(
    input: SupervisorReportInput<'_>,
) -> AppResult<DomainReplaySupervisorReport> {
    let authority_migration_records = build_authority_migration_records(
        &input.manifest.authority_migration_records,
        input.control_plane_run_record_id,
        input.created_at_ms,
        &input.manifest.manifest_id,
        input.manifest_checksum,
    );
    let control_plane_run_record = build_control_plane_run_record(ControlPlaneRunRecordInput {
        control_plane_run_record_id: input.control_plane_run_record_id,
        created_at_ms: input.created_at_ms,
        manifest_id: &input.manifest.manifest_id,
        manifest_checksum: input.manifest_checksum,
        changed_triggers: &input.args.changed_triggers,
        domains_seen: input.manifest.domains.len(),
        workflow_commands: &input.selection.workflow_commands,
        authority_migration_records: &authority_migration_records,
        output_report_key: None,
    })?;

    let mut report = DomainReplaySupervisorReport {
        schema_version: REPORT_SCHEMA_VERSION.to_owned(),
        supervisor_run_id: input.supervisor_run_id.to_owned(),
        producer_app: PRODUCER_APP.to_owned(),
        producer_version: env!("CARGO_PKG_VERSION").to_owned(),
        created_at_ms: input.created_at_ms,
        manifest_id: input.manifest.manifest_id.clone(),
        manifest_checksum: input.manifest_checksum.to_owned(),
        changed_triggers: input.args.changed_triggers.iter().cloned().collect(),
        domains_seen: input.manifest.domains.len(),
        domains_selected: input.selection.workflow_commands.len(),
        workflow_commands_created: input.selection.workflow_commands.len(),
        input_keys_estimated: input.selection.input_keys_estimated,
        skipped_domains: input.selection.skipped_domains,
        workflow_commands: input.selection.workflow_commands,
        control_plane_run_record,
        authority_migration_records,
        report_key: None,
        checksum: String::new(),
    };
    report.checksum = checksum_json(&report)?;
    Ok(report)
}

pub(super) fn build_run_summary(
    supervisor_run_id: String,
    output_files: Vec<String>,
    output_s3_uris: Vec<String>,
    report: &DomainReplaySupervisorReport,
) -> RunSummary {
    RunSummary {
        supervisor_run_id,
        domains_selected: report.domains_selected,
        workflow_commands_created: report.workflow_commands_created,
        input_keys_estimated: report.input_keys_estimated,
        output_files,
        output_s3_uris,
    }
}
