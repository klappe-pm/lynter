# Contributing

Thank you for helping improve Lynter. Open an issue to discuss a change before writing a large pull request, so the effort goes where it will be merged. This page holds the rules every contribution follows; the `lynter.yaml` reference is [docs/lynter-yaml.md](docs/lynter-yaml.md).

## Setup

```bash
git clone https://github.com/klappe-pm/lynter && cd lynter
```

You need a stable Rust toolchain and Node.js 20 or later. Install the site and lint dependencies from the repository root with `npm ci --ignore-scripts`, and the plugin's own dependencies with `cd plugin && npm ci`.

## Tests

Run the same checks CI runs before every commit. They are listed in `.github/workflows/ci.yml` and `.github/workflows/plugin.yml`.

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd plugin && npm ci && npm run build && npm test
npm ci --ignore-scripts && npx eslint . --ext .ts
npm run build -w @lynter/site && npm test -w @lynter/site
```

The root `manifest.json` and `versions.json` are exact copies of the files in `plugin/`. Change the plugin's files and copy them to the root. After a change to `Cargo.lock`, run `python3 packaging/licenses/generate.py` to refresh the bundled license texts.

## Guidelines

- Keep each change small and focused, with tests for new behavior. Every bug fix gets a test that fails without the fix.
- Tests read fixtures only: never the live home directory, a real network or a real credential. Test values are synthetic.
- Documentation goes in `docs/`. Each paragraph and list item is one line, not hard-wrapped. Filenames are lowercase kebab-case, apart from the community files and `README.md`.
- Do not use a dash as punctuation. Use a comma, colon, semicolon, period or parentheses, and write ranges as "3 to 5".
- Everything committed here is public. Never add a session link or session provenance keys to any file, and never credit an agent, model, model provider or agent runtime as author or contributor.
- Do not copy anything from a private repository: no client names, personal paths, notes, plans or prompts. Examples and fixtures are synthetic, apart from the documented examples credited in `THIRD-PARTY-NOTICES.md`.
- Never commit a secret, credential, private key, `.env` file or `settings.local.json`. `.env.example` with placeholder values is allowed.
- Never bypass commit hooks with `--no-verify`; fix what they report.
- Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/).

## Pull Requests

Keep each pull request to one change, describe what it does and how you tested it, and make sure every check passes.

## License

By contributing, you agree that your contributions are licensed under the license in `LICENSE`.
