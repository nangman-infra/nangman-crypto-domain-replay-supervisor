use super::command::WorkflowCommand;
use super::records::{AuthorityMigrationRecord, ControlPlaneRunRecord};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct DomainReplaySupervisorReport {
    pub(crate) schema_version: String,
    pub(crate) supervisor_run_id: String,
    pub(crate) producer_app: String,
    pub(crate) producer_version: String,
    pub(crate) created_at_ms: i64,
    pub(crate) manifest_id: String,
    pub(crate) manifest_checksum: String,
    pub(crate) changed_triggers: Vec<String>,
    pub(crate) domains_seen: usize,
    pub(crate) domains_selected: usize,
    pub(crate) workflow_commands_created: usize,
    pub(crate) input_keys_estimated: usize,
    pub(crate) skipped_domains: Vec<SkippedDomain>,
    pub(crate) workflow_commands: Vec<WorkflowCommand>,
    pub(crate) control_plane_run_record: ControlPlaneRunRecord,
    pub(crate) authority_migration_records: Vec<AuthorityMigrationRecord>,
    pub(crate) report_key: Option<String>,
    pub(crate) checksum: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct SkippedDomain {
    pub(crate) domain_id: String,
    pub(crate) reason: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct RunSummary {
    pub(crate) supervisor_run_id: String,
    pub(crate) domains_selected: usize,
    pub(crate) workflow_commands_created: usize,
    pub(crate) input_keys_estimated: usize,
    pub(crate) output_files: Vec<String>,
    pub(crate) output_s3_uris: Vec<String>,
}
