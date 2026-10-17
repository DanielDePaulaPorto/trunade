/*
 * Sätteri mdast plugin: after every ````markdown preview=ID block, the picture
 * of that Markdown in the real Trunade! page (public/previews/<lang>/ID.png,
 * made by `pnpm previews` in web/). Without a picture, nothing is added.
 */
import { existsSync, readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'

const PREVIEWS = fileURLToPath(new URL('../public/previews/', import.meta.url))
const CAPTION = { pt: 'Na página do Trunade!', en: 'On the Trunade! page' }

function sizes(lang) {
  const file = `${PREVIEWS}${lang}/sizes.json`
  return existsSync(file) ? JSON.parse(readFileSync(file, 'utf8')) : {}
}

function language(fileURL) {
  return fileURL && /[\\/]pages[\\/]en[\\/]/.test(fileURLToPath(fileURL)) ? 'en' : 'pt'
}

export const previews = {
  name: 'trunade-previews',
  code(node, ctx) {
    const id = /preview=([\w-]+)/.exec(node.meta ?? '')?.[1]
    if (!id) return
    const lang = language(ctx.fileURL)
    const size = sizes(lang)[id]
    if (!size) return
    ctx.insertAfter(node, {
      raw: `<figure class="preview"><figcaption>${CAPTION[lang]}</figcaption><img src="/previews/${lang}/${id}.png" width="${size.width}" height="${size.height}" alt="" loading="lazy"></figure>`,
      mdxExpressions: false,
    })
  },
}
