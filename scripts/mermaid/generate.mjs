#!/usr/bin/env node
import { promises as fs } from 'fs';
import path from 'path';
import os from 'os';
import { fileURLToPath } from 'url';
import { spawn } from 'child_process';

const repoRoot = process.cwd();
const outDir = path.join(repoRoot, 'docs', 'diagrams', 'generated');

async function* iterMarkdownFiles() {
  // Prefer git-tracked files for reproducibility
  const { execSync } = await import('child_process');
  let files = [];
  try {
    files = execSync("git ls-files -- '*.md'", { encoding: 'utf8' })
      .split(/\r?\n/)
      .filter(Boolean);
  } catch {
    // Fallback: recursive scan
    async function walk(dir) {
      const entries = await fs.readdir(dir, { withFileTypes: true });
      for (const e of entries) {
        const p = path.join(dir, e.name);
        if (e.isDirectory()) {
          if (['.git', 'target', '.obsidian', 'node_modules'].includes(e.name)) continue;
          yield* walk(p);
        } else if (e.isFile() && p.endsWith('.md')) {
          files.push(p);
        }
      }
    }
    for await (const _ of walk(repoRoot)) {
      void _; // noop
    }
  }
  for (const f of files) yield f;
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

async function main() {
  let mmdc = binPath('mmdc');
  await ensureDir(outDir);
  let countBlocks = 0;
  for await (const mdPath of iterMarkdownFiles()) {
    const text = await fs.readFile(mdPath, 'utf8');
    let match;
    let idx = 0;
    while ((match = MERMAID_RE.exec(text)) !== null) {
      const code = match[1].trim() + '\n';
      idx += 1;
      countBlocks += 1;
      const tmpDir = await fs.mkdtemp(path.join(os.tmpdir(), 'gatos-mmd-'));
      const tmpIn = path.join(tmpDir, 'in.mmd');
      await fs.writeFile(tmpIn, code, 'utf8');
      const outFile = path.join(outDir, outNameFor(mdPath, idx));
      let cmd, args;
      if (await fs
        .access(mmdc)
        .then(() => true)
        .catch(() => false)) {
        cmd = mmdc;
        args = ['-i', tmpIn, '-o', outFile, '-e', 'svg', '-t', 'default'];
      } else {
        // Fallback to npx without requiring repo-local deps
        cmd = 'npx';
        args = ['-y', '@mermaid-js/mermaid-cli', '-i', tmpIn, '-o', outFile, '-e', 'svg', '-t', 'default'];
      }
      await run(cmd, args);
    }
  }
  console.log(`Generated ${countBlocks} mermaid diagram(s) into ${path.relative(repoRoot, outDir)}`);
}

main().catch((err) => {
  console.error(err.message || err);
  process.exit(1);
});
