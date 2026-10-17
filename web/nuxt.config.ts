import { fileURLToPath } from 'node:url'

// The Rust server serves the generated static app; there is no Node server at runtime.
// TRUNADE_STATIC=1 builds the variant used to export a lesson as one HTML file:
// a single JS bundle (no chunks to fetch), no Mermaid, output in .output-static.
const exportBuild = process.env.TRUNADE_STATIC === '1'

export default defineNuxtConfig({
  compatibilityDate: '2026-10-01',
  ssr: false,
  devtools: { enabled: false },
  telemetry: false,
  css: [
    '@fontsource-variable/jetbrains-mono',
    '@fontsource-variable/fira-code',
    '@fontsource-variable/inter',
    '@fontsource-variable/open-sans',
    '@fontsource/gloock',
    '@fontsource/young-serif',
    '@fontsource-variable/fraunces',
    '@fontsource-variable/cormorant-garamond',
    'katex/dist/katex.min.css',
    '@xterm/xterm/css/xterm.css',
    '~/assets/main.css',
  ],
  app: {
    head: {
      meta: [{ name: 'viewport', content: 'width=device-width, initial-scale=1' }],
      link: [
        { rel: 'icon', type: 'image/svg+xml', href: '/favicon.svg' },
        { rel: 'alternate icon', href: '/favicon.ico' },
        { rel: 'apple-touch-icon', href: '/apple-touch-icon.png' },
      ],
    },
  },
  ...(exportBuild && {
    nitro: { output: { dir: '.output-static' } },
    experimental: { appManifest: false },
    alias: { mermaid: fileURLToPath(new URL('./app/static/mermaid-stub.ts', import.meta.url)) },
    vite: {
      build: {
        cssCodeSplit: false,
        assetsInlineLimit: 0,
        rollupOptions: { output: { codeSplitting: false } },
      },
    },
  }),
  runtimeConfig: {
    public: {
      // In `pnpm dev` the page comes from Nuxt (port 3000) but the backend is the
      // Rust server, started with `--dev-origin http://localhost:3000`.
      backendUrl: '',
    },
  },
})
