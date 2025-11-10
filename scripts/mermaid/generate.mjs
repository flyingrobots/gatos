#!/usr/bin/env node
import { promises as fs } from 'fs';
import path from 'path';
import os from 'os';
import { spawn } from 'child_process';

const repoRoot = process.cwd();
const outDir = path.join(repoRoot, 'docs', 'diagrams', 'generated');

// Simple argv parser: flags first, then files
const rawArgs = process.argv.slice(2);
const flags = new Set();
const cliFiles = [];
for (const a of rawArgs) {
  if (a.startsWith('-')) flags.add(a);
  else cliFiles.push(a);
}
const scanAll = flags.has('--all');

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
  const { execSync } = await import('child_process');
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
  const argsNpx = ['-y', '@mermaid-js/mermaid-cli', '-i', tmpIn, '-o', task.outFile, '-e', 'svg', '-t', 'default', '-p', puppetCfg];
  if (await hasLocal(mmdcPath)) {
    await run(mmdcPath, argsLocal);
  } else {
    await run('npx', argsNpx);
  }
}

async function main() {
  await ensureDir(outDir);
  const mdFiles = await listMarkdownFiles();
  if (mdFiles.length === 0) {
    console.log('No Markdown files specified; skipping Mermaid generation.');
    return;
  }
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
