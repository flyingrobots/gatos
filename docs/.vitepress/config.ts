import { pagefindPlugin } from 'vitepress-plugin-pagefind'
import fs from 'node:fs'
import path from 'node:path'

// Site base for GitHub Pages. For project pages at
// https://<org>.github.io/<repo>/ the base should be '/<repo>/'
// Default to '/gatos/' (derived from repo name). Override in CI with DOCS_BASE
// e.g. '/gatos/pr-preview/pr-123/'.
const SITE_BASE = (process.env.DOCS_BASE || '/gatos/').replace(/([^/])$/, '$1/')

function chapterItems(): { text: string; link: string }[] {
  const guideDir = path.join(process.cwd(), 'docs', 'guide')
  const files = fs
    .readdirSync(guideDir)
    .filter((f) => /^CHAPTER-\d{3}\.md$/.test(f))
    .sort()
  return files.map((f) => {
    const p = path.join(guideDir, f)
    const md = fs.readFileSync(p, 'utf-8')
    const m = md.match(/^#\s+(.+)$/m)
    const title = m ? m[1] : f.replace(/\.md$/, '')
    return { text: title, link: `/guide/${f.replace(/\.md$/, '')}` }
  })
}

function mermaidToImg(md: any) {
  const defaultFence = md.renderer.rules.fence?.bind(md)
  const counters = new Map<string, number>()
  md.renderer.rules.fence = (tokens: any[], idx: number, options: any, env: any, self: any) => {
    const token = tokens[idx]
    const info = (token.info || '').trim()
    if (info.startsWith('mermaid')) {
      const rel = (env.relativePath || env.path || 'unknown') as string
      const count = (counters.get(rel) || 0) + 1
      counters.set(rel, count)
      const safe = rel.replace(/\\/g, '/').replace(/\.md$/, '').replace(/\//g, '__')
      // Filesystem-relative base (under docs/)
      const fsBase = `diagrams/generated/${safe}__mermaid_${count}`
      const abs = (p: string) => path.join(process.cwd(), 'docs', p)
      const fsLight = `${fsBase}-light.svg`
      const fsDark = `${fsBase}-dark.svg`
      const fsPlain = `${fsBase}.svg`
      const hasLight = fs.existsSync(abs(fsLight))
      const hasDark = fs.existsSync(abs(fsDark))
      const hasPlain = fs.existsSync(abs(fsPlain))
      // Public URLs must honor the site base for GitHub Pages subpaths
      const url = (p: string) => `${SITE_BASE.replace(/\/$/, '')}/${p}`.replace(/([^:]\/)\/+/g, '$1')
      const hrefLight = url(fsLight)
      const hrefDark = url(fsDark)
      const hrefPlain = url(fsPlain)
      if (hasLight && hasDark) {
        return `<figure><picture><source srcset="${hrefDark}" media="(prefers-color-scheme: dark)"><img src="${hrefLight}" alt="diagram ${count}" loading="lazy"></picture></figure>\n`
      } else if (hasPlain) {
        return `<figure><img src="${hrefPlain}" alt="diagram ${count}" loading="lazy"/></figure>\n`
      }
    }
    return defaultFence ? defaultFence(tokens, idx, options, env, self) : self.renderToken(tokens, idx, options)
  }
}

export default {
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
        { text: 'The GATOS Book', items: chapterItems() }
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
}
