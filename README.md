# domain-replay-supervisor-app

`domain-replay-supervisor-app`은 도메인 로직을 다시 구현하지 않는다.

역할:

```text
domain replay manifest 읽기
-> changed trigger와 domain replay trigger 비교
-> durable input prefix 스캔
-> workflow_command_v1 생성
-> supervisor report를 로컬 또는 S3에 저장
```

기본 실행 예시:

```bash
cargo run \
  --manifest-path /Volumes/WD/Developments/nangman-crypto/apps/domain-replay-supervisor-app/Cargo.toml \
  -- \
  --manifest-file /Volumes/WD/Developments/nangman-crypto/domains/runtime/domain-replay-manifest.dev.json \
  --changed-trigger scoring_policy_version_changed \
  --output-dir /Volumes/WD/Developments/nangman-crypto/data/replay-supervisor
```

S3 report까지 쓰는 실행 예시:

```bash
cargo run \
  --manifest-path /Volumes/WD/Developments/nangman-crypto/apps/domain-replay-supervisor-app/Cargo.toml \
  -- \
  --manifest-file /Volumes/WD/Developments/nangman-crypto/domains/runtime/domain-replay-manifest.dev.json \
  --changed-trigger scoring_policy_version_changed \
  --output-s3-bucket nangman-crypto-dev-control-plane-<account-suffix> \
  --output-s3-prefix domain-replay-supervisor
```

`force_all` trigger를 쓰면 manifest에 등록된 모든 domain을 replay 대상으로 잡는다.
