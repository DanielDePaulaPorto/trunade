//! Exports the lesson as one self-contained HTML file next to the `.md`: the
//! single-bundle build of the page (`web/.output-static`, built with
//! TRUNADE_STATIC=1), its CSS with only the fonts in use (latin subset) as
//! data URIs, and the lesson recorded by the page (cells, runs, diagrams).
//! Opened from disk it needs no server: ▶ replays the recorded runs.

use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use rust_embed::RustEmbed;
use serde_json::Value;

#[derive(RustEmbed)]
#[folder = "../web/.output-static/public"]
#[allow_missing = true]
struct ExportAssets;

const ASSETS_DIR: &str = "_nuxt/";
/// The latin subset's first range, as fontsource writes it and as minifiers rewrite it.
const LATIN_RANGES: &[&str] = &["U+0000-00FF", "U+0-FF", "U+??"];
const KATEX_FONTS: &str = "KaTeX";

/// Writes `<stem>.html` beside the document (`<stem>-1.html`… if taken) and returns its path.
pub fn write(document: &Path, lesson: &Value, fonts: &[String]) -> Result<PathBuf> {
    let html = build_html(lesson, fonts, &title(document))?;
    let (path, mut file) = create_beside(document)?;
    file.write_all(html.as_bytes())
        .with_context(|| format!("writing {}", path.display()))?;
    Ok(path)
}

fn title(document: &Path) -> String {
    document.file_stem().unwrap_or_default().to_string_lossy().into_owned()
}

fn create_beside(document: &Path) -> Result<(PathBuf, std::fs::File)> {
    let dir = document.parent().context("document has no directory")?;
    let stem = title(document);
    for attempt in 0..1000 {
        let name = match attempt {
            0 => format!("{stem}.html"),
            n => format!("{stem}-{n}.html"),
        };
        let path = dir.join(name);
        match std::fs::File::options().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error).with_context(|| format!("creating {}", path.display())),
        }
    }
    bail!("too many exported copies of {stem}.html")
}

fn build_html(lesson: &Value, fonts: &[String], title: &str) -> Result<String> {
    let index = asset_text("index.html")
        .context("the export build is missing: run `pnpm generate` in TrunadeCode/web and rebuild")?;
    let mut html = String::with_capacity(4 * 1024 * 1024);
    let mut rest = index.as_str();
    while let Some(start) = rest.find('<') {
        html.push_str(&rest[..start]);
        let tag_end = tag_end(rest, start).context("malformed index.html")?;
        html.push_str(&rewrite_tag(&rest[start..tag_end], lesson, fonts, title)?);
        rest = &rest[tag_end..];
    }
    html.push_str(rest);
    Ok(html)
}

/// End of the tag at `start`, including the body of `<script>` and `<title>`.
fn tag_end(text: &str, start: usize) -> Option<usize> {
    let close = start + text[start..].find('>')? + 1;
    for element in ["script", "title"] {
        if text[start..].starts_with(&format!("<{element}")) {
            let closing = format!("</{element}>");
            return Some(close + text[close..].find(&closing)? + closing.len());
        }
    }
    Some(close)
}

fn rewrite_tag(tag: &str, lesson: &Value, fonts: &[String], title: &str) -> Result<String> {
    if tag.starts_with("<title") {
        return Ok(format!("<title>{} · Trunade!</title>", escape_html(title)));
    }
    if tag.starts_with("<link") {
        return match (attribute(tag, "rel"), attribute(tag, "href")) {
            (Some("stylesheet"), Some(href)) => Ok(format!("<style>{}</style>", inline_css(href, fonts)?)),
            (Some("icon"), Some(href)) => Ok(format!("<link rel=\"icon\" href=\"{}\">", data_uri(href)?)),
            _ => Ok(String::new()), // preloads, other icons
        };
    }
    if tag.starts_with("<script") {
        if attribute(tag, "type") == Some("importmap") {
            return Ok(String::new());
        }
        if let (Some("module"), Some(src)) = (attribute(tag, "type"), attribute(tag, "src")) {
            let code = asset_text(src.trim_start_matches('/')).context("missing script")?;
            return Ok(format!(
                "<script>window.__TRUNADE_STATIC__={}</script><script type=\"module\">{}</script>",
                json_in_script(&lesson.to_string()),
                js_in_script(&code),
            ));
        }
    }
    Ok(tag.to_string())
}

fn attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let marker = format!(" {name}=\"");
    let start = tag.find(&marker)? + marker.len();
    Some(&tag[start..start + tag[start..].find('"')?])
}

/// No `<` at all in the data, so nothing in it can end the `<script>` element.
fn json_in_script(json: &str) -> String {
    json.replace('<', "\\u003c")
}

