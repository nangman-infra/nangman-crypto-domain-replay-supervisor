use std::collections::BTreeSet;

use crate::types::{COMMAND_SCHEMA_VERSION, DomainRuntimeSpec, WorkflowCommand};
use intel_candidate_app::hash::stable_id;

pub(crate) fn build_workflow_command(
    supervisor_run_id: &str,
    domain: &DomainRuntimeSpec,
    changed_triggers: &BTreeSet<String>,
    estimated_input_keys: usize,
    issued_at_ms: i64,
) -> WorkflowCommand {
    let target_scope = domain.input_prefixes.join(",");
    let reason = changed_triggers
        .iter()
        .cloned()
        .collect::<Vec<_>>()
        .join(",");
    let workflow_command_id = stable_id(
        "workflow_cmd",
        &[
            supervisor_run_id,
            &domain.domain_id,
            &domain.target_app,
            &domain.target_mode,
            &target_scope,
        ],
    );
    WorkflowCommand {
        schema_version: COMMAND_SCHEMA_VERSION.to_owned(),
        workflow_command_id: workflow_command_id.clone(),
        workflow_type: "healing_action".to_owned(),
        target_app: domain.target_app.clone(),
        target_scope,
        target_mode: domain.target_mode.clone(),
        reason,
        source_run_record_id: supervisor_run_id.to_owned(),
        issued_at_ms,
        idempotency_key: stable_id("workflow_cmd_idem", &[&workflow_command_id]),
        domain_id: domain.domain_id.clone(),
        input_bucket: domain.input_bucket.clone(),
        input_prefixes: domain.input_prefixes.clone(),
        estimated_input_keys,
        command: render_command_template(domain),
    }
}

pub(crate) fn render_command_template(domain: &DomainRuntimeSpec) -> Vec<String> {
    let mut rendered = Vec::new();
    for part in &domain.command_template {
        if part == "{repeat_input_prefixes}" {
            for prefix in &domain.input_prefixes {
                rendered.push("--replay-input-prefix".to_owned());
                rendered.push(prefix.clone());
            }
            continue;
        }
        rendered.push(
            part.replace("{input_bucket}", &domain.input_bucket)
                .replace("{output_bucket}", &domain.output_bucket)
                .replace("{input_prefixes}", &domain.input_prefixes.join(","))
                .replace("{output_report_prefix}", &domain.output_report_prefix),
        );
    }
    rendered
}

pub(crate) fn domain_matches_triggers(
    domain: &DomainRuntimeSpec,
    changed_triggers: &BTreeSet<String>,
) -> bool {
    if changed_triggers.contains("force_all") {
        return true;
    }
    let replay_triggers = domain
        .replay_triggers
        .iter()
        .map(|value| value.as_str())
        .collect::<BTreeSet<_>>();
    changed_triggers
        .iter()
        .any(|trigger| replay_triggers.contains(trigger.as_str()))
}
