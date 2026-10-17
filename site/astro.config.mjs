import { defineConfig } from 'astro/config'
import { satteri } from '@astrojs/markdown-satteri'
import { previews } from './plugins/previews.mjs'

export default defineConfig({
  site: 'https://www.trunade.org',
  markdown: {
    shikiConfig: { theme: 'monokai' },
    processor: satteri({ mdastPlugins: [previews] }),
  },
})
