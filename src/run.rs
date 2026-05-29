use super::*;

pub(crate) async fn run(args: Args) -> AppResult<RunSummary> {
    let manifest_bytes = fs::read(&args.manifest_file).map_err(|error| {
        AppError::config(format!(
            "read manifest {}: {error}",
            args.manifest_file.display()
        ))
    })?;
    let manifest: DomainReplayManifest = serde_json::from_slice(&manifest_bytes)?;
    validate_manifest(&manifest)?;
    let manifest_checksum = sha256_hex(&manifest_bytes);
    let created_at_ms = args.now_ms.unwrap_or_else(now_ms);
    let supervisor_run_id = stable_id(
        "domain_replay_supervisor_run",
        &[
            &manifest.manifest_id,
            &manifest_checksum,
            &created_at_ms.to_string(),
        ],
    );
    log_event(
        "supervisor_started",
        json!({
            "supervisor_run_id": supervisor_run_id,
            "manifest_id": manifest.manifest_id,
            "changed_triggers": args.changed_triggers,
        }),
    );

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
            &supervisor_run_id,
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

    let control_plane_run_record_id = stable_id("control_plane_run_record", &[&supervisor_run_id]);
    let authority_migration_records = build_authority_migration_records(
        &manifest.authority_migration_records,
        &control_plane_run_record_id,
        created_at_ms,
        &manifest.manifest_id,
        &manifest_checksum,
    );
    let control_plane_run_record = build_control_plane_run_record(ControlPlaneRunRecordInput {
        control_plane_run_record_id: &control_plane_run_record_id,
        created_at_ms,
        manifest_id: &manifest.manifest_id,
        manifest_checksum: &manifest_checksum,
        changed_triggers: &args.changed_triggers,
        domains_seen: manifest.domains.len(),
        workflow_commands: &workflow_commands,
        authority_migration_records: &authority_migration_records,
        output_report_key: None,
    })?;

    let mut report = DomainReplaySupervisorReport {
        schema_version: REPORT_SCHEMA_VERSION.to_owned(),
        supervisor_run_id: supervisor_run_id.clone(),
        producer_app: PRODUCER_APP.to_owned(),
        producer_version: env!("CARGO_PKG_VERSION").to_owned(),
        created_at_ms,
        manifest_id: manifest.manifest_id,
        manifest_checksum,
        changed_triggers: args.changed_triggers.iter().cloned().collect(),
        domains_seen: manifest.domains.len(),
        domains_selected: workflow_commands.len(),
        workflow_commands_created: workflow_commands.len(),
        input_keys_estimated,
        skipped_domains,
        workflow_commands,
        control_plane_run_record,
        authority_migration_records,
        report_key: None,
        checksum: String::new(),
    };
    report.checksum = checksum_json(&report)?;

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
    Ok(RunSummary {
        supervisor_run_id,
        domains_selected: report.domains_selected,
        workflow_commands_created: report.workflow_commands_created,
        input_keys_estimated: report.input_keys_estimated,
        output_files,
        output_s3_uris,
    })
}
