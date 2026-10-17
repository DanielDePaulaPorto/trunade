use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

use comrak::nodes::{NodeCodeBlock, NodeValue};
use comrak::{Arena, Options, markdown_to_html, parse_document};
use serde::Serialize;

use crate::judge::TestCase;

pub const STDIN_FENCE: &str = "stdin";
pub const EXPECTED_FENCE: &str = "expected";
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
        /// ```stdin / ```expected blocks right after the code. None: interactive run.
        cases: Vec<TestCase>,
        /// Name of the template used (```cpp-name), if any.
        template: Option<String>,
        /// Hidden template code around `source` (see `Template`).
        before: String,
        after: String,
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
    /// ```cpp template=name declares a template; a bare `template` gives Some("").
    pub template: Option<String>,
}

/// Code the compiler gets but the page hides: what a ```lang template=name
/// block has before and after its `{code}` line (a whole template without one
/// is `before`). Only blocks that ask for it by name (```lang-name) use it.
#[derive(Debug, Clone, Default, PartialEq)]
struct Template {
    before: String,
    after: String,
}

impl Template {
    fn parse(text: &str) -> Self {
        let mut before = String::new();
        let mut after = String::new();
        let mut marker_seen = false;
        for line in text.lines() {
            if !marker_seen && is_code_marker(line) {
                marker_seen = true;
                continue;
            }
            let part = if marker_seen { &mut after } else { &mut before };
            part.push_str(line);
            part.push('\n');
        }
        Self { before, after }
    }
}

/// `{code}`, optionally behind a line comment (`//{code}`, `# {code}`).
fn is_code_marker(line: &str) -> bool {
    line.trim().trim_start_matches(|c: char| !c.is_alphanumeric() && c != '{') == "{code}"
}

/// The program the compiler gets: the visible source inside its template.
pub fn assemble(before: &str, source: &str, after: &str) -> String {
    format!("{before}{source}\n{after}")
}

/// Splits `markdown` into Markdown and code cells. A fenced block becomes a code
/// cell when `language_key` knows its language (returning the configured name,
/// shared by aliases); ```stdin and ```expected blocks right after it become its test cases.
pub fn parse(markdown: &str, language_key: impl Fn(&str) -> Option<String>) -> Vec<Cell> {
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
        let attaches_case = matches!(info.language.as_str(), STDIN_FENCE | EXPECTED_FENCE)
            && builder.last_is_code()
            && is_blank(&lines, next_markdown_line, start);
        let is_quiz = info.language == QUIZ_FENCE;
        let language = resolve_language(&info.language, &language_key);
        if !attaches_case && !is_quiz && language.is_none() {
            continue;
        }
        builder.push_markdown(&lines, next_markdown_line, start);
        match language {
            _ if attaches_case && info.language == STDIN_FENCE => builder.add_stdin(&block.literal),
            _ if attaches_case => builder.add_expected(&block.literal),
            _ if is_quiz => builder.push_quiz(&block.literal),
            Some(language) if info.template.is_some() => builder.declare_template(&language, &info, &block.literal),
            Some(language) => builder.push_code(info, language, start, block),
            None => {}
        }
        next_markdown_line = end + 1;
    }
    builder.push_markdown(&lines, next_markdown_line, lines.len() + 1);
    builder.finish()
}

/// A fence's language once resolved: `cpp-maratona` is C++ with the template `maratona`.
struct FenceLanguage {
    /// Configured language name, shared by aliases.
    key: String,
    /// The fence's own spelling, without the template suffix.
    written: String,
    template: Option<String>,
}

