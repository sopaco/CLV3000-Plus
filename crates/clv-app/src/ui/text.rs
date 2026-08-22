//! Text truncation helpers for path and label display.

use crate::prelude::*;
use crate::theme::colors;

/// Minimum character length before using middle-ellipsis layout for paths.
const PATH_MIDDLE_ELLIPSIS_THRESHOLD: usize = 32;

/// Truncate `text` to `max_chars` with an ellipsis in the middle.
pub fn truncate_middle(text: &str, max_chars: usize) -> String {
    let count = text.chars().count();
    if count <= max_chars {
        return text.to_string();
    }
    if max_chars <= 1 {
        return "…".to_string();
    }
    let keep = max_chars - 1;
    let keep_start = (keep + 1) / 2;
    let keep_end = keep - keep_start;
    let chars: Vec<char> = text.chars().collect();
    let start = chars.iter().take(keep_start).collect::<String>();
    let end = chars.iter().skip(count - keep_end).collect::<String>();
    format!("{start}…{end}")
}

/// Split a path string near the middle, preferring a directory separator.
pub fn split_path_near_middle(path: &str) -> (String, String) {
    let chars: Vec<char> = path.chars().collect();
    let len = chars.len();
    if len == 0 {
        return (String::new(), String::new());
    }
    let mid = len / 2;
    let mut split_at = mid;

    for delta in 0..=mid {
        if mid + delta < len && (chars[mid + delta] == '/' || chars[mid + delta] == '\\') {
            split_at = mid + delta + 1;
            break;
        }
        if delta > 0 && mid >= delta && (chars[mid - delta] == '/' || chars[mid - delta] == '\\') {
            split_at = mid - delta + 1;
            break;
        }
    }

    let left: String = chars[..split_at].iter().collect();
    let right: String = chars[split_at..].iter().collect();
    (left, right)
}

fn path_text_style(el: Div) -> Div {
    el.text_base()
        .font_weight(FontWeight::MEDIUM)
        .text_color(colors::text_primary())
}

/// Single-line path label: short paths truncate at the end; long paths use middle ellipsis.
pub fn middle_truncated_path(label: impl Into<SharedString>) -> impl IntoElement {
    let label: SharedString = label.into();
    let label_str = label.to_string();
    let char_len = label_str.chars().count();

    if char_len <= PATH_MIDDLE_ELLIPSIS_THRESHOLD {
        path_text_style(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .child(label),
        )
    } else {
        let (left, right) = split_path_near_middle(&label_str);
        let left: SharedString = left.into();
        let right: SharedString = right.into();

        path_text_style(
            h_flex()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .child(
                    div()
                        .flex_shrink()
                        .min_w_0()
                        .truncate()
                        .child(left),
                )
                .child(div().flex_shrink_0().child("…"))
                .child(
                    div()
                        .flex_shrink()
                        .min_w_0()
                        .truncate()
                        .child(right),
                ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_middle_keeps_both_ends() {
        let s = "abcdefghijklmnopqrstuvwxyz";
        assert_eq!(truncate_middle(s, 10), "abcde…wxyz");
    }

    #[test]
    fn truncate_middle_short_string_unchanged() {
        assert_eq!(truncate_middle("short", 10), "short");
    }

    #[test]
    fn split_path_near_middle_prefers_separator() {
        let path = "/Users/bob/Workspace/project/target/debug";
        let (left, right) = split_path_near_middle(path);
        assert!(left.starts_with("/Users/"));
        assert!(right.contains("target"));
        assert_eq!(format!("{left}{right}"), path);
    }
}
