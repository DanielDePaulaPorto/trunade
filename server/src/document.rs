use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

use comrak::nodes::{NodeCodeBlock, NodeValue};
use comrak::{Arena, Options, markdown_to_html, parse_document};
use serde::Serialize;

pub const STDIN_FENCE: &str = "stdin";
pub const QUIZ_FENCE: &str = "quiz";

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuizOption {
    pub html: String,
    pub correct: bool,
}

/// One step of an exercise: `<!-- step: Title -->` up to the next marker.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Step {
    pub title: String,
    pub cells: Vec<Cell>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Cell {
    Markdown {
        id: String,
        html: String,
    },
    #[serde(rename_all = "camelCase")]
    Code {
        id: String,
        /// Language as written in the fence (`c++`, `py`...).
        fence_language: String,
        /// 1-based line of the opening fence in the Markdown file.
        line: usize,
        source: String,
        flags: Vec<String>,
        timeout_seconds: Option<f64>,
        stdin: Option<String>,
    },
    /// ```quiz block: question in Markdown, then `- [ ]` / `- [x]` options.
    Quiz {
        id: String,
        question: String,
        options: Vec<QuizOption>,
    },
    /// Cells grouped by `<!-- step: … -->` markers, closed by `<!-- /steps -->`.
    Steps {
        id: String,
        steps: Vec<Step>,
    },
}

impl Cell {
    pub fn id(&self) -> &str {
        match self {
            Cell::Markdown { id, .. }
            | Cell::Code { id, .. }
            | Cell::Quiz { id, .. }
            | Cell::Steps { id, .. } => id,
        }
    }
}

/// Finds a cell by id, including cells inside steps.
pub fn find_cell<'a>(cells: &'a [Cell], id: &str) -> Option<&'a Cell> {
    cells.iter().find_map(|cell| match cell {
        _ if cell.id() == id => Some(cell),
        Cell::Steps { steps, .. } => steps.iter().find_map(|step| find_cell(&step.cells, id)),
        _ => None,
    })
}

enum StepMarker {
    Start(String),
    End,
}

fn parse_step_marker(html: &str) -> Option<StepMarker> {
    let inner = html.trim().strip_prefix("<!--")?.strip_suffix("-->")?.trim();
    if inner == "/steps" {
        return Some(StepMarker::End);
    }
    let title = inner.strip_prefix("step:")?.trim();
    Some(StepMarker::Start(title.to_string()))
}

/// Fence info string split into language and `{key=value ...}` attributes.
#[derive(Debug, Default, PartialEq)]
pub struct FenceInfo {
    pub language: String,
    pub flags: Vec<String>,
    pub timeout_seconds: Option<f64>,
}

/// Splits `markdown` into Markdown and code cells. A fenced block becomes a code
/// cell when `is_runnable(language)`; a ```stdin block right after it becomes its stdin.
pub fn parse(markdown: &str, is_runnable: impl Fn(&str) -> bool) -> Vec<Cell> {
    let arena = Arena::new();
    let root = parse_document(&arena, markdown, &markdown_options());
    let lines: Vec<&str> = markdown.lines().collect();
    let mut builder = CellBuilder::default();
    let mut next_markdown_line = 1;

    for node in root.children() {
        let ast = node.data();
        let (start, end) = (ast.sourcepos.start.line, ast.sourcepos.end.line);
        let block = match &ast.value {
            NodeValue::HtmlBlock(html) => {
                if let Some(marker) = parse_step_marker(&html.literal) {
                    builder.push_markdown(&lines, next_markdown_line, start);
                    match marker {
                        StepMarker::Start(title) => builder.start_step(title),
                        StepMarker::End => builder.end_steps(),
                    }
                    next_markdown_line = end + 1;
                }
                continue;
            }
            NodeValue::CodeBlock(block) if block.fenced => block,
            _ => continue,
        };
        let info = parse_fence_info(&block.info);
        let attaches_stdin = info.language == STDIN_FENCE
            && builder.last_code_accepts_stdin()
            && is_blank(&lines, next_markdown_line, start);
        let is_quiz = info.language == QUIZ_FENCE;
        if !attaches_stdin && !is_quiz && !is_runnable(&info.language) {
            continue;
        }
        builder.push_markdown(&lines, next_markdown_line, start);
        if attaches_stdin {
            builder.attach_stdin(&block.literal);
        } else if is_quiz {
            builder.push_quiz(&block.literal);
        } else {
            builder.push_code(info, start, block);
        }
        next_markdown_line = end + 1;
    }
    builder.push_markdown(&lines, next_markdown_line, lines.len() + 1);
    builder.finish()
}

