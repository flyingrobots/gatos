.PHONY: diagrams lint-md fix-md link-check schemas

diagrams:
	@bash -lc 'scripts/mermaid/generate_all.sh'

lint-md:
	@bash -lc 'if command -v node >/dev/null 2>&1; then \
	  npx -y markdownlint-cli "**/*.md" --config .markdownlint.json --ignore docs/ok-real-talk.md; \
	elif command -v docker >/dev/null 2>&1; then \
	  docker run --rm -v "$$PWD:/work" -w /work node:20 bash -lc "npx -y markdownlint-cli \"**/*.md\" --config .markdownlint.json --ignore docs/ok-real-talk.md"; \
	else echo "Need Node.js or Docker" >&2; exit 1; fi'

fix-md:
	@bash -lc 'if command -v node >/dev/null 2>&1; then \
	  npx -y markdownlint-cli "**/*.md" --fix --config .markdownlint.json --ignore docs/ok-real-talk.md; \
	elif command -v docker >/dev/null 2>&1; then \
	  docker run --rm -v "$$PWD:/work" -w /work node:20 bash -lc "npx -y markdownlint-cli \"**/*.md\" --fix --config .markdownlint.json --ignore docs/ok-real-talk.md"; \
	else echo "Need Node.js or Docker" >&2; exit 1; fi'

link-check:
	@bash -lc 'if command -v lychee >/dev/null 2>&1; then \
	  lychee --no-progress --config .lychee.toml **/*.md; \
	elif command -v docker >/dev/null 2>&1; then \
	  docker run --rm -v "$$PWD:/work" -w /work ghcr.io/lycheeverse/lychee:latest --no-progress --config .lychee.toml **/*.md; \
	else echo "Need lychee or Docker" >&2; exit 1; fi'

