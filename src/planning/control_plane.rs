use std::collections::BTreeSet;

use crate::keys::checksum_json;
use crate::types::{
    AuthorityMigrationRecord, CONTROL_PLANE_RUN_RECORD_SCHEMA_VERSION, ControlPlaneRunRecord,
    PRODUCER_APP, WorkflowCommand,
};
use intel_candidate_app::error::AppResult;

pub(crate) struct ControlPlaneRunRecordInput<'a> {
    pub(crate) control_plane_run_record_id: &'a str,
    pub(crate) created_at_ms: i64,
    pub(crate) manifest_id: &'a str,
    pub(crate) manifest_checksum: &'a str,
    pub(crate) changed_triggers: &'a BTreeSet<String>,
    pub(crate) domains_seen: usize,
    pub(crate) workflow_commands: &'a [WorkflowCommand],
    pub(crate) authority_migration_records: &'a [AuthorityMigrationRecord],
    pub(crate) output_report_key: Option<String>,
}

pub(crate) fn build_control_plane_run_record(
    input: ControlPlaneRunRecordInput<'_>,
) -> AppResult<ControlPlaneRunRecord> {
    let mut record = ControlPlaneRunRecord {
        schema_version: CONTROL_PLANE_RUN_RECORD_SCHEMA_VERSION.to_owned(),
        control_plane_run_record_id: input.control_plane_run_record_id.to_owned(),
        producer_app: PRODUCER_APP.to_owned(),
        producer_version: env!("CARGO_PKG_VERSION").to_owned(),
        created_at_ms: input.created_at_ms,
        run_type: "replay_repair_planning".to_owned(),
        manifest_id: input.manifest_id.to_owned(),
        manifest_checksum: input.manifest_checksum.to_owned(),
        changed_triggers: input.changed_triggers.iter().cloned().collect(),
        domains_seen: input.domains_seen,
        domains_selected: input.workflow_commands.len(),
        workflow_command_ids: input
            .workflow_commands
            .iter()
            .map(|command| command.workflow_command_id.clone())
            .collect(),
        authority_migration_record_ids: input
            .authority_migration_records
            .iter()
            .map(|record| record.authority_migration_record_id.clone())
            .collect(),
        output_report_key: input.output_report_key,
        checksum: String::new(),
    };
    record.checksum = checksum_json(&record)?;
    Ok(record)
}
