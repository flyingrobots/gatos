#!/usr/bin/env node
import { promises as fs } from 'fs';
import path from 'path';
import os from 'os';
import { spawn, execSync } from 'child_process';
import { fileURLToPath } from 'url';

// Resolved at runtime inside main(); declared here so helpers can reference them.
let repoRoot; // absolute path to repository root
let outDir;   // docs/diagrams/generated under repoRoot

// Simple argv parser: flags first, then files
const rawArgs = process.argv.slice(2);
const allowedFlags = new Set(['--all', '-h', '--help']);
const flags = new Set();
const cliFiles = [];
const unknownFlags = [];
for (const a of rawArgs) {
  if (a.startsWith('-')) {
    if (allowedFlags.has(a)) flags.add(a);
    else unknownFlags.push(a);
  } else {
    cliFiles.push(a);
  }
}
const scanAll = flags.has('--all');

function usage() {
  const lines = [
    'Usage:',
    '  node scripts/mermaid/generate.mjs [--all] [file1.md file2.md ...]',
    '',
    'Options:',
    '  --all         Scan all tracked .md files via git ls-files',
    '  -h, --help    Show this help and exit',
    '',
    'Environment:',
    '  MERMAID_MAX_PARALLEL   Concurrency (default: min(cpu, 8))',
    '  MERMAID_CLI_VERSION    @mermaid-js/mermaid-cli version (default: 10.9.0)',
    '  MERMAID_SVG_INTRINSIC_DIM  Set to 0 to disable SVG intrinsic-size normalization',
  ];
  return lines.join('\n');
}

if (flags.has('-h') || flags.has('--help')) {
  console.log(usage());
  process.exit(0);
}

if (unknownFlags.length > 0) {
  for (const f of unknownFlags) console.error(`Unknown option: ${f}`);
  console.error('\n' + usage());
  process.exit(2);
}

const MERMAID_RE = /```mermaid\s*\n([\s\S]*?)```/g;

async function ensureDir(p) {
  await fs.mkdir(p, { recursive: true });
}

async function isRepoRoot(dir) {
  try {
    const st = await fs.stat(path.join(dir, '.git')).catch(() => null);
    if (st && (st.isDirectory() || st.isFile())) return true; // .git dir or file (worktree)
  } catch {}
  try {
    await fs.access(path.join(dir, 'package.json'));
    return true;
  } catch {}
  return false;
}

async function findRepoRoot(startDir) {
  let dir = startDir;
  for (let i = 0; i < 15; i++) {
    if (await isRepoRoot(dir)) return dir;
    const parent = path.dirname(dir);
    if (parent === dir) break;
    dir = parent;
  }
  return null;
}

function binPath(name) {
  const ext = process.platform === 'win32' ? '.cmd' : '';
  return path.join(repoRoot, 'node_modules', '.bin', name + ext);
}

function run(cmd, args, opts = {}) {
  return new Promise((resolve, reject) => {
    const child = spawn(cmd, args, { stdio: ['ignore', 'inherit', 'inherit'], ...opts });
    child.on('exit', (code) => {
      if (code === 0) resolve();
      else reject(new Error(`${cmd} exited with code ${code}`));
    });
  });
}

function outNameFor(mdPath, index) {
  const rel = path.relative(repoRoot, mdPath).replace(/[\\/]/g, '__').replace(/\.md$/i, '');
  return `${rel}__mermaid_${index}.svg`;
}

async function listMarkdownFiles() {
  if (!scanAll) {
    if (cliFiles.length === 0) {
      // Safety: if invoked with no files and without --all, do nothing.
      return [];
    }
    const files = [];
    for (const f of cliFiles) {
      if (!f.toLowerCase().endsWith('.md')) continue;
      try { await fs.access(f); files.push(f); } catch { /* skip missing */ }
    }
    return files;
  }
  // --all: use git-tracked files for reproducibility
  const out = execSync("git ls-files -- '*.md'", { encoding: 'utf8', cwd: repoRoot });
  return out.split(/\r?\n/).filter(Boolean);
}

async function collectRenderTasks(mdFiles) {
  const tasks = [];
  for (const mdPath of mdFiles) {
    const text = await fs.readFile(mdPath, 'utf8');
    let match; let idx = 0;
    while ((match = MERMAID_RE.exec(text)) !== null) {
      idx += 1;
      const code = match[1].trim() + '\n';
      const outFile = path.join(outDir, outNameFor(mdPath, idx));
      tasks.push({ mdPath, index: idx, code, outFile });
    }
  }
  return tasks;
}

async function hasLocal(cmdPath) {
  try { await fs.access(cmdPath); return true; } catch { return false; }
}

async function renderTask(task, mmdcPath) {
  const tmpDir = await fs.mkdtemp(path.join(os.tmpdir(), 'gatos-mmd-'));
  const tmpIn = path.join(tmpDir, 'in.mmd');
  await fs.writeFile(tmpIn, task.code, 'utf8');
  const puppetCfg = path.join(repoRoot, 'scripts', 'mermaid', 'puppeteer.json');
  // Validate puppeteer config exists to surface clear errors if missing
  try {
    await fs.access(puppetCfg);
  } catch {
    throw new Error(`Puppeteer config not found: ${puppetCfg}`);
  }
  const argsLocal = ['-i', tmpIn, '-o', task.outFile, '-e', 'svg', '-t', 'default', '-p', puppetCfg];
  // Pin to 10.9.0 for stable Mermaid syntax support and reproducible CI output.
  // Override via MERMAID_CLI_VERSION env var; must match CI (.github/workflows/ci.yml)
  // and docker-compose.yml (ci-diagrams service) for consistency.
  const cliVer = process.env.MERMAID_CLI_VERSION || '10.9.0';
  const argsNpx = ['-y', `@mermaid-js/mermaid-cli@${cliVer}`, '-i', tmpIn, '-o', task.outFile, '-e', 'svg', '-t', 'default', '-p', puppetCfg];
  if (await hasLocal(mmdcPath)) {
    await run(mmdcPath, argsLocal);
  } else {
    await run('npx', argsNpx);
  }
  if ((process.env.MERMAID_SVG_INTRINSIC_DIM || '1') !== '0') {
    await normalizeSvgIntrinsicSize(task.outFile);
  }
}

