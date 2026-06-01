use crate::types::{AuthorityMigrationRecord, DomainReplaySupervisorReport};
use intel_candidate_app::error::AppResult;

pub(super) fn commands_jsonl(report: &DomainReplaySupervisorReport) -> AppResult<String> {
    let commands = report
        .workflow_commands
        .iter()
        .map(serde_json::to_string)
        .collect::<Result<Vec<_>, _>>()?
        .join("\n");
    Ok(format!("{commands}\n"))
}

pub(super) fn authority_migrations_jsonl(
    records: &[AuthorityMigrationRecord],
) -> AppResult<String> {
    let migrations = records
        .iter()
        .map(serde_json::to_string)
        .collect::<Result<Vec<_>, _>>()?
        .join("\n");
    Ok(format!("{migrations}\n"))
}