pub fn parse_fence_info(info: &str) -> FenceInfo {
    let info = info.trim();
    let (language, attributes) = match info.find(|c: char| c.is_whitespace() || c == '{') {
        Some(index) => (&info[..index], info[index..].trim()),
        None => (info, ""),
    };
    let mut fence = FenceInfo {
        language: language.to_lowercase(),
        ..FenceInfo::default()
    };
    let attributes = attributes.trim_start_matches('{').trim_end_matches('}');
    for (key, value) in split_attributes(attributes) {
        match key.as_str() {
            "flags" => fence.flags = split_words(&value),
            "timeout" => fence.timeout_seconds = value.parse().ok(),
            _ => {}
        }
    }
    fence
}

/// `key=value key="quoted value"` → pairs. Tokens without `=` are ignored.
fn split_attributes(text: &str) -> Vec<(String, String)> {
    split_words(text)
        .into_iter()
        .filter_map(|token| {
            let (key, value) = token.split_once('=')?;
            Some((key.to_string(), value.to_string()))
        })
        .collect()
}

/// Whitespace split that keeps double-quoted runs together and drops the quotes.
fn split_words(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    for c in text.chars() {
        match c {
            '"' => quoted = !quoted,
            c if c.is_whitespace() && !quoted => {
                if !current.is_empty() {
                    words.push(std::mem::take(&mut current));
                }
            }
            c => current.push(c),
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

fn markdown_options() -> Options<'static> {
    let mut options = Options::default();
    options.extension.table = true;
    options.extension.strikethrough = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.footnotes = true;
    options.extension.alerts = true;
    options.extension.math_dollars = true; // $x$, $$x$$ → rendered by KaTeX in the browser
    options.extension.math_code = true; // ```math blocks
    // Single user rendering their own files: raw HTML (div, iframe, img) is wanted.
    options.render.r#unsafe = true;
    options
}

/// `- [ ] text` or `- [x] text` → option with inline-rendered text.
fn parse_quiz_option(line: &str) -> Option<QuizOption> {
    let rest = line.trim_start().strip_prefix(['-', '*'])?.trim_start();
    let (mark, text) = rest.strip_prefix('[')?.split_once(']')?;
    let correct = match mark {
        " " => false,
        "x" | "X" => true,
        _ => return None,
    };
    Some(QuizOption { html: render_inline(text.trim()), correct })
}

/// Renders one line of Markdown without the surrounding paragraph.
fn render_inline(text: &str) -> String {
    let html = markdown_to_html(text, &markdown_options());
    let html = html.trim();
    html.strip_prefix("<p>")
        .and_then(|inner| inner.strip_suffix("</p>"))
        .unwrap_or(html)
        .to_string()
}

/// Lines `[from, to)` are all blank (1-based).
fn is_blank(lines: &[&str], from: usize, to: usize) -> bool {
    line_range(lines, from, to).iter().all(|line| line.trim().is_empty())
}

fn line_range<'a>(lines: &'a [&'a str], from: usize, to: usize) -> &'a [&'a str] {
    let start = from.saturating_sub(1).min(lines.len());
    let end = to.saturating_sub(1).min(lines.len()).max(start);
    &lines[start..end]
}

#[derive(Default)]
struct CellBuilder {
    cells: Vec<Cell>,
    /// Open steps: new cells go into the last one until `<!-- /steps -->`.
    steps: Option<Vec<Step>>,
    seen_ids: HashMap<u64, usize>,
}

impl CellBuilder {
    fn target(&mut self) -> &mut Vec<Cell> {
        match self.steps.as_mut().and_then(|steps| steps.last_mut()) {
            Some(step) => &mut step.cells,
            None => &mut self.cells,
        }
    }

    fn start_step(&mut self, title: String) {
        let step = Step { title, cells: vec![] };
        self.steps.get_or_insert_with(Vec::new).push(step);
    }

    fn end_steps(&mut self) {
        if let Some(steps) = self.steps.take() {
            // id assigned in finish(), once the children have theirs
            self.cells.push(Cell::Steps { id: String::new(), steps });
        }
    }

    fn push_markdown(&mut self, lines: &[&str], from: usize, to: usize) {
        let text = line_range(lines, from, to).join("\n");
        if text.trim().is_empty() {
            return;
        }
        let id = self.unique_id(&("markdown", &text));
        let html = markdown_to_html(&text, &markdown_options());
        self.target().push(Cell::Markdown { id, html });
    }

    fn push_code(&mut self, info: FenceInfo, line: usize, block: &NodeCodeBlock) {
        self.target().push(Cell::Code {
            id: String::new(), // assigned in finish(), once stdin is known
            fence_language: info.language,
            line,
            // The closing fence's newline is not part of the program; editors show it as an empty line.
            source: block.literal.strip_suffix('\n').unwrap_or(&block.literal).to_string(),
            flags: info.flags,
            timeout_seconds: info.timeout_seconds,
            stdin: None,
        });
    }

    fn push_quiz(&mut self, text: &str) {
        let mut question = Vec::new();
        let mut options = Vec::new();
        for line in text.lines() {
            match parse_quiz_option(line) {
                Some(option) => options.push(option),
                None => question.push(line),
            }
        }
        let question = markdown_to_html(&question.join("\n"), &markdown_options());
        let id = self.unique_id(&("quiz", text));
        self.target().push(Cell::Quiz { id, question, options });
    }

    fn last_code_accepts_stdin(&mut self) -> bool {
        matches!(self.target().last(), Some(Cell::Code { stdin: None, .. }))
    }

    fn attach_stdin(&mut self, text: &str) {
        if let Some(Cell::Code { stdin, .. }) = self.target().last_mut() {
            *stdin = Some(text.to_string());
        }
    }

    fn finish(mut self) -> Vec<Cell> {
        self.end_steps(); // a missing <!-- /steps --> closes at the end of the file
        let mut cells = std::mem::take(&mut self.cells);
        self.assign_ids(&mut cells);
        cells
    }

    /// Code and steps ids depend on content known only after parsing.
    fn assign_ids(&mut self, cells: &mut [Cell]) {
        for cell in cells {
            match cell {
                Cell::Code { id, fence_language, source, flags, stdin, .. } => {
                    *id = self.unique_id(&("code", &fence_language, &source, &flags, &stdin));
                }
                Cell::Steps { id, steps } => {
                    for step in steps.iter_mut() {
                        self.assign_ids(&mut step.cells);
                    }
                    let outline: Vec<(&str, Vec<&str>)> = steps
                        .iter()
                        .map(|step| (step.title.as_str(), step.cells.iter().map(Cell::id).collect()))
                        .collect();
                    *id = self.unique_id(&("steps", outline));
                }
                Cell::Markdown { .. } | Cell::Quiz { .. } => {}
            }
        }
    }

    /// Content hash, so a cell keeps its id while other cells change.
    fn unique_id(&mut self, content: &impl Hash) -> String {
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        let hash = hasher.finish();
        let occurrence = self.seen_ids.entry(hash).or_insert(0);
        *occurrence += 1;
        format!("{hash:016x}-{occurrence}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn runnable(language: &str) -> bool {
        matches!(language, "cpp" | "python")
    }

    fn code_cells(cells: &[Cell]) -> Vec<&Cell> {
        cells.iter().filter(|c| matches!(c, Cell::Code { .. })).collect()
    }

    const LESSON: &str = "# Aula\n\nTexto.\n\n```cpp {flags=\"-O2 -g\" timeout=5}\nint main() {}\n```\n\n```stdin\n2 40\n```\n\n```text\nnot runnable\n```\n\n<div class=\"note\">raw</div>\n";

    #[test]
    fn splits_markdown_and_code_cells() {
        let cells = parse(LESSON, runnable);
        assert_eq!(cells.len(), 3);
        assert!(matches!(&cells[0], Cell::Markdown { html, .. } if html.contains("<h1>Aula</h1>")));
        assert!(matches!(&cells[2], Cell::Markdown { html, .. }
            if html.contains("not runnable") && html.contains("<div class=\"note\">raw</div>")));
    }

    #[test]
    fn code_cell_has_line_attributes_and_stdin() {
        let cells = parse(LESSON, runnable);
        let Cell::Code { fence_language, line, source, flags, timeout_seconds, stdin, .. } = &cells[1]
        else {
            panic!("expected code cell");
        };
        assert_eq!(fence_language, "cpp");
        assert_eq!(*line, 5);
        assert_eq!(source, "int main() {}");
        assert_eq!(flags, &["-O2", "-g"]);
        assert_eq!(*timeout_seconds, Some(5.0));
        assert_eq!(stdin.as_deref(), Some("2 40\n"));
    }

    #[test]
    fn stdin_separated_by_text_stays_markdown() {
        let cells = parse("```cpp\nx\n```\n\npara\n\n```stdin\n1\n```\n", runnable);
        let Cell::Code { stdin, .. } = code_cells(&cells)[0] else { unreachable!() };
        assert!(stdin.is_none());
        assert!(matches!(cells.last(), Some(Cell::Markdown { html, .. }) if html.contains("<code")));
    }

    #[test]
    fn ids_are_stable_when_other_cells_change() {
        let before = parse("intro\n\n```cpp\nint a;\n```\n", runnable);
        let after = parse("intro edited\n\n```cpp\nint a;\n```\n", runnable);
        assert_ne!(before[0].id(), after[0].id());
        assert_eq!(before[1].id(), after[1].id());
    }

    #[test]
    fn identical_cells_get_distinct_ids() {
        let cells = parse("```cpp\nx\n```\n\n```cpp\nx\n```\n", runnable);
        assert_ne!(cells[0].id(), cells[1].id());
    }

    #[test]
    fn quiz_block_becomes_quiz_cell() {
        let cells = parse("```quiz\nQual é **O(log n)**?\n- [ ] busca linear\n- [x] busca `binária`\n```\n", runnable);
        let Cell::Quiz { question, options, .. } = &cells[0] else { panic!("expected quiz") };
        assert!(question.contains("<strong>O(log n)</strong>"));
        assert_eq!(options.len(), 2);
        assert!(!options[0].correct);
        assert!(options[1].correct);
        assert_eq!(options[1].html, "busca <code>binária</code>");
    }

    #[test]
    fn math_is_marked_for_the_browser() {
        let cells = parse("Área $\\pi r^2$.\n", runnable);
        assert!(matches!(&cells[0], Cell::Markdown { html, .. } if html.contains("data-math-style=\"inline\"")));
    }

    #[test]
    fn step_markers_group_cells_and_keep_code_runnable() {
        let lesson = "intro\n\n<!-- step: Enunciado -->\nSome dois números.\n\n<!-- step: Solução -->\n```cpp\nint main() {}\n```\n\n```stdin\n1 2\n```\n<!-- /steps -->\n\nfim\n";
        let cells = parse(lesson, runnable);
        assert_eq!(cells.len(), 3);
        let Cell::Steps { steps, .. } = &cells[1] else { panic!("expected steps") };
        assert_eq!(steps.iter().map(|s| s.title.as_str()).collect::<Vec<_>>(), ["Enunciado", "Solução"]);
        let Cell::Code { id, stdin, .. } = &steps[1].cells[0] else { panic!("expected code") };
        assert_eq!(stdin.as_deref(), Some("1 2\n"));
        assert!(find_cell(&cells, id).is_some());
    }

    #[test]
    fn unclosed_steps_end_with_the_file() {
        let cells = parse("<!-- step: A -->\ntexto\n", runnable);
        assert!(matches!(&cells[0], Cell::Steps { steps, .. } if steps[0].cells.len() == 1));
    }

    #[test]
    fn fence_info_without_attributes() {
        assert_eq!(parse_fence_info("C++"), FenceInfo { language: "c++".into(), ..Default::default() });
        assert_eq!(parse_fence_info("cpp{timeout=1}").timeout_seconds, Some(1.0));
    }
}
