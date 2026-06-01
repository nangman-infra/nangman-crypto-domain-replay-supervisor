#[test]
fn parse_args_rejects_paths_with_relative_components() {
    for (flag, value) in [
        ("--manifest-file", "/tmp/../manifest.json"),
        ("--output-dir", "/tmp/./domain-replay-out"),
    ] {
        let error = crate::args::parse_args(
            [
                "--manifest-file",
                "/tmp/manifest.json",
                "--changed-trigger",
                "force_all",
                flag,
                value,
            ]
            .into_iter()
            .map(ToOwned::to_owned),
        )
        .expect_err("ambiguous absolute path should fail");
        let text = error.to_string();
        assert!(
            text.contains(flag),
            "expected {flag} in error for {value:?}, got {text}"
        );
        assert!(
            text.contains("relative path components"),
            "expected relative component error for {value:?}, got {text}"
        );
    }
}

#[test]
fn parse_args_accepts_local_and_s3_outputs() {
    let parsed = crate::args::parse_args(
        [
            "--manifest-file",
            "/tmp/domain-replay-manifest.json",
            "--changed-trigger",
            "force_all",
            "--changed-trigger",
            "scoring_policy_version_changed",
            "--output-dir",
            "/tmp/domain-replay-out",
            "--output-s3-bucket",
            "control-plane-bucket",
            "--output-s3-region",
            "us-east-1",
            "--output-s3-prefix",
            "/domain-replay-supervisor/",
            "--aws-profile",
            "dev-profile",
            "--now-ms",
            "7200000",
        ]
        .into_iter()
        .map(ToOwned::to_owned),
    )
    .unwrap()
    .expect("args should parse");

    assert_eq!(
        parsed.manifest_file,
        std::path::PathBuf::from("/tmp/domain-replay-manifest.json")
    );
    assert_eq!(parsed.changed_triggers.len(), 2);
    assert_eq!(
        parsed.output_dir,
        Some(std::path::PathBuf::from("/tmp/domain-replay-out"))
    );
    assert_eq!(parsed.now_ms, Some(7_200_000));

    let s3 = parsed.output_s3.expect("s3 output should be configured");
    assert_eq!(s3.bucket, "control-plane-bucket");
    assert_eq!(s3.region, "us-east-1");
    assert_eq!(s3.prefix, "/domain-replay-supervisor/");
    assert_eq!(s3.profile.as_deref(), Some("dev-profile"));
}

#[test]
fn parse_args_handles_help_without_required_args() {
    assert!(
        crate::args::parse_args(["--help"].into_iter().map(ToOwned::to_owned))
            .unwrap()
            .is_none()
    );
}

#[test]
fn parse_args_rejects_invalid_timestamp_and_unknown_args() {
    let timestamp_error = crate::args::parse_args(
        [
            "--manifest-file",
            "/tmp/domain-replay-manifest.json",
            "--changed-trigger",
            "force_all",
            "--now-ms",
            "-1",
        ]
        .into_iter()
        .map(ToOwned::to_owned),
    )
    .unwrap_err()
    .to_string();
    assert!(timestamp_error.contains("timestamp must be non-negative"));

    let unknown_error = crate::args::parse_args(
        [
            "--manifest-file",
            "/tmp/domain-replay-manifest.json",
            "--changed-trigger",
            "force_all",
            "--unsupported",
        ]
        .into_iter()
        .map(ToOwned::to_owned),
    )
    .unwrap_err()
    .to_string();
    assert!(unknown_error.contains("unknown argument"));
    assert!(unknown_error.contains("domain-replay-supervisor-app"));
}
