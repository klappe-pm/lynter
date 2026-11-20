`check` exits 1 when a file breaks a rule and 0 when every file passes. `--changed` limits it to files git sees as changed,. The examples below build the CLI from the repository, because Lynter is not on a package registry yet.

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
      - run: cargo install --git https://github.com/klappe-pm/lynter.git lynter-cli
      - run: lynter check .
```

## GitLab CI

```yaml
lynter:
  image: rust:1
  script:
    - cargo install --git https://github.com/klappe-pm/lynter.git lynter-cli
    - lynter check .
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