// Ensure Quick Look and other viewers render at intrinsic size rather than a huge canvas.
// - Remove width="100%" and style max-width:…
// - Set width/height from viewBox numbers
async function normalizeSvgIntrinsicSize(svgPath) {
  let text = await fs.readFile(svgPath, 'utf8');
  let changed = false;
  const openTagMatch = text.match(/<svg\b[^>]*>/i); // matches across newlines until first '>'
  if (!openTagMatch) {
    if (!text.endsWith('\n')) { await fs.writeFile(svgPath, text + '\n', 'utf8'); }
    return;
  }

  const openTag = openTagMatch[0];
  const start = openTagMatch.index;
  const end = start + openTag.length;

  // Parse viewBox = "minx miny width height" (accept any origin)
  const vb = openTag.match(/viewBox\s*=\s*"\s*([0-9.]+)\s+([0-9.]+)\s+([0-9.]+)\s+([0-9.]+)\s*"/i);
  if (!vb) {
    if (!text.endsWith('\n')) { await fs.writeFile(svgPath, text + '\n', 'utf8'); }
    return;
  }
  const width = vb[3];
  const height = vb[4];

  // Remove width attribute, clean style's max-width, add preserveAspectRatio if missing, and set width/height
  let newTag = openTag.replace(/\swidth\s*=\s*"[^"]*"/i, '');
  // clean style attribute safely (property list)
  newTag = newTag.replace(/style\s*=\s*"([^"]*)"/i, (m, style) => {
    const props = style.split(';').map(s => s.trim()).filter(Boolean);
    const kept = props.filter(p => !/^max-width\s*:/i.test(p));
    return kept.length ? ` style="${kept.join(';')}"` : '';
  });
  if (!/preserveAspectRatio=/i.test(newTag)) {
    newTag = newTag.replace(/<svg\b/i, '<svg preserveAspectRatio="xMidYMid meet"');
  }
  newTag = newTag.replace(/<svg\b/i, `<svg width="${width}" height="${height}"`);

  if (newTag !== openTag) {
    text = text.slice(0, start) + newTag + text.slice(end);
    changed = true;
  }

  if (!text.endsWith('\n')) { text += '\n'; changed = true; }
  if (changed) await fs.writeFile(svgPath, text, 'utf8');
}

async function main() {
  // Resolve repoRoot anchored to this script's location (robust to cwd/symlinks)
  const scriptDir = path.dirname(fileURLToPath(import.meta.url));
  const located = await findRepoRoot(scriptDir);
  if (!located) {
    console.error('[mermaid] Error: could not locate repository root from script path:', scriptDir);
    console.error('Run this script inside the repository, or adjust invocation so it can find .git.');
    process.exit(1);
  }
  repoRoot = located;
  outDir = path.join(repoRoot, 'docs', 'diagrams', 'generated');
  await ensureDir(outDir);
  if (!scanAll && cliFiles.length === 0) {
    console.error('Error: no input files provided. Pass one or more .md files or use --all.');
    console.error('\n' + usage());
    process.exit(1);
  }
  const mdFiles = await listMarkdownFiles();
  const tasks = await collectRenderTasks(mdFiles);
  if (tasks.length === 0) {
    console.log('No Mermaid code blocks found in specified files.');
    return;
  }

  const mmdcPath = binPath('mmdc');
  const maxParallel = Math.max(1, Math.min(Number(process.env.MERMAID_MAX_PARALLEL || 0) || os.cpus().length, 8));

  let idx = 0;
  let completed = 0;
  let failed = 0;
  const errors = [];

  const next = async () => {
    if (idx >= tasks.length) return;
    const t = tasks[idx++];
    try {
      await renderTask(t, mmdcPath);
      completed++;
    } catch (err) {
      failed++;
      const id = `${t.mdPath}#${t.index}`;
      const msg = (err && err.message) || String(err);
      console.error(`Mermaid render failed: ${id}: ${msg}`);
      errors.push({ id, error: msg });
    }
  };

  const runners = Array.from({ length: maxParallel }, async () => {
    while (idx < tasks.length) {
      await next();
    }
  });

  await Promise.all(runners);
  const total = tasks.length;
  console.log(`Generated ${completed}/${total} mermaid diagram(s) into ${path.relative(repoRoot, outDir)} (parallel=${maxParallel})`);
  if (failed > 0) {
    const summary = errors.map(e => ` - ${e.id}: ${e.error}`).join('\n');
    throw new Error(`Mermaid generation failed for ${failed}/${total} diagram(s):\n${summary}`);
  }
}

main().catch((err) => {
  console.error(err && err.stack ? err.stack : (err && err.message) || err);
  process.exit(1);
});
