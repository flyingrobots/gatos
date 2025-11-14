#!/usr/bin/env node
import { promises as fs } from 'fs';
import path from 'path';
import os from 'os';
import { spawn, execSync } from 'child_process';
import { createHash } from 'crypto';
import { fileURLToPath } from 'url';

// Resolved at runtime inside main(); declared here so helpers can reference them.
let repoRoot; // absolute path to repository root
let outDir;   // docs/diagrams/generated under repoRoot

// Simple argv parser: flags first, then files
const rawArgs = process.argv.slice(2);
const allowedFlags = new Set(['--all', '--verify', '-h', '--help']);
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
const verifyOnly = flags.has('--verify');

function usage() {
  const lines = [
    'Usage:',
    '  node scripts/mermaid/generate.mjs [--verify] [--all] [file1.md file2.md ...]',
    '',
    'Options:',
    '  --all         Scan all tracked .md files via git ls-files',
    '  --verify      Do not render; verify committed SVGs are up-to-date (metadata + tool pins)',
    '  -h, --help    Show this help and exit',
    '',
    'Environment:',
    '  MERMAID_MAX_PARALLEL   Concurrency (default: min(cpu, 8))',
    '  MERMAID_CLI_VERSION    @mermaid-js/mermaid-cli version (default: from scripts/pins.sh, else 10.9.0)',
    '  MERMAID_CLI_PREV_ALLOW Allow this older CLI version in verify (transitional)',
    '  MERMAID_SVG_INTRINSIC_DIM  Set to 0 to disable SVG intrinsic-size normalization',
    '  MERMAID_CMD_TIMEOUT_MS Command timeout for mmdc/npx child processes (default: 120000)',
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

// Match mermaid code fences in both multi-line and single-line forms:
//   ```mermaid\n...``` and ```mermaid ...```
// Accept one-or-more whitespace after the fence label rather than requiring a newline.
const MERMAID_RE = /```mermaid\s+([\s\S]*?)```/g;

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

function run(cmd, args, opts = {}, timeoutMs = 120000) {
  return new Promise((resolve, reject) => {
    let done = false;
    const child = spawn(cmd, args, { stdio: ['ignore', 'inherit', 'inherit'], ...opts });
    const timer = setTimeout(() => {
      if (done) return;
      // Try graceful then forceful termination
      child.kill('SIGTERM');
      setTimeout(() => child.kill('SIGKILL'), 5000);
      done = true;
      reject(new Error(`${cmd} timed out after ${timeoutMs}ms`));
    }, timeoutMs);
    child.on('exit', (code) => {
      if (done) return;
      done = true;
      clearTimeout(timer);
      if (code === 0) resolve();
      else reject(new Error(`${cmd} exited with code ${code}`));
    });
  });
}

function outNameFor(mdPath, index) {
  // Use repo-relative POSIX-style path to compute a deterministic short hash
  const relPosix = path.relative(repoRoot, mdPath).split(path.sep).join('/');
  const safeStem = relPosix.replace(/\.md$/i, '').replace(/[^A-Za-z0-9._-]/g, '_');
  const hash = createHash('sha256').update(relPosix).digest('hex').slice(0, 10);
  return `${safeStem}__${hash}__mermaid_${index}.svg`;
}

// Legacy filename scheme (pre-hash): replace path separators with "__"
function outNameLegacy(mdPath, index) {
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
      try {
        await fs.access(f);
        files.push(f);
      } catch (e) {
        const msg = e && e.message ? e.message : String(e);
        console.warn(`[mermaid] warning: skipping missing/inaccessible file: ${f} (${msg})`);
      }
    }
    return files;
  }
  // --all: use git-tracked files for reproducibility
  try {
    const out = execSync("git ls-files -- '*.md'", { encoding: 'utf8', cwd: repoRoot });
    return out.split(/\r?\n/).filter(Boolean);
  } catch (e) {
    const msg = e && e.message ? e.message : String(e);
    throw new Error(
      `Failed to list markdown files via 'git ls-files'; ensure git is installed and this is a git repository (cwd=${repoRoot}).\nUnderlying error: ${msg}`
    );
  }
}