fn resolve_language(fence: &str, language_key: &impl Fn(&str) -> Option<String>) -> Option<FenceLanguage> {
    if let Some(key) = language_key(fence) {
        return Some(FenceLanguage { key, written: fence.to_string(), template: None });
    }
    let (written, name) = fence.split_once('-')?;
    let key = language_key(written)?;
    (!name.is_empty()).then(|| FenceLanguage {
        key,
        written: written.to_string(),
        template: Some(name.to_string()),
    })
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
    for word in split_words(attributes) {
        match word.split_once('=') {
            Some(("template", name)) => fence.template = Some(name.to_string()),
            None if word == "template" => fence.template = Some(String::new()),
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
    /// Templates declared so far, by (configured language, name).
    templates: HashMap<(String, String), Template>,
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

    fn declare_template(&mut self, language: &FenceLanguage, info: &FenceInfo, text: &str) {
        match info.template.as_deref() {
            Some(name) if !name.is_empty() => {
                self.templates.insert((language.key.clone(), name.to_string()), Template::parse(text));
            }
            _ => self.push_warning(&format!(
                "This template has no name. Declare it as <code>```{0} template=name</code> and use it with <code>```{0}-name</code>.",
                language.written
            )),
        }
    }

    fn push_code(&mut self, info: FenceInfo, language: FenceLanguage, line: usize, block: &NodeCodeBlock) {
        let template = match &language.template {
            None => Template::default(),
            Some(name) => match self.templates.get(&(language.key.clone(), name.clone())) {
                Some(template) => template.clone(),
                None => {
                    self.push_warning(&format!(
                        "The template <code>{name}</code> is not declared above this block (<code>```{} template={name}</code>).",
                        language.written
                    ));
                    Template::default()
                }
            },
        };
        self.target().push(Cell::Code {
            id: String::new(), // assigned in finish(), once the cases are known
            fence_language: language.written,
            template: language.template,
            line,
            // The closing fence's newline is not part of the program; editors show it as an empty line.
            source: block.literal.strip_suffix('\n').unwrap_or(&block.literal).to_string(),
            flags: info.flags,
            timeout_seconds: info.timeout_seconds,
            cases: vec![],
            before: template.before,
            after: template.after,
        });
    }

    /// A mistake in the lesson, shown where it happens instead of failing silently.
    fn push_warning(&mut self, message: &str) {
        let html = format!(
            "<div class=\"markdown-alert markdown-alert-caution\">\n<p class=\"markdown-alert-title\">Trunade!</p>\n<p>{message}</p>\n</div>\n"
        );
        let id = self.unique_id(&("warning", &html));
        self.target().push(Cell::Markdown { id, html });
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

    fn last_is_code(&mut self) -> bool {
        matches!(self.target().last(), Some(Cell::Code { .. }))
    }

    fn last_cases(&mut self) -> Option<&mut Vec<TestCase>> {
        match self.target().last_mut() {
            Some(Cell::Code { cases, .. }) => Some(cases),
            _ => None,
        }
    }

    /// Every ```stdin starts a new case.
    fn add_stdin(&mut self, text: &str) {
        if let Some(cases) = self.last_cases() {
            cases.push(TestCase { stdin: text.to_string(), expected: None });
        }
    }

    /// Completes the last case, or starts one with empty input.
    fn add_expected(&mut self, text: &str) {
        let Some(cases) = self.last_cases() else { return };
        match cases.last_mut() {
            Some(case) if case.expected.is_none() => case.expected = Some(text.to_string()),
            _ => cases.push(TestCase { stdin: String::new(), expected: Some(text.to_string()) }),
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
                Cell::Code { id, fence_language, source, flags, cases, before, after, .. } => {
                    // the template's content is in before/after, its name changes nothing
                    let content = (&fence_language, &source, &flags, &cases, &before, &after);
                    *id = self.unique_id(&("code", content));
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

    fn runnable(language: &str) -> Option<String> {
        match language {
            "cpp" | "c++" => Some("cpp".into()),
            "python" => Some("python".into()),
            _ => None,
        }
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
        let Cell::Code { fence_language, line, source, flags, timeout_seconds, cases, .. } = &cells[1]
        else {
            panic!("expected code cell");
        };
        assert_eq!(fence_language, "cpp");
        assert_eq!(*line, 5);
        assert_eq!(source, "int main() {}");
        assert_eq!(flags, &["-O2", "-g"]);
        assert_eq!(*timeout_seconds, Some(5.0));
        assert_eq!(cases, &[TestCase { stdin: "2 40\n".into(), expected: None }]);
    }

    #[test]
    fn stdin_and_expected_pairs_become_cases() {
        let lesson = "```cpp\nx\n```\n\n```stdin\n12 18\n```\n```expected\n6\n```\n\n```stdin\n0 0\n```\n\n```expected\n0\n```\n\n```stdin\n7\n```\n";
        let Cell::Code { cases, .. } = &parse(lesson, runnable)[0] else { panic!("expected code") };
        let pairs: Vec<(&str, Option<&str>)> =
            cases.iter().map(|case| (case.stdin.as_str(), case.expected.as_deref())).collect();
        assert_eq!(pairs, [("12 18\n", Some("6\n")), ("0 0\n", Some("0\n")), ("7\n", None)]);
    }

    #[test]
    fn expected_without_stdin_runs_with_empty_input() {
        let lesson = "```python\nprint(1)\n```\n\n```expected\n1\n```\n";
        let Cell::Code { cases, .. } = &parse(lesson, runnable)[0] else { panic!("expected code") };
        assert_eq!(cases, &[TestCase { stdin: String::new(), expected: Some("1\n".into()) }]);
    }

    #[test]
    fn stdin_separated_by_text_stays_markdown() {
        let cells = parse("```cpp\nx\n```\n\npara\n\n```stdin\n1\n```\n", runnable);
        let Cell::Code { cases, .. } = code_cells(&cells)[0] else { unreachable!() };
        assert!(cases.is_empty());
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
        let Cell::Code { id, cases, .. } = &steps[1].cells[0] else { panic!("expected code") };
        assert_eq!(cases[0].stdin, "1 2\n");
        assert!(find_cell(&cells, id).is_some());
    }

    #[test]
    fn unclosed_steps_end_with_the_file() {
        let cells = parse("<!-- step: A -->\ntexto\n", runnable);
        assert!(matches!(&cells[0], Cell::Steps { steps, .. } if steps[0].cells.len() == 1));
    }

    #[test]
    fn named_template_wraps_only_blocks_that_ask_for_it() {
        let lesson = "```cpp template=cp\n#include <x>\n//{code}\nint tail;\n```\n\n```c++-cp\nint main() {}\n```\n\n```cpp\nint whole;\n```\n";
        let cells = parse(lesson, runnable);
        assert_eq!(cells.len(), 2, "the template itself is not a cell");
        let Cell::Code { fence_language, template, before, after, source, .. } = &cells[0] else { panic!("expected code") };
        assert_eq!(fence_language, "c++");
        assert_eq!(template.as_deref(), Some("cp"));
        assert_eq!(assemble(before, source, after), "#include <x>\nint main() {}\nint tail;\n");
        let Cell::Code { template, before, .. } = &cells[1] else { panic!("expected code") };
        assert!(template.is_none() && before.is_empty(), "a block without a name is whole");
    }

    #[test]
    fn several_templates_and_marker_less_template() {
        let lesson = "```python template=io\nimport sys\n```\n\n```cpp template=io\n#include <cstdio>\n```\n\n```python-io\nx\n```\n\n```cpp-io\ny\n```\n";
        let cells = parse(lesson, runnable);
        let Cell::Code { before, after, .. } = &cells[0] else { panic!("expected code") };
        assert_eq!((before.as_str(), after.as_str()), ("import sys\n", ""));
        let Cell::Code { before, .. } = &cells[1] else { panic!("expected code") };
        assert_eq!(before, "#include <cstdio>\n", "same name, other language");
    }

    #[test]
    fn mistakes_with_templates_are_shown() {
        let cells = parse("```cpp template\nx\n```\n\n```cpp-missing\nint main() {}\n```\n", runnable);
        assert!(matches!(&cells[0], Cell::Markdown { html, .. } if html.contains("no name")));
        assert!(matches!(&cells[1], Cell::Markdown { html, .. } if html.contains("<code>missing</code>")));
        assert!(matches!(&cells[2], Cell::Code { before, .. } if before.is_empty()));
    }

    #[test]
    fn unknown_language_with_a_dash_stays_text() {
        let cells = parse("```objective-c\nx\n```\n", runnable);
        assert!(matches!(&cells[0], Cell::Markdown { .. }));
    }

    #[test]
    fn code_marker_accepts_comments() {
        assert!(is_code_marker("//{code}"));
        assert!(is_code_marker("  # {code}  "));
        assert!(!is_code_marker("int code;"));
    }

    #[test]
    fn fence_info_without_attributes() {
        assert_eq!(parse_fence_info("C++"), FenceInfo { language: "c++".into(), ..Default::default() });
        assert_eq!(parse_fence_info("cpp{timeout=1}").timeout_seconds, Some(1.0));
        assert_eq!(parse_fence_info("cpp template=cp").template.as_deref(), Some("cp"));
        assert_eq!(parse_fence_info("cpp {template=cp timeout=1}").template.as_deref(), Some("cp"));
        assert_eq!(parse_fence_info("cpp template").template.as_deref(), Some(""));
    }
}
