`check` exits 1 when a file breaks a rule and 0 when every file passes. `--changed` limits it to files git sees as changed. The examples below use the binaries from the [latest release](https://github.com/klappe-pm/lynter/releases/latest).

## GitHub Actions

```yaml
name: lynter
on: [pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with: {fetch-depth: 0}
      - uses: klappe-pm/lynter/.github/actions/lynter@0.1.0
```

The action downloads the release binary for the runner, verifies it against `SHA256SUMS`, and runs `lynter check`. Its inputs are `paths` (default `.`), `config`, `changed` (`true` checks only files changed against the base branch) and `version` (default `latest`).

```yaml
      - uses: klappe-pm/lynter/.github/actions/lynter@0.1.0
        with:
          paths: docs notes
          changed: 'true'
```

## GitLab CI

```yaml
lynter:
  image: debian:stable-slim
  script:
    - apt-get update && apt-get install -y curl ca-certificates
    - curl -LO https://github.com/klappe-pm/lynter/releases/download/0.1.0/lynter-0.1.0-x86_64-unknown-linux-gnu.tar.gz
    - curl -LO https://github.com/klappe-pm/lynter/releases/download/0.1.0/SHA256SUMS
    - grep x86_64-unknown-linux-gnu SHA256SUMS | sha256sum -c -
    - tar -xzf lynter-0.1.0-x86_64-unknown-linux-gnu.tar.gz
    - ./lynter-0.1.0-x86_64-unknown-linux-gnu/lynter check .
  rules:
    - if: $CI_PIPELINE_SOURCE == "merge_request_event"
```

## pre-commit

```yaml
repos:
  - repo: local
    hooks:
      - id: lynter
        name: lynter
        entry: {{CLI}} check --changed
        language: system
        pass_filenames: false
```

## lefthook

```yaml
pre-commit:
  commands:
    lynter:
      run: {{CLI}} check --changed
```

## husky

```bash
# .husky/pre-commit
{{CLI}} check --changed
```

## JSON output

Add `--json` to print the result as JSON for a script or a CI annotation step.

```bash
{{CLI}} check --json > lynter-report.json
```
