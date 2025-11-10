# Contributing

## Developer Tooling

This repo includes optional tooling to keep docs tidy and diagrams fresh.

### Markdown Linting

- CI runs markdownlint via `markdownlint-cli2`.
- Local: install Node 20+ and run:

```
npm i
npm run md:lint      # check only
npm run md:lint:fix  # auto-fix
```

### Mermaid Diagrams

- Generate SVGs from all Mermaid code blocks in Markdown:

```
npm run md:mermaid:gen
```

Outputs are written to `docs/diagrams/generated/`.

### Git Hooks

Install the pre-commit hook (runs markdownlint fix + mermaid generation and stages results):

```
scripts/setup-hooks.sh
```

If the hook fails, fix the reported issues and retry the commit.

