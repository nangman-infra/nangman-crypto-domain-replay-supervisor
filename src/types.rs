use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::PathBuf;

pub(crate) const MANIFEST_SCHEMA_VERSION: &str = "domain_replay_manifest_v1";
pub(crate) const COMMAND_SCHEMA_VERSION: &str = "workflow_command_v1";
pub(crate) const REPORT_SCHEMA_VERSION: &str = "domain_replay_supervisor_report_v1";
pub(crate) const CONTROL_PLANE_RUN_RECORD_SCHEMA_VERSION: &str = "control_plane_run_record_v1";
pub(crate) const AUTHORITY_MIGRATION_RECORD_SCHEMA_VERSION: &str = "authority_migration_record_v1";
pub(crate) const PRODUCER_APP: &str = "domain-replay-supervisor-app";
pub(crate) const DEFAULT_AWS_REGION: &str = "ap-northeast-2";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Args {
    pub(crate) manifest_file: PathBuf,
    pub(crate) changed_triggers: BTreeSet<String>,
    pub(crate) output_dir: Option<PathBuf>,
    pub(crate) output_s3: Option<S3OutputArgs>,
    pub(crate) now_ms: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct S3OutputArgs {
    pub(crate) bucket: String,
    pub(crate) region: String,
    pub(crate) prefix: String,
    pub(crate) profile: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct DomainReplayManifest {
    pub(crate) schema_version: String,
    pub(crate) manifest_id: String,
    pub(crate) domains: Vec<DomainRuntimeSpec>,
    #[serde(default)]
    pub(crate) authority_migration_records: Vec<AuthorityMigrationInput>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct DomainRuntimeSpec {
    pub(crate) domain_id: String,
    pub(crate) target_app: String,
    pub(crate) target_mode: String,
    pub(crate) input_bucket: String,
    #[serde(default = "default_region")]
    pub(crate) input_region: String,
    #[serde(default)]
    pub(crate) input_profile: Option<String>,
    pub(crate) input_prefixes: Vec<String>,
    #[serde(default)]
    pub(crate) input_suffixes: Vec<String>,
    #[serde(default = "default_max_keys")]
    pub(crate) max_keys_per_prefix: usize,
    #[serde(default)]
    pub(crate) replay_triggers: Vec<String>,
    pub(crate) output_bucket: String,
    #[serde(default = "default_region")]
    pub(crate) output_region: String,
    pub(crate) output_report_prefix: String,
    pub(crate) command_template: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct WorkflowCommand {
    pub(crate) schema_version: String,
    pub(crate) workflow_command_id: String,
    pub(crate) workflow_type: String,
    pub(crate) target_app: String,
    pub(crate) target_scope: String,
    pub(crate) target_mode: String,
    pub(crate) reason: String,
    pub(crate) source_run_record_id: String,
    pub(crate) issued_at_ms: i64,
    pub(crate) idempotency_key: String,
    pub(crate) domain_id: String,
    pub(crate) input_bucket: String,
    pub(crate) input_prefixes: Vec<String>,
    pub(crate) estimated_input_keys: usize,
    pub(crate) command: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct AuthorityMigrationInput {
    pub(crate) authority_scope: String,
    #[serde(default)]
    pub(crate) from_app: Option<String>,
    pub(crate) to_app: String,
    pub(crate) migration_reason: String,
    #[serde(default)]
    pub(crate) effective_at_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct AuthorityMigrationRecord {
    pub(crate) schema_version: String,
    pub(crate) authority_migration_record_id: String,
    pub(crate) control_plane_run_record_id: String,
    pub(crate) authority_scope: String,
    pub(crate) from_app: Option<String>,
    pub(crate) to_app: String,
    pub(crate) migration_reason: String,
    pub(crate) effective_at_ms: i64,
    pub(crate) created_at_ms: i64,
    pub(crate) manifest_id: String,
    pub(crate) manifest_checksum: String,
    pub(crate) idempotency_key: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct ControlPlaneRunRecord {
    pub(crate) schema_version: String,
    pub(crate) control_plane_run_record_id: String,
    pub(crate) producer_app: String,
    pub(crate) producer_version: String,
    pub(crate) created_at_ms: i64,
    pub(crate) run_type: String,
    pub(crate) manifest_id: String,
    pub(crate) manifest_checksum: String,
    pub(crate) changed_triggers: Vec<String>,
    pub(crate) domains_seen: usize,
    pub(crate) domains_selected: usize,
    pub(crate) workflow_command_ids: Vec<String>,
    pub(crate) authority_migration_record_ids: Vec<String>,
    pub(crate) output_report_key: Option<String>,
    pub(crate) checksum: String,
}

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

pub(crate) fn default_region() -> String {
    DEFAULT_AWS_REGION.to_owned()
}

pub(crate) fn default_max_keys() -> usize {
    10_000
}
