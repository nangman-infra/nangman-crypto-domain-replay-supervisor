use intel_candidate_app::error::{AppError, AppResult};
use intel_candidate_app::hash::{sha256_hex, stable_id};
use intel_candidate_app::storage::{ObjectStore, ObjectStoreConfig};
use intel_candidate_app::time::{now_ms, path_segment, time_part};
use serde::Serialize;
use serde_json::json;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

mod args;
mod keys;
mod output;
mod planning;
mod run;
mod types;

use args::*;
use keys::*;
use output::*;
use planning::*;
use run::run;
use types::*;

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
                match serde_json::to_string_pretty(&summary) {
                    Ok(output) => println!("{output}"),
                    Err(error) => {
                        eprintln!("{error}");
                        std::process::exit(1);
                    }
                }
            }
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests;
