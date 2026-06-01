use serde::Serialize;

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
