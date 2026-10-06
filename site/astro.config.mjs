import { defineConfig } from 'astro/config'

export default defineConfig({
  site: 'https://www.trunade.org',
  markdown: {
    shikiConfig: { theme: 'monokai' },
  },
})
