use crate::planning::validate_manifest;
use crate::types::{Args, DomainReplayManifest};
use intel_candidate_app::error::{AppError, AppResult};
use intel_candidate_app::hash::sha256_hex;
use std::fs;

pub(super) struct LoadedManifest {
    pub(super) manifest: DomainReplayManifest,
    pub(super) manifest_checksum: String,
}

pub(super) fn load_manifest(args: &Args) -> AppResult<LoadedManifest> {
    let manifest_bytes = fs::read(&args.manifest_file).map_err(|error| {
        AppError::config(format!(
            "read manifest {}: {error}",
            args.manifest_file.display()
        ))
    })?;
    let manifest: DomainReplayManifest = serde_json::from_slice(&manifest_bytes)?;
    validate_manifest(&manifest)?;
    Ok(LoadedManifest {
        manifest,
        manifest_checksum: sha256_hex(&manifest_bytes),
    })
}
