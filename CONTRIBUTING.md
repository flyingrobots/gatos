# Contributing

## Developer Tooling

This repo includes optional tooling to keep docs tidy and diagrams fresh.

### Markdown Linting

- CI runs markdownlint (CLI2) via Node 20.
- Local (preferred): no setup required. The pre-commit hook uses `npx` or a Docker fallback so you don’t need Node installed globally.
- Manual runs:

```bash
# Check
npx -y markdownlint-cli2
# Auto-fix
npx -y markdownlint-cli2-fix
```

### Mermaid Diagrams

- Generate SVGs from all Mermaid code blocks in Markdown:

```bash
node scripts/mermaid/generate.mjs
```

Outputs are written to `docs/diagrams/generated/`.

Diagram generation modes

- Pre-commit: generates SVGs only for the staged Markdown files you’re committing (fast).
- CI: regenerates all diagrams across the repo and fails if there’s drift (ensures reproducibility).

Manual full regeneration (all Markdown files):

```bash
scripts/mermaid/generate_all.sh
```

### Git Hooks

Install the pre-commit hook (runs markdownlint fix + mermaid generation for staged files and stages results; uses Node if available, otherwise Docker with a Node 20 image):

```bash
scripts/setup-hooks.sh
```

If the hook fails, fix the reported issues and retry the commit.
