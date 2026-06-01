use crate::keys::log_event;
use crate::output::{write_local_outputs, write_s3_outputs};
use crate::types::{Args, RunSummary};
use intel_candidate_app::error::AppResult;
use intel_candidate_app::hash::stable_id;
use intel_candidate_app::time::now_ms;
use serde_json::json;

mod manifest;
mod report;
mod selection;

use manifest::load_manifest;
use report::{SupervisorReportInput, build_run_summary, build_supervisor_report};
use selection::select_domains;

pub(crate) async fn run(args: Args) -> AppResult<RunSummary> {
    let loaded = load_manifest(&args)?;
    let created_at_ms = args.now_ms.unwrap_or_else(now_ms);
    let supervisor_run_id = stable_id(
        "domain_replay_supervisor_run",
        &[
            &loaded.manifest.manifest_id,
            &loaded.manifest_checksum,
            &created_at_ms.to_string(),
        ],
    );
    log_event(
        "supervisor_started",
        json!({
            "supervisor_run_id": supervisor_run_id,
            "manifest_id": loaded.manifest.manifest_id,
            "changed_triggers": args.changed_triggers,
        }),
    );

    let selection =
        select_domains(&loaded.manifest, &args, &supervisor_run_id, created_at_ms).await?;
    let control_plane_run_record_id = stable_id("control_plane_run_record", &[&supervisor_run_id]);
    let mut report = build_supervisor_report(SupervisorReportInput {
        manifest: &loaded.manifest,
        manifest_checksum: &loaded.manifest_checksum,
        args: &args,
        supervisor_run_id: &supervisor_run_id,
        control_plane_run_record_id: &control_plane_run_record_id,
        created_at_ms,
        selection,
    })?;

    let mut output_files = Vec::new();
    if let Some(output_dir) = args.output_dir.as_deref() {
        output_files = write_local_outputs(
            &report,
            output_dir,
            &supervisor_run_id,
            &control_plane_run_record_id,
        )?;
    }

    let mut output_s3_uris = Vec::new();
    if let Some(s3) = args.output_s3.as_ref() {
        output_s3_uris = write_s3_outputs(
            s3,
            &mut report,
            created_at_ms,
            &supervisor_run_id,
            &control_plane_run_record_id,
        )
        .await?;
    }

    log_event(
        "supervisor_finished",
        json!({
            "supervisor_run_id": supervisor_run_id,
            "domains_selected": report.domains_selected,
            "workflow_commands_created": report.workflow_commands_created,
            "input_keys_estimated": report.input_keys_estimated,
        }),
    );
    Ok(build_run_summary(
        supervisor_run_id,
        output_files,
        output_s3_uris,
        &report,
    ))
}
