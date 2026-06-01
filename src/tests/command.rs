use super::fixtures::domain;
use crate::planning::render_command_template;

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
