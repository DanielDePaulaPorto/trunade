// The Rust server serves the generated static app; there is no Node server at runtime.
export default defineNuxtConfig({
  compatibilityDate: '2026-10-01',
  ssr: false,
  devtools: { enabled: false },
  telemetry: false,
  css: [
    '@fontsource-variable/jetbrains-mono',
    'katex/dist/katex.min.css',
    '@xterm/xterm/css/xterm.css',
    '~/assets/main.css',
  ],
  app: {
    head: {
      title: 'Trunade!',
      meta: [{ name: 'viewport', content: 'width=device-width, initial-scale=1' }],
      link: [
        { rel: 'icon', type: 'image/svg+xml', href: '/favicon.svg' },
        { rel: 'alternate icon', href: '/favicon.ico' },
        { rel: 'apple-touch-icon', href: '/apple-touch-icon.png' },
      ],
    },
  },
  runtimeConfig: {
    public: {
      // In `pnpm dev` the page comes from Nuxt (port 3000) but the backend is the
      // Rust server, started with `--dev-origin http://localhost:3000`.
      backendUrl: '',
    },
  },
})
