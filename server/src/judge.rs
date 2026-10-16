//! Test cases of a block (```stdin / ```expected pairs) and the comparison a
//! contest judge makes: trailing spaces and trailing blank lines do not count.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct TestCase {
    pub stdin: String,
    pub expected: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Verdict {
    Accepted,
    /// First differing line (1-based); `None` means that side had ended.
    WrongAnswer {
        line: usize,
        expected: Option<String>,
        actual: Option<String>,
    },
}

pub fn judge(expected: &str, actual: &str) -> Verdict {
    let expected = significant_lines(expected);
    let actual = significant_lines(actual);
    let differing = (0..expected.len().max(actual.len()))
        .find(|&index| expected.get(index) != actual.get(index));
    match differing {
        None => Verdict::Accepted,
        Some(index) => Verdict::WrongAnswer {
            line: index + 1,
            expected: expected.get(index).map(|line| line.to_string()),
            actual: actual.get(index).map(|line| line.to_string()),
        },
    }
}

fn significant_lines(text: &str) -> Vec<&str> {
    let mut lines: Vec<&str> = text.lines().map(str::trim_end).collect();
    while lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trailing_spaces_and_blank_lines_are_ignored() {
        assert_eq!(judge("6\n", "6   \r\n\n\n"), Verdict::Accepted);
        assert_eq!(judge("1 2\n3", "1 2\n3\n"), Verdict::Accepted);
    }

    #[test]
    fn first_difference_is_reported() {
        assert_eq!(
            judge("1\n2\n3\n", "1\n5\n3\n"),
            Verdict::WrongAnswer { line: 2, expected: Some("2".into()), actual: Some("5".into()) }
        );
    }

    #[test]
    fn missing_output_has_no_actual_line() {
        assert_eq!(
            judge("0\n", ""),
            Verdict::WrongAnswer { line: 1, expected: Some("0".into()), actual: None }
        );
        assert_eq!(
            judge("1\n", "1\n2\n"),
            Verdict::WrongAnswer { line: 2, expected: None, actual: Some("2".into()) }
        );
    }

    #[test]
    fn leading_spaces_count() {
        assert!(matches!(judge("6", " 6"), Verdict::WrongAnswer { line: 1, .. }));
    }
}
