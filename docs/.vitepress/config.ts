import { defineConfig } from 'vitepress'
import fs from 'node:fs'
import path from 'node:path'

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
      const svgRel = `/diagrams/generated/${safe}__mermaid_${count}.svg`
      const svgAbs = path.join(process.cwd(), 'docs', svgRel.replace(/^\//, ''))
      if (fs.existsSync(svgAbs)) {
        return `<figure><img src="${svgRel}" alt="diagram ${count}" loading="lazy"/></figure>\n`
      }
    }
    return defaultFence ? defaultFence(tokens, idx, options, env, self) : self.renderToken(tokens, idx, options)
  }
}

export default defineConfig({
  title: 'GATOS',
  description: 'Git As The Operating Surface',
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
  }
})