async function collectRenderTasks(mdFiles) {
  const tasks = [];
  const seen = new Set();
  for (const mdPath of mdFiles) {
    let text;
    try {
      text = await fs.readFile(mdPath, 'utf8');
    } catch (e) {
      const msg = e && e.message ? e.message : String(e);
      console.error(`[mermaid] error: failed to read ${mdPath}: ${msg}`);
      continue; // skip unreadable file, continue with the rest
    }
    // Reset regex index for each new file to avoid cross-file state with /g
    MERMAID_RE.lastIndex = 0;
    let match; let idx = 0;
    while ((match = MERMAID_RE.exec(text)) !== null) {
      idx += 1;
      const code = match[1].trim() + '\n';
      const outFile = path.join(outDir, outNameFor(mdPath, idx));
      if (seen.has(outFile)) {
        throw new Error(`Output name collision detected for ${mdPath} block #${idx}: ${outFile}`);
      }
      seen.add(outFile);
      tasks.push({ mdPath, index: idx, code, outFile });
    }
  }
  return tasks;
}

async function hasLocal(cmdPath) {
  try { await fs.access(cmdPath); return true; } catch { return false; }
}

async function ensureLocalMmdc() {
  const cliVer = await resolveMermaidCliVersion();
  const bin = binPath('mmdc');
  if (await hasLocal(bin)) return bin;
  await run('npm', ['i', '--no-save', `@mermaid-js/mermaid-cli@${cliVer}`], { cwd: repoRoot }, 300000);
  return bin;
}

async function renderTask(task, mmdcPath) {
  const tmpDir = await fs.mkdtemp(path.join(os.tmpdir(), 'gatos-mmd-'));
  try {
    const tmpIn = path.join(tmpDir, 'in.mmd');
    const tmpOut = path.join(tmpDir, 'out.svg');
    await fs.writeFile(tmpIn, task.code, 'utf8');
    const puppetCfg = path.join(repoRoot, 'scripts', 'mermaid', 'puppeteer.json');
    // Validate puppeteer config exists to surface clear errors if missing
    try {
      await fs.access(puppetCfg);
    } catch {
      throw new Error(`Puppeteer config not found: ${puppetCfg}`);
    }
    const argsLocal = ['-i', tmpIn, '-o', tmpOut, '-e', 'svg', '-t', 'default', '-p', puppetCfg];
    // Use MERMAID_CLI_VERSION from env or scripts/pins.sh (if present) for reproducibility.
    const cliVer = await resolveMermaidCliVersion();
    const argsNpx = ['-y', `@mermaid-js/mermaid-cli@${cliVer}`, '-i', tmpIn, '-o', tmpOut, '-e', 'svg', '-t', 'default', '-p', puppetCfg];
    // Parse and validate timeout from env (ms). Fall back to 120000 on any invalid value.
    const envTimeout = process.env.MERMAID_CMD_TIMEOUT_MS;
    const DEFAULT_TIMEOUT = 120000; // 2 minutes
    const MIN_TIMEOUT = 10000;      // 10 seconds (floor)
    const MAX_TIMEOUT = 900000;     // 15 minutes (ceiling)
    let validated = DEFAULT_TIMEOUT;
    if (typeof envTimeout === 'string' && envTimeout.trim() !== '') {
      const parsed = Number.parseInt(envTimeout, 10);
      if (Number.isFinite(parsed) && Number.isSafeInteger(parsed) && parsed >= MIN_TIMEOUT && parsed <= MAX_TIMEOUT) {
        validated = parsed;
      }
    }
    const timeoutMs = Math.max(MIN_TIMEOUT, validated);
    if (await hasLocal(mmdcPath)) {
      await run(mmdcPath, argsLocal, {}, timeoutMs);
    } else {
      try {
        await run('npx', argsNpx, {}, timeoutMs);
      } catch (e) {
        const msg = String(e && e.message || e || '');
        // Retry by installing locally if npx failed (seen as exit code 126 in CI under concurrency)
        if (msg.includes('exited with code 126') || msg.includes('exited with code')) {
          const local = await ensureLocalMmdc().catch(() => null);
          if (local) {
            await run(local, argsLocal, {}, timeoutMs);
            return;
          }
        }
        throw e;
      }
    }
    // Post-process the tmp output file after temp cleanup
    if ((process.env.MERMAID_SVG_INTRINSIC_DIM || '1') !== '0') {
      await normalizeSvgIntrinsicSize(tmpOut);
    }
    await embedMeta(tmpOut, task, cliVer);

    // If an existing file already matches the expected metadata, keep it to avoid noisy diffs in CI.
    // Otherwise, replace it with the freshly generated svg.
    let keepExisting = false;
    try {
      const existing = await fs.readFile(task.outFile, 'utf8');
      const meta = extractMeta(existing);
      if (meta && meta.src === path.relative(repoRoot, task.mdPath).split(path.sep).join('/') && meta.index === task.index && meta.code_sha256 === sha256(task.code) && meta.cli === cliVer) {
        keepExisting = true;
      }
    } catch {}
    if (!keepExisting) {
      await fs.mkdir(path.dirname(task.outFile), { recursive: true });
      await fs.copyFile(tmpOut, task.outFile);
    }
  } finally {
    // Clean up temporary directory regardless of success/failure
    try { await fs.rm(tmpDir, { recursive: true, force: true }); } catch {}
  }
}