/// Only `</script` ends the element; `</` alone is valid code (`/</g`).
fn js_in_script(code: &str) -> String {
    let lower = code.to_ascii_lowercase();
    let mut out = String::with_capacity(code.len());
    let mut last = 0;
    for (index, _) in lower.match_indices("</script") {
        out.push_str(&code[last..index]);
        out.push_str("<\\/");
        last = index + 2;
    }
    out.push_str(&code[last..]);
    out
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

// ------------------------------------------------------------------ CSS

fn inline_css(href: &str, fonts: &[String]) -> Result<String> {
    let path = href.trim_start_matches('/');
    let css = asset_text(path).context("missing stylesheet")?;
    let base = path.rsplit_once('/').map_or("", |(dir, _)| dir);
    let mut out = String::with_capacity(css.len());
    let mut rest = css.as_str();
    while let Some(start) = rest.find("@font-face{") {
        out.push_str(&rest[..start]);
        let end = start + rest[start..].find('}').context("unclosed @font-face")? + 1;
        if let Some(face) = kept_font_face(&rest[start..end], fonts) {
            out.push_str(&face);
        }
        rest = &rest[end..];
    }
    out.push_str(rest);
    inline_urls(&out, base)
}

/// A face of a font in use, latin subset only, with just its woff2 source.
fn kept_font_face(face: &str, fonts: &[String]) -> Option<String> {
    let family = declaration(face, "font-family")?.trim_matches(|c| c == '"' || c == '\'');
    let wanted = fonts.iter().any(|font| font == family)
        || (family.starts_with("KaTeX_") && fonts.iter().any(|font| font == KATEX_FONTS));
    let latin = declaration(face, "unicode-range")
        .is_none_or(|range| range.split(',').any(|part| LATIN_RANGES.contains(&part.trim())));
    if !wanted || !latin {
        return None;
    }
    let source = declaration(face, "src")?;
    let woff2 = source.split(',').find(|part| part.contains(".woff2"))?;
    Some(face.replace(source, woff2))
}

/// Value of `name:` inside a `{…}` block (no nested blocks in @font-face).
fn declaration<'a>(block: &'a str, name: &str) -> Option<&'a str> {
    let body = block.split_once('{')?.1.trim_end_matches('}');
    body.split(';').find_map(|item| {
        let (key, value) = item.split_once(':')?;
        (key.trim() == name).then(|| value.trim())
    })
}

/// `url(./file)` and `url(/_nuxt/file)` → data URIs of the embedded assets.
fn inline_urls(css: &str, base: &str) -> Result<String> {
    let mut out = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(start) = rest.find("url(") {
        out.push_str(&rest[..start + 4]);
        let end = start + 4 + rest[start + 4..].find(')').context("unclosed url(")?;
        let reference = rest[start + 4..end].trim_matches(|c| c == '"' || c == '\'');
        if reference.starts_with("data:") {
            out.push_str(reference);
        } else {
            let path = match reference.strip_prefix("./") {
                Some(relative) => format!("{base}/{relative}"),
                None => reference.trim_start_matches('/').to_string(),
            };
            out.push_str(&data_uri(&path)?);
        }
        rest = &rest[end..];
    }
    out.push_str(rest);
    Ok(out)
}

// ---------------------------------------------------------------- assets

fn asset_text(path: &str) -> Option<String> {
    ExportAssets::get(path).map(|file| String::from_utf8_lossy(&file.data).into_owned())
}

fn data_uri(path: &str) -> Result<String> {
    let path = path.trim_start_matches('/');
    let file = ExportAssets::get(path)
        .or_else(|| ExportAssets::get(&format!("{ASSETS_DIR}{path}")))
        .with_context(|| format!("missing asset {path}"))?;
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    Ok(format!("data:{mime};base64,{}", base64(&file.data)))
}

fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = chunk.iter().enumerate().fold(0u32, |n, (i, &b)| n | u32::from(b) << (16 - 8 * i));
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(ALPHABET[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_pads() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn keeps_only_latin_faces_of_fonts_in_use() {
        let fonts = vec!["Gloock".to_string(), KATEX_FONTS.to_string()];
        let latin = "@font-face{font-family:Gloock;src:url(./g.woff2)format(\"woff2\"),url(./g.woff)format(\"woff\");unicode-range:U+0-FF,U+131}";
        assert_eq!(
            kept_font_face(latin, &fonts).unwrap(),
            "@font-face{font-family:Gloock;src:url(./g.woff2)format(\"woff2\");unicode-range:U+0-FF,U+131}"
        );
        let minified = "@font-face{font-family:Gloock;src:url(./g.woff2)format(\"woff2\");unicode-range:U+??,U+131}";
        assert!(kept_font_face(minified, &fonts).is_some());
        let cyrillic = "@font-face{font-family:Gloock;src:url(./c.woff2)format(\"woff2\");unicode-range:U+400-45F}";
        assert!(kept_font_face(cyrillic, &fonts).is_none());
        let unused = "@font-face{font-family:\"Inter Variable\";src:url(./i.woff2);unicode-range:U+0-FF}";
        assert!(kept_font_face(unused, &fonts).is_none());
        let katex = "@font-face{font-family:KaTeX_Main;src:url(./k.woff2)format(\"woff2\"),url(./k.ttf)}";
        assert!(kept_font_face(katex, &fonts).unwrap().contains("k.woff2"));
    }

    #[test]
    fn scripts_cannot_close_early() {
        assert_eq!(js_in_script("a='</SCRIPT>';b=/</g"), "a='<\\/SCRIPT>';b=/</g");
        assert_eq!(json_in_script(r#"{"html":"<p>"}"#), r#"{"html":"\u003cp>"}"#);
    }

    #[test]
    fn exports_beside_the_document_without_overwriting() {
        let dir = tempfile::tempdir().unwrap();
        let document = dir.path().join("aula.md");
        std::fs::write(dir.path().join("aula.html"), "old").unwrap();
        let (path, _) = create_beside(&document).unwrap();
        assert_eq!(path.file_name().unwrap(), "aula-1.html");
        assert_eq!(std::fs::read_to_string(dir.path().join("aula.html")).unwrap(), "old");
    }
}
