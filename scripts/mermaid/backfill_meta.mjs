#!/usr/bin/env node
// Backfill mermaid-meta into already-committed legacy SVGs so that
// scripts/mermaid/generate.mjs --verify passes without re-rendering.
//
// This is intended as a one-time migration to aid the transition from
// legacy naming (docs__path__mermaid_N.svg) to the new hashed scheme and
// metadata embedding. It computes the expected metadata from the current
// Markdown sources and injects it as an HTML comment after the opening
// <svg> tag of any existing SVG.
//
// Usage:
//   node scripts/mermaid/backfill_meta.mjs [--all] [file1.md ...]
//
// Notes:
// - Does NOT render or change diagram geometry/content; it only adds the
//   metadata comment used by the verifier.
// - Embeds CLI version from MERMAID_CLI_VERSION or defaults to 10.9.0
//   (keep in sync with scripts/pins.sh and CI).

import { promises as fs } from 'fs';
import path from 'path';
import { execSync } from 'child_process';
import { createHash } from 'crypto';

const rawArgs = process.argv.slice(2);
const flags = new Set();
const filesArg = [];
for (const a of rawArgs) {
  if (a.startsWith('-')) flags.add(a); else filesArg.push(a);
}
const scanAll = flags.has('--all') || filesArg.length === 0;

const MERMAID_RE = /```mermaid\s+([\s\S]*?)```/g;

function sha256(s) {
  return createHash('sha256').update(s).digest('hex');
}

async function findRepoRoot(startDir) {
  let dir = startDir;
  for (let i = 0; i < 15; i++) {
    try {
      const st = await fs.stat(path.join(dir, '.git')).catch(() => null);
      if (st && (st.isDirectory() || st.isFile())) return dir;
    } catch {}
    const parent = path.dirname(dir);
    if (parent === dir) break;
    dir = parent;
  }
  return startDir;
}

function outNameLegacy(repoRoot, mdPath, index) {
  const rel = path.relative(repoRoot, mdPath).replace(/[\\/]/g, '__').replace(/\.md$/i, '');
  return `${rel}__mermaid_${index}.svg`;
}

function findSvgOpenTag(text) {
  const start = text.indexOf('<svg');
  if (start === -1) return null;
  let i = start + 4;
  let inQuote = false;
  let quoteChar = '';
  while (i < text.length) {
    const ch = text[i];
    if (inQuote) {
      if (ch === quoteChar) inQuote = false;
    } else {
      if (ch === '"' || ch === "'") { inQuote = true; quoteChar = ch; }
      else if (ch === '>') break;
    }
    i++;
  }
  const end = i + 1;
  const tag = text.slice(start, end);
  return { tag, start, end };
}

async function embedMeta(svgPath, repoRoot, mdPath, index, code, cliVer) {
  const meta = {
    src: path.relative(repoRoot, mdPath).split(path.sep).join('/'),
    index,
    code_sha256: sha256(code),
    cli: cliVer,
    gen: 'v1',
  };
  let text = await fs.readFile(svgPath, 'utf8');
  const found = findSvgOpenTag(text);
  const comment = `<!-- mermaid-meta: ${JSON.stringify(meta)} -->\n`;
  if (found && found.tag) {
    text = text.slice(0, found.end) + comment + text.slice(found.end);
  } else {
    text = comment + text;
  }
  if (!text.endsWith('\n')) text += '\n';
  await fs.writeFile(svgPath, text, 'utf8');
}

async function listMarkdownFiles(repoRoot) {
  if (!scanAll) {
    return filesArg.filter(f => f.toLowerCase().endsWith('.md'));
  }
  const out = execSync("git ls-files -- '*.md'", { encoding: 'utf8', cwd: repoRoot });
  return out.split(/\r?\n/).filter(Boolean);
}

async function main() {
  const repoRoot = await findRepoRoot(process.cwd());
  const outDir = path.join(repoRoot, 'docs', 'diagrams', 'generated');
  const mdFiles = await listMarkdownFiles(repoRoot);
  const cliVer = process.env.MERMAID_CLI_VERSION || '10.9.0';
  let updated = 0;
  let missing = 0;
  for (const mdPath of mdFiles) {
    let text;
    try { text = await fs.readFile(mdPath, 'utf8'); } catch { continue; }
    MERMAID_RE.lastIndex = 0;
    let m; let idx = 0;
    while ((m = MERMAID_RE.exec(text)) !== null) {
      idx += 1;
      const code = (m[1].trim() + '\n');
      const legacy = path.join(outDir, outNameLegacy(repoRoot, mdPath, idx));
      try {
        await fs.access(legacy);
      } catch {
        missing += 1;
        continue;
      }
      await embedMeta(legacy, repoRoot, mdPath, idx, code, cliVer);
      updated += 1;
    }
  }
  console.log(`Backfilled mermaid-meta into ${updated} SVG(s). Missing legacy outputs for ${missing} block(s) were skipped.`);
}

main().catch((err) => {
  console.error(err && err.stack ? err.stack : String(err));
  process.exit(1);
});

