use crate::types::DomainRuntimeSpec;

pub(super) fn domain() -> DomainRuntimeSpec {
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
