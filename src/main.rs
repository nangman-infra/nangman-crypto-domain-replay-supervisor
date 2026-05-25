use intel_candidate_app::error::{AppError, AppResult};
use intel_candidate_app::hash::{sha256_hex, stable_id};
use intel_candidate_app::storage::{ObjectStore, ObjectStoreConfig};
use intel_candidate_app::time::{now_ms, path_segment, time_part};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const MANIFEST_SCHEMA_VERSION: &str = "domain_replay_manifest_v1";
const COMMAND_SCHEMA_VERSION: &str = "workflow_command_v1";
const REPORT_SCHEMA_VERSION: &str = "domain_replay_supervisor_report_v1";
const CONTROL_PLANE_RUN_RECORD_SCHEMA_VERSION: &str = "control_plane_run_record_v1";
const AUTHORITY_MIGRATION_RECORD_SCHEMA_VERSION: &str = "authority_migration_record_v1";
const PRODUCER_APP: &str = "domain-replay-supervisor-app";
const DEFAULT_AWS_REGION: &str = "ap-northeast-2";

#[derive(Debug, Clone, PartialEq, Eq)]
struct Args {
    manifest_file: PathBuf,
    changed_triggers: BTreeSet<String>,
    output_dir: Option<PathBuf>,
    output_s3: Option<S3OutputArgs>,
    now_ms: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct S3OutputArgs {
    bucket: String,
    region: String,
    prefix: String,
    profile: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct DomainReplayManifest {
    schema_version: String,
    manifest_id: String,
    domains: Vec<DomainRuntimeSpec>,
    #[serde(default)]
    authority_migration_records: Vec<AuthorityMigrationInput>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct DomainRuntimeSpec {
    domain_id: String,
    target_app: String,
    target_mode: String,
    input_bucket: String,
    #[serde(default = "default_region")]
    input_region: String,
    #[serde(default)]
    input_profile: Option<String>,
    input_prefixes: Vec<String>,
    #[serde(default)]
    input_suffixes: Vec<String>,
    #[serde(default = "default_max_keys")]
    max_keys_per_prefix: usize,
    #[serde(default)]
    replay_triggers: Vec<String>,
    output_bucket: String,
    #[serde(default = "default_region")]
    output_region: String,
    output_report_prefix: String,
    command_template: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct WorkflowCommand {
    schema_version: String,
    workflow_command_id: String,
    workflow_type: String,
    target_app: String,
    target_scope: String,
    target_mode: String,
    reason: String,
    source_run_record_id: String,
    issued_at_ms: i64,
    idempotency_key: String,
    domain_id: String,
    input_bucket: String,
    input_prefixes: Vec<String>,
    estimated_input_keys: usize,
    command: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct AuthorityMigrationInput {
    authority_scope: String,
    #[serde(default)]
    from_app: Option<String>,
    to_app: String,
    migration_reason: String,
    #[serde(default)]
    effective_at_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct AuthorityMigrationRecord {
    schema_version: String,
    authority_migration_record_id: String,
    control_plane_run_record_id: String,
    authority_scope: String,
    from_app: Option<String>,
    to_app: String,
    migration_reason: String,
    effective_at_ms: i64,
    created_at_ms: i64,
    manifest_id: String,
    manifest_checksum: String,
    idempotency_key: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct ControlPlaneRunRecord {
    schema_version: String,
    control_plane_run_record_id: String,
    producer_app: String,
    producer_version: String,
    created_at_ms: i64,
    run_type: String,
    manifest_id: String,
    manifest_checksum: String,
    changed_triggers: Vec<String>,
    domains_seen: usize,
    domains_selected: usize,
    workflow_command_ids: Vec<String>,
    authority_migration_record_ids: Vec<String>,
    output_report_key: Option<String>,
    checksum: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct DomainReplaySupervisorReport {
    schema_version: String,
    supervisor_run_id: String,
    producer_app: String,
    producer_version: String,
    created_at_ms: i64,
    manifest_id: String,
    manifest_checksum: String,
    changed_triggers: Vec<String>,
    domains_seen: usize,
    domains_selected: usize,
    workflow_commands_created: usize,
    input_keys_estimated: usize,
    skipped_domains: Vec<SkippedDomain>,
    workflow_commands: Vec<WorkflowCommand>,
    control_plane_run_record: ControlPlaneRunRecord,
    authority_migration_records: Vec<AuthorityMigrationRecord>,
    report_key: Option<String>,
    checksum: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct SkippedDomain {
    domain_id: String,
    reason: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct RunSummary {
    supervisor_run_id: String,
    domains_selected: usize,
    workflow_commands_created: usize,
    input_keys_estimated: usize,
    output_files: Vec<String>,
    output_s3_uris: Vec<String>,
}

#[tokio::main]
async fn main() {
    let result = match parse_args(env::args().skip(1)) {
        Ok(Some(args)) => run(args).await,
        Ok(None) => {
            print_help();
            Ok(RunSummary {
                supervisor_run_id: String::new(),
                domains_selected: 0,
                workflow_commands_created: 0,
                input_keys_estimated: 0,
                output_files: Vec::new(),
                output_s3_uris: Vec::new(),
            })
        }
        Err(error) => Err(error),
    };
    match result {
        Ok(summary) => {
            if !summary.supervisor_run_id.is_empty() {
                println!("{}", serde_json::to_string_pretty(&summary).unwrap());
            }
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

async fn run(args: Args) -> AppResult<RunSummary> {
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
    });

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
    report.checksum = checksum_json(&report);

    let mut output_files = Vec::new();
    if let Some(output_dir) = args.output_dir.as_deref() {
        fs::create_dir_all(output_dir).map_err(|error| {
            AppError::config(format!(
                "create output dir {}: {error}",
                output_dir.display()
            ))
        })?;
        let report_path = output_dir.join(format!("{supervisor_run_id}.report.json"));
        let commands_path = output_dir.join(format!("{supervisor_run_id}.commands.jsonl"));
        fs::write(&report_path, serde_json::to_vec_pretty(&report)?).map_err(|error| {
            AppError::config(format!("write report {}: {error}", report_path.display()))
        })?;
        let commands_jsonl = report
            .workflow_commands
            .iter()
            .map(serde_json::to_string)
            .collect::<Result<Vec<_>, _>>()?
            .join("\n");
        fs::write(&commands_path, format!("{commands_jsonl}\n")).map_err(|error| {
            AppError::config(format!(
                "write commands {}: {error}",
                commands_path.display()
            ))
        })?;
        let run_record_path = output_dir.join(format!(
            "{control_plane_run_record_id}.control-plane-run-record.json"
        ));
        fs::write(
            &run_record_path,
            serde_json::to_vec_pretty(&report.control_plane_run_record)?,
        )
        .map_err(|error| {
            AppError::config(format!(
                "write control-plane run record {}: {error}",
                run_record_path.display()
            ))
        })?;
        output_files.push(report_path.display().to_string());
        output_files.push(commands_path.display().to_string());
        output_files.push(run_record_path.display().to_string());
        if !report.authority_migration_records.is_empty() {
            let migrations_path = output_dir.join(format!(
                "{control_plane_run_record_id}.authority-migrations.jsonl"
            ));
            let migrations_jsonl = report
                .authority_migration_records
                .iter()
                .map(serde_json::to_string)
                .collect::<Result<Vec<_>, _>>()?
                .join("\n");
            fs::write(&migrations_path, format!("{migrations_jsonl}\n")).map_err(|error| {
                AppError::config(format!(
                    "write authority migrations {}: {error}",
                    migrations_path.display()
                ))
            })?;
            output_files.push(migrations_path.display().to_string());
        }
    }

    let mut output_s3_uris = Vec::new();
    if let Some(s3) = args.output_s3.as_ref() {
        let store = ObjectStore::connect(ObjectStoreConfig {
            bucket: s3.bucket.clone(),
            region: s3.region.clone(),
            profile: s3.profile.clone(),
            access_key_id: None,
            secret_access_key: None,
        })
        .await?;
        let report_key = supervisor_report_key(&s3.prefix, created_at_ms, &supervisor_run_id);
        report.report_key = Some(report_key.clone());
        report.control_plane_run_record.output_report_key = Some(report_key.clone());
        report.control_plane_run_record.checksum = String::new();
        report.control_plane_run_record.checksum = checksum_json(&report.control_plane_run_record);
        report.checksum = String::new();
        report.checksum = checksum_json(&report);
        store
            .put_bytes_idempotent(
                &report_key,
                serde_json::to_vec_pretty(&report)?,
                "application/json",
            )
            .await?;
        let commands_key = supervisor_commands_key(&s3.prefix, created_at_ms, &supervisor_run_id);
        let commands_jsonl = report
            .workflow_commands
            .iter()
            .map(serde_json::to_string)
            .collect::<Result<Vec<_>, _>>()?
            .join("\n");
        store
            .put_bytes_idempotent(
                &commands_key,
                format!("{commands_jsonl}\n").into_bytes(),
                "application/x-ndjson",
            )
            .await?;
        let run_record_key =
            control_plane_run_record_key(&s3.prefix, created_at_ms, &control_plane_run_record_id);
        store
            .put_bytes_idempotent(
                &run_record_key,
                serde_json::to_vec_pretty(&report.control_plane_run_record)?,
                "application/json",
            )
            .await?;
        output_s3_uris.push(format!("s3://{}/{}", s3.bucket, report_key));
        output_s3_uris.push(format!("s3://{}/{}", s3.bucket, commands_key));
        output_s3_uris.push(format!("s3://{}/{}", s3.bucket, run_record_key));
        if !report.authority_migration_records.is_empty() {
            let migrations_key = authority_migration_records_key(
                &s3.prefix,
                created_at_ms,
                &control_plane_run_record_id,
            );
            let migrations_jsonl = report
                .authority_migration_records
                .iter()
                .map(serde_json::to_string)
                .collect::<Result<Vec<_>, _>>()?
                .join("\n");
            store
                .put_bytes_idempotent(
                    &migrations_key,
                    format!("{migrations_jsonl}\n").into_bytes(),
                    "application/x-ndjson",
                )
                .await?;
            output_s3_uris.push(format!("s3://{}/{}", s3.bucket, migrations_key));
        }
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

struct ControlPlaneRunRecordInput<'a> {
    control_plane_run_record_id: &'a str,
    created_at_ms: i64,
    manifest_id: &'a str,
    manifest_checksum: &'a str,
    changed_triggers: &'a BTreeSet<String>,
    domains_seen: usize,
    workflow_commands: &'a [WorkflowCommand],
    authority_migration_records: &'a [AuthorityMigrationRecord],
    output_report_key: Option<String>,
}

fn build_control_plane_run_record(input: ControlPlaneRunRecordInput<'_>) -> ControlPlaneRunRecord {
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
    record.checksum = checksum_json(&record);
    record
}

fn build_authority_migration_records(
    inputs: &[AuthorityMigrationInput],
    control_plane_run_record_id: &str,
    created_at_ms: i64,
    manifest_id: &str,
    manifest_checksum: &str,
) -> Vec<AuthorityMigrationRecord> {
    inputs
        .iter()
        .map(|input| {
            let effective_at_ms = input.effective_at_ms.unwrap_or(created_at_ms);
            let authority_migration_record_id = stable_id(
                "authority_migration",
                &[
                    control_plane_run_record_id,
                    &input.authority_scope,
                    input.from_app.as_deref().unwrap_or("none"),
                    &input.to_app,
                    &effective_at_ms.to_string(),
                ],
            );
            AuthorityMigrationRecord {
                schema_version: AUTHORITY_MIGRATION_RECORD_SCHEMA_VERSION.to_owned(),
                authority_migration_record_id: authority_migration_record_id.clone(),
                control_plane_run_record_id: control_plane_run_record_id.to_owned(),
                authority_scope: input.authority_scope.clone(),
                from_app: input.from_app.clone(),
                to_app: input.to_app.clone(),
                migration_reason: input.migration_reason.clone(),
                effective_at_ms,
                created_at_ms,
                manifest_id: manifest_id.to_owned(),
                manifest_checksum: manifest_checksum.to_owned(),
                idempotency_key: stable_id(
                    "authority_migration_idem",
                    &[&authority_migration_record_id],
                ),
            }
        })
        .collect()
}

async fn estimate_input_keys(domain: &DomainRuntimeSpec) -> AppResult<usize> {
    let store = ObjectStore::connect(ObjectStoreConfig {
        bucket: domain.input_bucket.clone(),
        region: domain.input_region.clone(),
        profile: domain.input_profile.clone(),
        access_key_id: None,
        secret_access_key: None,
    })
    .await?;
    let suffixes = normalized_suffixes(&domain.input_suffixes);
    let mut unique = BTreeSet::new();
    for prefix in &domain.input_prefixes {
        for key in store.list_keys(prefix, domain.max_keys_per_prefix).await? {
            if suffixes.is_empty() || suffixes.iter().any(|suffix| key.ends_with(suffix)) {
                unique.insert(key);
            }
        }
    }
    Ok(unique.len())
}

fn build_workflow_command(
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

fn render_command_template(domain: &DomainRuntimeSpec) -> Vec<String> {
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

fn domain_matches_triggers(
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

fn validate_manifest(manifest: &DomainReplayManifest) -> AppResult<()> {
    if manifest.schema_version != MANIFEST_SCHEMA_VERSION {
        return Err(AppError::config(format!(
            "manifest schema_version must be {MANIFEST_SCHEMA_VERSION}"
        )));
    }
    if manifest.manifest_id.trim().is_empty() {
        return Err(AppError::config("manifest_id is required"));
    }
    if manifest.domains.is_empty() {
        return Err(AppError::config("at least one domain is required"));
    }
    for domain in &manifest.domains {
        if domain.domain_id.trim().is_empty()
            || domain.target_app.trim().is_empty()
            || domain.target_mode.trim().is_empty()
            || domain.input_bucket.trim().is_empty()
            || domain.input_prefixes.is_empty()
            || domain.output_bucket.trim().is_empty()
            || domain.output_report_prefix.trim().is_empty()
            || domain.command_template.is_empty()
        {
            return Err(AppError::config(format!(
                "domain {} is missing required fields",
                domain.domain_id
            )));
        }
    }
    for record in &manifest.authority_migration_records {
        if record.authority_scope.trim().is_empty()
            || record.to_app.trim().is_empty()
            || record.migration_reason.trim().is_empty()
        {
            return Err(AppError::config(
                "authority_migration_records require authority_scope, to_app, and migration_reason",
            ));
        }
        if record
            .effective_at_ms
            .is_some_and(|effective_at_ms| effective_at_ms < 0)
        {
            return Err(AppError::config(
                "authority_migration_records.effective_at_ms must be non-negative",
            ));
        }
    }
    Ok(())
}

fn parse_args(values: impl Iterator<Item = String>) -> AppResult<Option<Args>> {
    let mut args = Args {
        manifest_file: PathBuf::new(),
        changed_triggers: BTreeSet::new(),
        output_dir: None,
        output_s3: None,
        now_ms: None,
    };
    let mut values = values.peekable();
    while let Some(arg) = values.next() {
        match arg.as_str() {
            "-h" | "--help" => return Ok(None),
            "--manifest-file" => {
                args.manifest_file =
                    absolute_path_arg(values.next(), "--manifest-file requires an absolute path")?;
            }
            "--changed-trigger" => {
                args.changed_triggers.insert(next_string(
                    &mut values,
                    "--changed-trigger requires a value",
                )?);
            }
            "--output-dir" => {
                args.output_dir = Some(absolute_path_arg(
                    values.next(),
                    "--output-dir requires an absolute path",
                )?);
            }
            "--output-s3-bucket" => {
                let bucket = next_string(&mut values, "--output-s3-bucket requires a bucket")?;
                let existing = args.output_s3.get_or_insert_with(default_s3_output);
                existing.bucket = bucket;
            }
            "--output-s3-region" => {
                let region = next_string(&mut values, "--output-s3-region requires a region")?;
                let existing = args.output_s3.get_or_insert_with(default_s3_output);
                existing.region = region;
            }
            "--output-s3-prefix" => {
                let prefix = next_string(&mut values, "--output-s3-prefix requires a prefix")?;
                let existing = args.output_s3.get_or_insert_with(default_s3_output);
                existing.prefix = prefix;
            }
            "--aws-profile" => {
                let profile = Some(next_string(
                    &mut values,
                    "--aws-profile requires a profile",
                )?);
                let existing = args.output_s3.get_or_insert_with(default_s3_output);
                existing.profile = profile;
            }
            "--now-ms" => {
                args.now_ms = Some(parse_non_negative_i64(&next_string(
                    &mut values,
                    "--now-ms requires a timestamp",
                )?)?);
            }
            other => {
                return Err(AppError::config(format!(
                    "unknown argument: {other}\n\n{}",
                    help_text()
                )));
            }
        }
    }
    if args.manifest_file.as_os_str().is_empty() {
        return Err(AppError::config("--manifest-file is required"));
    }
    if args.changed_triggers.is_empty() {
        return Err(AppError::config("--changed-trigger is required"));
    }
    if let Some(s3) = args.output_s3.as_ref()
        && (s3.bucket.trim().is_empty() || s3.prefix.trim().is_empty())
    {
        return Err(AppError::config(
            "--output-s3-bucket and --output-s3-prefix are required for S3 output",
        ));
    }
    Ok(Some(args))
}

fn absolute_path_arg(value: Option<String>, message: &str) -> AppResult<PathBuf> {
    let value = value.ok_or_else(|| AppError::config(message))?;
    let path = PathBuf::from(value);
    if !path.is_absolute() {
        return Err(AppError::config(format!(
            "{message}; got {}",
            path.display()
        )));
    }
    Ok(path)
}

fn next_string(
    values: &mut std::iter::Peekable<impl Iterator<Item = String>>,
    message: &str,
) -> AppResult<String> {
    values.next().ok_or_else(|| AppError::config(message))
}

fn parse_non_negative_i64(value: &str) -> AppResult<i64> {
    let parsed = value
        .parse::<i64>()
        .map_err(|error| AppError::config(format!("invalid timestamp {value}: {error}")))?;
    if parsed < 0 {
        return Err(AppError::config("timestamp must be non-negative"));
    }
    Ok(parsed)
}

fn default_s3_output() -> S3OutputArgs {
    S3OutputArgs {
        bucket: String::new(),
        region: DEFAULT_AWS_REGION.to_owned(),
        prefix: String::new(),
        profile: None,
    }
}

fn default_region() -> String {
    DEFAULT_AWS_REGION.to_owned()
}

fn default_max_keys() -> usize {
    10_000
}

fn normalized_suffixes(values: &[String]) -> BTreeSet<String> {
    values
        .iter()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .collect()
}

fn supervisor_report_key(prefix: &str, created_at_ms: i64, supervisor_run_id: &str) -> String {
    partitioned_key(prefix, created_at_ms, supervisor_run_id, "report.json")
}

fn supervisor_commands_key(prefix: &str, created_at_ms: i64, supervisor_run_id: &str) -> String {
    partitioned_key(prefix, created_at_ms, supervisor_run_id, "commands.jsonl")
}

fn control_plane_run_record_key(
    prefix: &str,
    created_at_ms: i64,
    control_plane_run_record_id: &str,
) -> String {
    let part = time_part(created_at_ms);
    format!(
        "{}/schema={}/dt={}/hour={:02}/control_plane_run_record_id={}/record.json",
        prefix.trim_matches('/'),
        CONTROL_PLANE_RUN_RECORD_SCHEMA_VERSION,
        part.event_date,
        part.hour,
        path_segment(control_plane_run_record_id)
    )
}

fn authority_migration_records_key(
    prefix: &str,
    created_at_ms: i64,
    control_plane_run_record_id: &str,
) -> String {
    let part = time_part(created_at_ms);
    format!(
        "{}/schema={}/dt={}/hour={:02}/control_plane_run_record_id={}/part-000001.jsonl",
        prefix.trim_matches('/'),
        AUTHORITY_MIGRATION_RECORD_SCHEMA_VERSION,
        part.event_date,
        part.hour,
        path_segment(control_plane_run_record_id)
    )
}

fn partitioned_key(
    prefix: &str,
    created_at_ms: i64,
    supervisor_run_id: &str,
    leaf: &str,
) -> String {
    let part = time_part(created_at_ms);
    format!(
        "{}/schema={}/dt={}/hour={:02}/supervisor_run_id={}/{}",
        prefix.trim_matches('/'),
        REPORT_SCHEMA_VERSION,
        part.event_date,
        part.hour,
        path_segment(supervisor_run_id),
        leaf
    )
}

fn checksum_json<T: Serialize>(value: &T) -> String {
    sha256_hex(serde_json::to_vec(value).unwrap_or_default())
}

fn log_event(event: &str, fields: serde_json::Value) {
    let mut value = json!({
        "schema_version": "domain_replay_supervisor_log_v1",
        "producer_app": PRODUCER_APP,
        "timestamp_ms": now_ms(),
        "level": "info",
        "event": event,
    });
    if let (Some(target), Some(source)) = (value.as_object_mut(), fields.as_object()) {
        for (key, field_value) in source {
            target.insert(key.clone(), field_value.clone());
        }
    }
    println!("{value}");
}

fn print_help() {
    println!("{}", help_text());
}

fn help_text() -> &'static str {
    r#"domain-replay-supervisor-app
Usage:
  domain-replay-supervisor-app \
    --manifest-file /Volumes/WD/Developments/nangman-crypto/domains/runtime/domain-replay-manifest.dev.json \
    --changed-trigger scoring_policy_version_changed \
    --output-dir /Volumes/WD/Developments/nangman-crypto/data/replay-supervisor \
    --output-s3-bucket nangman-crypto-dev-control-plane-<account-suffix> \
    --output-s3-prefix domain-replay-supervisor

This app does not implement domain logic. It scans durable input prefixes,
creates workflow_command records, and writes a durable supervisor report."#
}

#[allow(dead_code)]
fn _assert_absolute_path(_: &Path) {}

#[cfg(test)]
mod tests {
    use super::*;

    fn domain() -> DomainRuntimeSpec {
        DomainRuntimeSpec {
            domain_id: "intel-candidate".to_owned(),
            target_app: "intel-candidate-app".to_owned(),
            target_mode: "replay".to_owned(),
            input_bucket: "input-bucket".to_owned(),
            input_region: "ap-northeast-2".to_owned(),
            input_profile: None,
            input_prefixes: vec![
                "structured-intel-packet/schema=structured_intel_packet_v1/".to_owned(),
            ],
            input_suffixes: vec![".json".to_owned(), ".jsonl".to_owned()],
            max_keys_per_prefix: 100,
            replay_triggers: vec!["scoring_policy_version_changed".to_owned()],
            output_bucket: "output-bucket".to_owned(),
            output_region: "ap-northeast-2".to_owned(),
            output_report_prefix: "candidate-replay-report".to_owned(),
            command_template: vec![
                "/usr/local/bin/intel-candidate-replay-worker".to_owned(),
                "--input-s3-bucket".to_owned(),
                "{input_bucket}".to_owned(),
                "--output-s3-bucket".to_owned(),
                "{output_bucket}".to_owned(),
                "{repeat_input_prefixes}".to_owned(),
            ],
        }
    }

    #[test]
    fn trigger_matching_allows_force_all() {
        let domain = domain();
        assert!(domain_matches_triggers(
            &domain,
            &BTreeSet::from(["force_all".to_owned()])
        ));
    }

    #[test]
    fn trigger_matching_requires_overlap() {
        let domain = domain();
        assert!(domain_matches_triggers(
            &domain,
            &BTreeSet::from(["scoring_policy_version_changed".to_owned()])
        ));
        assert!(!domain_matches_triggers(
            &domain,
            &BTreeSet::from(["unrelated".to_owned()])
        ));
    }

    #[test]
    fn command_template_is_rendered() {
        let command = render_command_template(&domain());
        assert_eq!(
            command,
            vec![
                "/usr/local/bin/intel-candidate-replay-worker",
                "--input-s3-bucket",
                "input-bucket",
                "--output-s3-bucket",
                "output-bucket",
                "--replay-input-prefix",
                "structured-intel-packet/schema=structured_intel_packet_v1/"
            ]
        );
    }

    #[test]
    fn control_plane_run_record_captures_commands_and_authority_migrations() {
        let changed_triggers = BTreeSet::from(["scoring_policy_version_changed".to_owned()]);
        let command = build_workflow_command(
            "supervisor_run_001",
            &domain(),
            &changed_triggers,
            3,
            1_700_000_000_000,
        );
        let migrations = build_authority_migration_records(
            &[AuthorityMigrationInput {
                authority_scope: "control-plane-lite:domain-replay-supervisor".to_owned(),
                from_app: None,
                to_app: "domain-replay-supervisor-app".to_owned(),
                migration_reason: "initial_owner".to_owned(),
                effective_at_ms: None,
            }],
            "control_run_001",
            1_700_000_000_000,
            "manifest_001",
            "checksum_001",
        );
        let record = build_control_plane_run_record(ControlPlaneRunRecordInput {
            control_plane_run_record_id: "control_run_001",
            created_at_ms: 1_700_000_000_000,
            manifest_id: "manifest_001",
            manifest_checksum: "checksum_001",
            changed_triggers: &changed_triggers,
            domains_seen: 1,
            workflow_commands: &[command],
            authority_migration_records: &migrations,
            output_report_key: Some("domain-replay-supervisor/report.json".to_owned()),
        });

        assert_eq!(
            record.schema_version,
            CONTROL_PLANE_RUN_RECORD_SCHEMA_VERSION
        );
        assert_eq!(record.domains_seen, 1);
        assert_eq!(record.domains_selected, 1);
        assert_eq!(record.workflow_command_ids.len(), 1);
        assert_eq!(record.authority_migration_record_ids.len(), 1);
        assert!(!record.checksum.is_empty());
        assert_eq!(
            migrations[0].schema_version,
            AUTHORITY_MIGRATION_RECORD_SCHEMA_VERSION
        );
        assert_eq!(migrations[0].control_plane_run_record_id, "control_run_001");
    }
}
