import { createHash } from 'node:crypto'
import { pagefindPlugin } from 'vitepress-plugin-pagefind'
import fs from 'node:fs'
import fsp from 'node:fs/promises'
import path from 'node:path'

// Site base for GitHub Pages. For project pages at
// https://<org>.github.io/<repo>/ the base should be '/<repo>/'
// Default to '/gatos/' (derived from repo name). Override in CI with DOCS_BASE
// e.g. '/gatos/pr-preview/pr-123/'.
const SITE_BASE = (process.env.DOCS_BASE || '/gatos/').replace(/([^/])$/, '$1/')

async function chapterItemsSafe(): Promise<{ text: string; link: string }[]> {
  const guideDir = path.join(process.cwd(), 'docs', 'guide')
  try {
    const entries = await fsp.readdir(guideDir)
    const files = entries.filter((f) => /^CHAPTER-\d{3}\.md$/.test(f)).sort()
    const items: { text: string; link: string }[] = []
    for (const f of files) {
      const p = path.join(guideDir, f)
      let title = f.replace(/\.md$/, '')
      try {
        const md = await fsp.readFile(p, 'utf-8')
        const m = md.match(/^#\s+(.+)$/m)
        if (m && m[1]) title = m[1]
      } catch (e) {
        // Non-fatal: skip or use fallback title
      }
      items.push({ text: title, link: `/guide/${f.replace(/\.md$/, '')}` })
    }
    return items
  } catch (err) {
    // Non-fatal: log once and continue with empty sidebar
    console.warn('[vitepress] chapterItemsSafe: unable to enumerate docs/guide:', err && (err as any).message || err)
    return []
  }
}

function mermaidToImg(md: any) {
  const defaultFence = md.renderer.rules.fence?.bind(md)
  const counters = new Map<string, number>()
  // Pre-scan diagrams directory once to avoid per-diagram fs.existsSync calls
  const diagramsDir = path.join(process.cwd(), 'docs', 'diagrams', 'generated')
  const existingSvgNames = new Set<string>()
  try {
    if (fs.existsSync(diagramsDir)) {
      for (const ent of fs.readdirSync(diagramsDir, { withFileTypes: true })) {
        if (ent.isFile() && ent.name.endsWith('.svg')) existingSvgNames.add(ent.name)
      }
    }
  } catch {
    // Non-fatal: leave the set empty
  }
  md.renderer.rules.fence = (tokens: any[], idx: number, options: any, env: any, self: any) => {
    const token = tokens[idx]
    const info = (token.info || '').trim()
    if (info.startsWith('mermaid')) {
      const rel = (env.relativePath || env.path || 'unknown') as string
      const count = (counters.get(rel) || 0) + 1
      counters.set(rel, count)
      const relPosix = `docs/${rel.replace(/\\/g, '/')}`
      const safeStem = relPosix.replace(/\.md$/i, '').replace(/[^A-Za-z0-9._-]/g, '_')
      const hash = createHash('sha256').update(relPosix).digest('hex').slice(0, 10)
      // Prefer hashed scheme; fall back to legacy (no hash) while transitioning
      const hashedBase = `diagrams/generated/${safeStem.split('/').join('__')}__${hash}__mermaid_${count}`
      const legacyBase = `diagrams/generated/${rel.replace(/\\/g, '/').replace(/\.md$/i, '').split('/').join('__')}__mermaid_${count}`
      const candidates = [hashedBase, legacyBase]
      let chosenBase = ''
      let pair = false
      for (const base of candidates) {
        const lightName = path.posix.basename(`${base}-light.svg`)
        const darkName = path.posix.basename(`${base}-dark.svg`)
        const plainName = path.posix.basename(`${base}.svg`)
        if (existingSvgNames.has(lightName) && existingSvgNames.has(darkName)) { chosenBase = base; pair = true; break }
        if (existingSvgNames.has(plainName)) { chosenBase = base; pair = false; break }
      }
      // Build relative URLs from the current markdown file to the generated asset
      // so Vite doesn't try to resolve an absolute import like 
      // "/gatos/pr-preview/pr-64/diagrams/..." during SSR. Relative links will
      // naturally inherit the site base at runtime (e.g., PR preview paths).
      const url = (p: string) => {
        const from = path.posix.join('docs', path.posix.dirname(rel))
        const to = path.posix.join('docs', p)
        let relUrl = path.posix.relative(from, to).replace(/^\.\//, '')
        // Ensure it remains a relative URL (so Vite doesn't treat it as root).
        if (!/^[.]{1,2}\//.test(relUrl)) relUrl = `./${relUrl}`
        return relUrl
      }
      if (chosenBase) {
        if (pair) {
          return `<figure><picture><source srcset="${url(chosenBase + '-dark.svg')}" media="(prefers-color-scheme: dark)"><img src="${url(chosenBase + '-light.svg')}" alt="diagram ${count}" loading="lazy"></picture></figure>\n`
        } else {
          return `<figure><img src="${url(chosenBase + '.svg')}" alt="diagram ${count}" loading="lazy"/></figure>\n`
        }
      }
    }
    return defaultFence ? defaultFence(tokens, idx, options, env, self) : self.renderToken(tokens, idx, options)
  }
}

export default async () => ({
  title: 'GATOS',
  description: 'Git As The Operating Surface',
  base: SITE_BASE,
  lastUpdated: true,
  themeConfig: {
    nav: [
      { text: 'Book', link: '/guide/CHAPTER-001' },
      { text: 'SPEC', link: '/SPEC' },
      { text: 'TECH-SPEC', link: '/TECH-SPEC' },
      { text: 'ADRs', link: '/decisions/README' }
    ],
    sidebar: {
      '/guide/': [
        { text: 'The GATOS Book', items: await chapterItemsSafe() }
      ],
      '/': [
        {
          text: 'Reference',
          items: [
            { text: 'SPEC', link: '/SPEC' },
            { text: 'TECH-SPEC', link: '/TECH-SPEC' },
            { text: 'ADRs', link: '/decisions/README' }
          ]
        }
      ]
    },
    outline: 'deep',
    socialLinks: [
      { icon: 'github', link: 'https://github.com/flyingrobots/gatos' }
    ]
  },
  markdown: {
    config: (md) => {
      mermaidToImg(md)
    }
  },
  vite: {
    plugins: [
      pagefindPlugin({
        // default options are fine; tweak here if needed
      }) as any
    ]
  }
})
