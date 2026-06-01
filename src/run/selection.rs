use crate::keys::log_event;
use crate::planning::{build_workflow_command, domain_matches_triggers, estimate_input_keys};
use crate::types::{Args, DomainReplayManifest, SkippedDomain, WorkflowCommand};
use intel_candidate_app::error::AppResult;
use serde_json::json;

pub(super) struct DomainSelection {
    pub(super) skipped_domains: Vec<SkippedDomain>,
    pub(super) workflow_commands: Vec<WorkflowCommand>,
    pub(super) input_keys_estimated: usize,
}

pub(super) async fn select_domains(
    manifest: &DomainReplayManifest,
    args: &Args,
    supervisor_run_id: &str,
    created_at_ms: i64,
) -> AppResult<DomainSelection> {
    let mut skipped_domains = Vec::new();
    let mut workflow_commands = Vec::new();
    let mut input_keys_estimated = 0usize;

    for domain in &manifest.domains {
        if !domain_matches_triggers(domain, &args.changed_triggers) {
            skipped_domains.push(SkippedDomain {
                domain_id: domain.domain_id.clone(),
                reason: "trigger_not_matched".to_owned(),
            });
            continue;
        }
        let estimated_keys = estimate_input_keys(domain).await?;
        input_keys_estimated += estimated_keys;
        let command = build_workflow_command(
            supervisor_run_id,
            domain,
            &args.changed_triggers,
            estimated_keys,
            created_at_ms,
        );
        log_event(
            "workflow_command_created",
            json!({
                "supervisor_run_id": supervisor_run_id,
                "domain_id": domain.domain_id,
                "target_app": domain.target_app,
                "target_mode": domain.target_mode,
                "estimated_input_keys": estimated_keys,
                "workflow_command_id": command.workflow_command_id,
            }),
        );
        workflow_commands.push(command);
    }

    Ok(DomainSelection {
        skipped_domains,
        workflow_commands,
        input_keys_estimated,
    })
}
