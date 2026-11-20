# Security Policy

## Reporting a Vulnerability

Report vulnerabilities privately through GitHub security advisories: open the repository's Security tab and choose "Report a vulnerability", or go to <https://github.com/klappe-pm/lynter/security/advisories/new>. Do not open a public issue, pull request or discussion for a suspected vulnerability.

Include the affected version or commit, the steps to reproduce, and the impact you observed. You will receive an acknowledgement, and the fix and disclosure are coordinated with you through the advisory.

## Scope

- The `lynter` command line, the `lynter-core`, `lynter-code` and `lynter-wasm` crates, the Obsidian plugin in `plugin/`, and the lynter.dev site in `packages/site/`.
- Any way to make Lynter print, log or return a secret or another user's data.
- Any input that makes Lynter take unbounded time or memory.
- Any way a check, guard or permission in Lynter fails open, including a `lynter.yaml` that is rejected but still partly applied.

Vulnerabilities in third-party dependencies and platforms are out of scope; report those to their maintainers. A dependency update that fixes a published advisory is a welcome pull request.

## Supported Versions

Only the latest release and the `master` branch receive security fixes. Until the first release, only `master` does.
