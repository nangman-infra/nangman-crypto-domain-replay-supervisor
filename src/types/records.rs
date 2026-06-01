use serde::Serialize;

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