// Ensure Quick Look and other viewers render at intrinsic size rather than a huge canvas.
// Parse and rewrite the opening <svg …> tag using a small attribute parser (avoids fragile regexes).
async function normalizeSvgIntrinsicSize(svgPath) {
  let text = await fs.readFile(svgPath, 'utf8');
  const { tag, start, end } = findSvgOpenTag(text) || {};
  if (!tag) {
    if (!text.endsWith('\n')) await fs.writeFile(svgPath, text + '\n', 'utf8');
    return;
  }
  const attrs = parseSvgAttributes(tag);
  const vb = attrs.get('viewBox');
  if (!vb) {
    if (!text.endsWith('\n')) await fs.writeFile(svgPath, text + '\n', 'utf8');
    return;
  }
  const parts = vb.trim().split(/\s+/).map(Number);
  if (parts.length !== 4 || Number.isNaN(parts[2]) || Number.isNaN(parts[3])) {
    if (!text.endsWith('\n')) await fs.writeFile(svgPath, text + '\n', 'utf8');
    return;
  }
  const width = String(parts[2]);
  const height = String(parts[3]);

  // Remove width; set width/height; clean style's max-width; ensure preserveAspectRatio
  attrs.delete('width');
  attrs.set('width', width);
  attrs.set('height', height);

  if (attrs.has('style')) {
    const style = attrs.get('style') || '';
    const props = style.split(';').map(s => s.trim()).filter(Boolean);
    const kept = props.filter(p => !/^max-width\s*:/i.test(p));
    if (kept.length) attrs.set('style', kept.join(';'));
    else attrs.delete('style');
  }
  if (!attrs.has('preserveAspectRatio')) attrs.set('preserveAspectRatio', 'xMidYMid meet');

  const newTag = buildSvgOpenTag(attrs);
  if (newTag !== tag) {
    text = text.slice(0, start) + newTag + text.slice(end);
  }
  if (!text.endsWith('\n')) text += '\n';
  await fs.writeFile(svgPath, text, 'utf8');
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

function parseSvgAttributes(tag) {
  // tag is like: <svg attr1="..." attr2='...'>
  const inside = tag.replace(/^<svg\s*/i, '').replace(/>\s*$/, '');
  const attrs = new Map();
  let i = 0;
  const len = inside.length;
  while (i < len) {
    // skip whitespace
    while (i < len && /\s/.test(inside[i])) i++;
    if (i >= len) break;
    // read name
    let name = '';
    while (i < len && /[^=\s]/.test(inside[i])) { name += inside[i++]; }
    name = name.trim();
    // skip whitespace
    while (i < len && /\s/.test(inside[i])) i++;
    let value = '';
    if (i < len && inside[i] === '=') {
      i++;
      while (i < len && /\s/.test(inside[i])) i++;
      if (i < len && (inside[i] === '"' || inside[i] === "'")) {
        const q = inside[i++];
        const start = i;
        while (i < len && inside[i] !== q) i++;
        value = inside.slice(start, i);
        i++; // skip closing quote
      } else {
        // unquoted value
        const start = i;
        while (i < len && !/\s/.test(inside[i])) i++;
        value = inside.slice(start, i);
      }
    }
    if (name) attrs.set(name, value);
  }
  return attrs;
}

function buildSvgOpenTag(attrs) {
  // Keep original order where possible: we don't track it here, but ensure width/height appear first for readability
  const ordered = new Map();
  if (attrs.has('width')) ordered.set('width', attrs.get('width'));
  if (attrs.has('height')) ordered.set('height', attrs.get('height'));
  if (attrs.has('preserveAspectRatio')) ordered.set('preserveAspectRatio', attrs.get('preserveAspectRatio'));
  if (attrs.has('style')) ordered.set('style', attrs.get('style'));
  for (const [k, v] of attrs.entries()) {
    if (!ordered.has(k)) ordered.set(k, v);
  }
  const parts = [];
  for (const [k, v] of ordered.entries()) {
    if (v === '') parts.push(k); else parts.push(`${k}="${v}"`);
  }
  return `<svg ${parts.join(' ')}>`;
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

  // Warm up a local mmdc to avoid repeated concurrent npx executions which can
  // occasionally fail under CI with exit 126 due to cache races. When the
  // local binary is present we always prefer it; otherwise we fall back to npx.
  let mmdcPath = binPath('mmdc');
  try {
    const cliVer = await resolveMermaidCliVersion();
    if (!(await hasLocal(mmdcPath))) {
      await run('npm', ['i', '--no-save', `@mermaid-js/mermaid-cli@${cliVer}`], { cwd: repoRoot }, 300000);
    }
  } catch (e) {
    // Do not fail generation if warmup install fails; we'll fall back to npx.
  }
  mmdcPath = binPath('mmdc');
  if (verifyOnly) {
    const errors = await verifyTasks(tasks);
    if (errors.length) {
      console.error('Verification failed for the following diagrams:');
      for (const e of errors) console.error(' - ' + e);
      process.exit(2);
    }
    console.log(`Verified ${tasks.length} diagrams up-to-date.`);
    return;
  }
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

function sha256(s) {
  return createHash('sha256').update(s).digest('hex');
}

async function embedMeta(svgPath, task, cliVer) {
  // Check for existing metadata and warn if inconsistent before writing
  try {
    const existing = await fs.readFile(svgPath, 'utf8');
    const existingMeta = extractMeta(existing);
    if (existingMeta) {
      const expectedSrc = path.relative(repoRoot, task.mdPath).split(path.sep).join('/');
      if (existingMeta.src !== expectedSrc || existingMeta.index !== task.index) {
        console.warn(
          `[meta] ${path.relative(repoRoot, svgPath)}: replacing inconsistent metadata ` +
          `(had src=${existingMeta.src} index=${existingMeta.index}, want src=${expectedSrc} index=${task.index})`
        );
      }
    }
  } catch {
    // File may not exist yet; proceed
  }
  const meta = {
    src: path.relative(repoRoot, task.mdPath).split(path.sep).join('/'),
    index: task.index,
    code_sha256: sha256(task.code),
    cli: cliVer,
    gen: 'v1'
  };
  let text = await fs.readFile(svgPath, 'utf8');
  const { tag, start, end } = findSvgOpenTag(text) || {};
  const comment = `<!-- mermaid-meta: ${JSON.stringify(meta)} -->\n`;
  if (tag) {
    text = text.slice(0, end) + comment + text.slice(end);
  } else {
    text = comment + text;
  }
  await fs.writeFile(svgPath, text.endsWith('\n') ? text : text + '\n', 'utf8');
}

function extractMeta(svgText) {
  // Robustly scan for HTML comments and extract the one starting with 'mermaid-meta:'
  // Avoid a single fragile regex that can fail with multiple comments or line breaks.
  const comments = [];
  for (let i = 0; i < svgText.length; ) {
    const start = svgText.indexOf('<!--', i);
    if (start === -1) break;
    const end = svgText.indexOf('-->', start + 4);
    if (end === -1) break; // malformed; stop scanning
    const body = svgText.slice(start + 4, end).trim();
    comments.push(body);
    i = end + 3;
  }
  const metaComment = comments.find(c => c.trim().startsWith('mermaid-meta:'));
  if (!metaComment) return null;
  const payload = metaComment.slice(metaComment.indexOf(':') + 1).trim();
  // payload is expected to be a JSON object. Parse defensively.
  try {
    return JSON.parse(payload);
  } catch {
    return null;
  }
}

async function verifyTasks(tasks) {
  const errs = [];
  for (const t of tasks) {
    const hashedPath = t.outFile;
    const legacyPath = path.join(outDir, outNameLegacy(t.mdPath, t.index));
    const relHashed = path.relative(repoRoot, hashedPath).split(path.sep).join('/');
    const relLegacy = path.relative(repoRoot, legacyPath).split(path.sep).join('/');
    try {
      let svg, usedLegacy = false;
      try {
        svg = await fs.readFile(hashedPath, 'utf8');
      } catch (e) {
        // Fallback to legacy filename during transition
        try {
          svg = await fs.readFile(legacyPath, 'utf8');
          usedLegacy = true;
        } catch {
          throw e; // rethrow original ENOENT
        }
      }
      const meta = extractMeta(svg);
      if (!meta) {
        errs.push(`${usedLegacy ? relLegacy : relHashed}: missing mermaid-meta comment`);
        continue;
      }
      // Validate source and index match expectations
      const expectedSrc = path.relative(repoRoot, t.mdPath).split(path.sep).join('/');
      if (meta.src !== expectedSrc) {
        errs.push(`${usedLegacy ? relLegacy : relHashed}: src mismatch (have ${meta.src}, want ${expectedSrc})`);
      }
      if (meta.index !== t.index) {
        errs.push(`${usedLegacy ? relLegacy : relHashed}: index mismatch (have ${meta.index}, want ${t.index})`);
      }
      const codeHash = sha256(t.code);
      if (meta.code_sha256 !== codeHash) {
        // Include a short snippet (first few lines, truncated) to aid debugging
        const firstLines = t.code.split('\n').slice(0, 8).join('\n');
        const truncated = (firstLines.length > 400 ? firstLines.slice(0, 400) + '…' : firstLines).replace(/\n/g, '\\n');
        errs.push(`${usedLegacy ? relLegacy : relHashed}: code hash mismatch (have ${meta.code_sha256}, want ${codeHash}) — snippet: ‹${truncated}›`);
      }
      const wantCli = await resolveMermaidCliVersion();
      const prevAllow = process.env.MERMAID_CLI_PREV_ALLOW || '';
      if (meta.cli !== wantCli) {
        if (!prevAllow || meta.cli !== prevAllow) {
          errs.push(`${usedLegacy ? relLegacy : relHashed}: cli version mismatch (have ${meta.cli}, want ${wantCli}${prevAllow ? ` or ${prevAllow}` : ''})`);
        }
      }
    } catch (e) {
      const msg = e && e.message ? e.message : String(e);
      // Provide both expected names to aid developer
      errs.push(`${relHashed} (or ${relLegacy}): ${msg}`);
    }
  }
  return errs;
}

main().catch((err) => {
  console.error(err && err.stack ? err.stack : (err && err.message) || err);
  process.exit(1);
});

// Attempt to read MERMAID_CLI_VERSION default from scripts/pins.sh to avoid drift.
async function resolveMermaidCliVersion() {
  if (process.env.MERMAID_CLI_VERSION && process.env.MERMAID_CLI_VERSION.trim() !== '') {
    return process.env.MERMAID_CLI_VERSION.trim();
  }
  try {
    const pinsPath = path.join(repoRoot || (await findRepoRoot(process.cwd())) || process.cwd(), 'scripts', 'pins.sh');
    const txt = await fs.readFile(pinsPath, 'utf8');
    const m = txt.match(/MERMAID_CLI_VERSION\s*=\s*"([^"]+)"/);
    if (m && m[1]) return m[1];
  } catch {}
  return '10.9.0';
}
