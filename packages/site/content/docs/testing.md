---
title: Testing
lede: How Lynter checks its own correctness with frozen fixtures, golden CLI runs, property tests, plugin and site tests, and the CI workflows.
---

Lynter's rules rewrite your notes, so the test suite is built around one question: does a given input always produce the same bytes? Most checks replay recorded cases and compare the output exactly. Property tests then probe inputs nobody wrote by hand. This page maps each layer to the files that hold it. The commands CI runs are also listed in `CONTRIBUTING.md`.

## Frozen fixtures and hashes

The core's behavior is pinned by JSON fixtures under `crates/lynter-core/tests/fixtures`. They hold recorded cases: an input, the options, and the expected output. They include per-rule tables (`rules-*.json`), scope resolution (`scope.json`), config loading (`config.json`), whole-file linting (`lint.json`) and settings conversion (`migrate.json`). A `compat/` subfolder holds fixtures for the compat rules.

The fixtures were exported from a TypeScript core and are frozen. A `SHA256SUMS` file beside each set lists the hash of every fixture. The tests in `tests/golden_hashes.rs` recompute each hash with a small SHA-256 implementation inside the test, so no extra dependency is needed. They fail in two cases: a fixture changed, or a fixture file exists that `SHA256SUMS` does not list. The failure message says to update `SHA256SUMS` only for an intended change, which makes accidental fixture edits visible in review.

`tests/parity.rs` and `tests/parity_slice2.rs` replay the fixtures against the Rust rules and the resolver, and assert byte-identical output and identical profile chains. They use the line-scan protection mode, which matches the recorded behavior. The core README lists the few intended differences from the recorded behavior. `tests/compat.rs` runs every example in `fixtures/compat/<id>.json` through its rule, plus the never-edited guarantees as direct tests.

Sources: `crates/lynter-core/tests/golden_hashes.rs`, `crates/lynter-core/tests/parity.rs`, `crates/lynter-core/tests/parity_slice2.rs`, `crates/lynter-core/tests/compat.rs`, `crates/lynter-core/tests/fixtures/SHA256SUMS`, `crates/lynter-core/README.md`

## Unit and table tests

- `tests/document.rs` checks the document model, such as list item offsets starting at the marker.
- `tests/lint_defaults.rs` checks the compiled defaults and the scaffold: the scaffold parses with no errors or warnings, and the opt-in rules are off in both.
- `tests/notes.rs` holds behavior tables and property tests for the notes rules, which are defined by those tests.
- In `crates/lynter-code/tests`, `data.rs`, `identifier_case.rs` and `structure_rules.rs` cover the JSON and YAML data rules and the code rules. The adapter output parsers use fixtures stored next to the source in `crates/lynter-code/src/adapters/fixtures`.

Sources: `crates/lynter-core/tests/document.rs`, `crates/lynter-core/tests/lint_defaults.rs`, `crates/lynter-core/tests/notes.rs`, `crates/lynter-code/tests/data.rs`

## Property tests

`proptest` generates documents from fragments of Markdown, frontmatter, code and math, then checks invariants that must hold for every input.

| File | Invariants |
| --- | --- |
| `props.rs` | Fixing is idempotent. Edits never touch protected regions. Protected blocks survive a full fix. Runs 400 cases per property. |
| `props_slice2.rs` | Fixing is idempotent and protected regions survive, for a second group of rules. |
| `compat_props.rs` | Idempotence, protected regions survive, frontmatter rules never touch the body, check mode never changes the text, and rules stay off unless enabled. |

Failing cases that proptest finds are kept in `*.proptest-regressions` files beside the tests, so they replay on every run.

Sources: `crates/lynter-core/tests/props.rs`, `crates/lynter-core/tests/props_slice2.rs`, `crates/lynter-core/tests/compat_props.rs`

## CLI tests

`crates/lynter-cli/tests/cli.rs` runs the real binary against temporary vaults and checks exit codes, discovery, deprecated names, `--config`, `explain`, `init`, `--json` keys, `--changed` and the routing of code files. `tests/adapters.rs` covers the ruff and Biome adapters with fake executables standing in for the tools, and exercises real ruff when it is installed.

`tests/golden.rs` is a golden test. It copies a small vault into a temp folder, runs `check` and then `fix` with `--json --today 2026-01-01`, and compares standard output, the exit code and every resulting file with frozen goldens in `tests/golden`. A `SHA256SUMS` file pins the goldens, and a mismatch means a golden changed.

Sources: `crates/lynter-cli/tests/cli.rs`, `crates/lynter-cli/tests/adapters.rs`, `crates/lynter-cli/tests/golden.rs`, `crates/lynter-cli/tests/golden/SHA256SUMS`

## Plugin and site tests

The plugin tests run under Jest with `ts-jest`. Mocks replace the `obsidian` module, the WebAssembly glue and the inlined binary, so tests run in plain Node. They cover base64 and gzip decoding, config discovery, the debounce timing of lint on save, the save guard, and the call contract with the WebAssembly `lint` and `validate_config` exports.

The site tests run under Jest as well. `examples.test.ts` lints the example vaults and the gallery with the committed WebAssembly build, in a child Node process. `parity.test.ts` checks that `lynter fix --stdin` from the debug CLI binary produces the same text as the WebAssembly build for every sample and example note. It is skipped when `target/debug/lynter` is not built. Site scripts add three more checks: `check:export` verifies the static export, `check:contrast` checks color contrast, and `smoke` fetches every page of a deployed site.

Sources: `plugin/jest.config.cjs`, `plugin/test/lint-contract.test.ts`, `plugin/test/debounce.test.ts`, `packages/site/__tests__/examples.test.ts`, `packages/site/__tests__/parity.test.ts`, `packages/site/scripts/check-export.mjs`, `packages/site/scripts/smoke.mjs`

## CI workflows

| Workflow | Runs on | What it checks |
| --- | --- | --- |
| `ci.yml`, job `rust` | Pull requests and pushes to `master` | `cargo fmt --check`, `cargo clippy` over all targets with warnings denied, `cargo test --workspace`, and `generate.py --check` for the bundled license texts. |
| `ci.yml`, job `node` | Same | Builds the debug CLI, installs dependencies, runs ESLint, builds the site, checks the export, runs the site tests and the npm installer test. |
| `ci.yml`, job `manifest` | Same | The root `manifest.json` and `versions.json` are identical to the plugin's. |
| `plugin.yml` | Pull requests and pushes | Builds and tests the plugin, enforces the 1,300,000 byte size limit, and forbids Node built-ins in the bundle. |
| `site.yml` | Pushes to `master` that touch the site, docs or examples | Builds the site, checks the export, deploys to GitHub Pages, then smoke tests the live site. |
| `release.yml` | Version tags | Builds and publishes a release. See [releases](/docs/releases/). |
| `attribution-check.yml` | Pull requests and pushes | Checks titles, bodies and commit messages against the contribution rules. A second workflow checks comments. |

Sources: `.github/workflows/ci.yml`, `.github/workflows/plugin.yml`, `.github/workflows/site.yml`, `.github/workflows/attribution-check.yml`, `CONTRIBUTING.md`
