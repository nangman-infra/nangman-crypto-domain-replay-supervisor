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
