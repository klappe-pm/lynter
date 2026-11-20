Agents write Markdown fast and in their own style. A hook runs Lynter on each file the agent writes, so the file lands in house style. The rules are deterministic code. There is no model call and no token cost.

## The command

`fix --stdin` reads a file from standard input and writes the fixed text to standard output. `--stdin-path` tells Lynter which scope applies, because the path decides which profiles match.

```bash
{{CLI}} fix --stdin --stdin-path "$FILE" < "$FILE" > "$FILE.tmp" && mv "$FILE.tmp" "$FILE"
```

## Claude Code

Add a PostToolUse hook that runs after the Write and Edit tools. The hook script reads the file path from the hook input, then runs the command above on Markdown files.

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Write|Edit",
        "hooks": [{"type": "command", "command": "./scripts/lynter-hook.sh"}]
      }
    ]
  }
}
```

```bash
#!/usr/bin/env bash
FILE=$(jq -r '.tool_input.file_path // empty')
case "$FILE" in
  *.md) {{CLI}} fix --stdin --stdin-path "$FILE" < "$FILE" > "$FILE.tmp" && mv "$FILE.tmp" "$FILE" ;;
esac
```

## Codex

Codex reads `AGENTS.md` for instructions. Add a line that tells it to run the check before it finishes, and run the same command in CI as the backstop.

```markdown
Before you finish, run `{{CLI}} fix --changed` and commit the result.
```

## Cursor

Add a rule in `.cursor/rules` with the same instruction, and run `{{CLI}} check --changed` in a pre-commit hook so a file that skips the instruction still gets caught.

## Check, not just fix

Use `check` when you want a failing exit code instead of a rewrite. Add `--json` to read the violations from a script.

```bash
{{CLI}} check --changed --json
```
