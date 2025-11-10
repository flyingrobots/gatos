#!/usr/bin/env node
import { promises as fs } from 'fs';
import path from 'path';
import os from 'os';
import { spawn, execSync } from 'child_process';

const repoRoot = process.cwd();
const outDir = path.join(repoRoot, 'docs', 'diagrams', 'generated');

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
  const out = execSync("git ls-files -- '*.md'", { encoding: 'utf8' });
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
  const openTagMatch = text.match(/<svg\b[^>]*>/i);
  if (openTagMatch) {
    const openTag = openTagMatch[0];
    const vb = openTag.match(/viewBox\s*=\s*"\s*0\s+0\s+([0-9.]+)\s+([0-9.]+)\s*"/i);
    if (vb) {
      const w = vb[1];
      const h = vb[2];
      let newTag = openTag
        .replace(/\swidth\s*=\s*"[^"]*"/i, '')
        .replace(/style\s*=\s*"([^"]*)"/i, (m, style) => {
          const cleaned = style
            .replace(/max-width\s*:\s*[^;]+;?/i, '')
            .trim()
            .replace(/^;|;$/g, '');
          return cleaned ? ` style="${cleaned}"` : '';
        });
      if (!/preserveAspectRatio=/i.test(newTag)) {
        newTag = newTag.replace(/<svg\b/i, '<svg preserveAspectRatio="xMidYMid meet"');
      }
      newTag = newTag.replace(/<svg\b/i, `<svg width="${w}" height="${h}"`);
      if (newTag !== openTag) {
        text = text.replace(openTag, newTag);
        changed = true;
      }
    }
  }
  if (!text.endsWith('\n')) { text += '\n'; changed = true; }
  if (changed) await fs.writeFile(svgPath, text, 'utf8');
}

async function main() {
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

  let inFlight = 0; let idx = 0; let completed = 0; const total = tasks.length;
  const next = () => {
    if (idx >= total) return Promise.resolve();
    const t = tasks[idx++];
    inFlight++;
    return renderTask(t, mmdcPath)
      .then(() => { completed++; })
      .finally(() => { inFlight--; });
  };

  const runners = Array.from({ length: maxParallel }, async () => {
    while (idx < total) {
      await next();
    }
  });

  await Promise.all(runners);
  console.log(`Generated ${completed}/${total} mermaid diagram(s) into ${path.relative(repoRoot, outDir)} (parallel=${maxParallel})`);
}

main().catch((err) => {
  console.error(err && err.stack ? err.stack : (err && err.message) || err);
  process.exit(1);
});
