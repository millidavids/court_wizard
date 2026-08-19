//! Lightweight markdown parser and Bevy UI renderer.
//!
//! Handles the subset of markdown used by CHANGELOG.md, CREDITS.md, and
//! INSTRUCTIONS.md: headings (h1–h3), bold, links, unordered lists,
//! horizontal rules, and paragraphs.

use bevy::prelude::*;

use crate::ui::constants::{ACTIVE_ACCENT, CONTENT_BORDER, TEXT_BODY, TEXT_MUTED, TEXT_PRIMARY};

// ── Markdown-specific styling constants ─────────────────────────────────────

const H1_FONT_SIZE: f32 = 20.0;
const H2_FONT_SIZE: f32 = 16.0;
const H3_FONT_SIZE: f32 = 13.0;
const BODY_FONT_SIZE: f32 = 11.0;

const HEADING_BOTTOM_MARGIN: f32 = 12.0;
const PARAGRAPH_BOTTOM_MARGIN: f32 = 8.0;
const LIST_ITEM_BOTTOM_MARGIN: f32 = 4.0;
const LIST_INDENT: f32 = 20.0;
const HR_HEIGHT: f32 = 1.0;
const HR_VERTICAL_MARGIN: f32 = 12.0;

// ── Data model ──────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
pub(crate) enum MarkdownBlock {
    Heading { level: u8, text: String },
    Paragraph { spans: Vec<MarkdownSpan> },
    ListItem { spans: Vec<MarkdownSpan> },
    HorizontalRule,
}

#[derive(Debug, PartialEq)]
pub(crate) enum MarkdownSpan {
    Plain(String),
    Bold(String),
    Link { text: String },
}

// ── Parser ──────────────────────────────────────────────────────────────────

/// Parse a markdown string into a list of block-level elements.
pub(crate) fn parse_markdown(input: &str) -> Vec<MarkdownBlock> {
    let mut blocks = Vec::new();
    let mut paragraph_lines: Vec<&str> = Vec::new();

    for line in input.lines() {
        let trimmed = line.trim();

        // Blank line: flush accumulated paragraph
        if trimmed.is_empty() {
            flush_paragraph(&mut paragraph_lines, &mut blocks);
            continue;
        }

        // Horizontal rule: 3+ dashes only
        if trimmed.len() >= 3 && trimmed.chars().all(|c| c == '-') {
            flush_paragraph(&mut paragraph_lines, &mut blocks);
            blocks.push(MarkdownBlock::HorizontalRule);
            continue;
        }

        // Heading: starts with 1-3 '#' followed by space
        if let Some(heading) = try_parse_heading(trimmed) {
            flush_paragraph(&mut paragraph_lines, &mut blocks);
            blocks.push(heading);
            continue;
        }

        // List item: starts with "- "
        if let Some(rest) = trimmed.strip_prefix("- ") {
            flush_paragraph(&mut paragraph_lines, &mut blocks);
            blocks.push(MarkdownBlock::ListItem {
                spans: parse_inline_spans(rest),
            });
            continue;
        }

        // Otherwise accumulate as paragraph text
        paragraph_lines.push(trimmed);
    }

    flush_paragraph(&mut paragraph_lines, &mut blocks);
    blocks
}

fn try_parse_heading(line: &str) -> Option<MarkdownBlock> {
    let bytes = line.as_bytes();
    let mut level: u8 = 0;

    for &b in bytes {
        if b == b'#' {
            level += 1;
        } else {
            break;
        }
    }

    if level == 0 || level > 3 {
        return None;
    }

    // Must be followed by a space
    if bytes.get(level as usize) != Some(&b' ') {
        return None;
    }

    let text = line[(level as usize + 1)..].trim().to_string();
    Some(MarkdownBlock::Heading { level, text })
}

fn flush_paragraph(lines: &mut Vec<&str>, blocks: &mut Vec<MarkdownBlock>) {
    if lines.is_empty() {
        return;
    }
    let joined = lines.join(" ");
    blocks.push(MarkdownBlock::Paragraph {
        spans: parse_inline_spans(&joined),
    });
    lines.clear();
}

/// Parse inline spans: `**bold**` and `[text](url)` within a string.
fn parse_inline_spans(input: &str) -> Vec<MarkdownSpan> {
    let mut spans = Vec::new();
    let mut plain_buf = String::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        // Bold: **...**
        if i + 1 < chars.len()
            && chars[i] == '*'
            && chars[i + 1] == '*'
            && let Some(end) = find_closing_bold(&chars, i + 2)
        {
            if !plain_buf.is_empty() {
                spans.push(MarkdownSpan::Plain(std::mem::take(&mut plain_buf)));
            }
            let bold_text: String = chars[i + 2..end].iter().collect();
            spans.push(MarkdownSpan::Bold(bold_text));
            i = end + 2; // skip closing **
            continue;
        }

        // Link: [text](url)
        if chars[i] == '['
            && let Some((link_text, end)) = try_parse_link(&chars, i)
        {
            if !plain_buf.is_empty() {
                spans.push(MarkdownSpan::Plain(std::mem::take(&mut plain_buf)));
            }
            spans.push(MarkdownSpan::Link { text: link_text });
            i = end;
            continue;
        }

        plain_buf.push(chars[i]);
        i += 1;
    }

    if !plain_buf.is_empty() {
        spans.push(MarkdownSpan::Plain(plain_buf));
    }

    spans
}

/// Find closing `**` starting from position `start` (which is after the opening `**`).
fn find_closing_bold(chars: &[char], start: usize) -> Option<usize> {
    let mut i = start;
    while i + 1 < chars.len() {
        if chars[i] == '*' && chars[i + 1] == '*' {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Try to parse `[text](url)` starting at position `start` (which is at `[`).
/// Returns (link_text, position_after_closing_paren).
fn try_parse_link(chars: &[char], start: usize) -> Option<(String, usize)> {
    // Find closing ]
    let mut i = start + 1;
    while i < chars.len() && chars[i] != ']' {
        i += 1;
    }
    if i >= chars.len() {
        return None;
    }
    let bracket_end = i;

    // Must be followed by (
    if bracket_end + 1 >= chars.len() || chars[bracket_end + 1] != '(' {
        return None;
    }

    // Find closing )
    i = bracket_end + 2;
    while i < chars.len() && chars[i] != ')' {
        i += 1;
    }
    if i >= chars.len() {
        return None;
    }

    let link_text: String = chars[start + 1..bracket_end].iter().collect();
    Some((link_text, i + 1))
}

// ── Renderer ────────────────────────────────────────────────────────────────

/// Spawn formatted markdown content as Bevy UI nodes into a parent entity.
pub(crate) fn spawn_markdown(parent: &mut ChildSpawnerCommands, blocks: &[MarkdownBlock]) {
    for block in blocks {
        match block {
            MarkdownBlock::Heading { level, text } => {
                spawn_heading(parent, *level, text);
            }
            MarkdownBlock::Paragraph { spans } => {
                spawn_paragraph(parent, spans);
            }
            MarkdownBlock::ListItem { spans } => {
                spawn_list_item(parent, spans);
            }
            MarkdownBlock::HorizontalRule => {
                spawn_hr(parent);
            }
        }
    }
}

fn spawn_heading(parent: &mut ChildSpawnerCommands, level: u8, text: &str) {
    let (font_size, color) = match level {
        1 => (H1_FONT_SIZE, ACTIVE_ACCENT),
        2 => (H2_FONT_SIZE, ACTIVE_ACCENT),
        _ => (H3_FONT_SIZE, TEXT_PRIMARY),
    };

    parent
        .spawn(Node {
            margin: UiRect::bottom(Val::Px(HEADING_BOTTOM_MARGIN)),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new(text),
                TextFont::from_font_size(font_size),
                TextColor(color),
                TextLayout::linebreak(bevy::text::LineBreak::WordBoundary),
            ));
        });
}

fn spawn_paragraph(parent: &mut ChildSpawnerCommands, spans: &[MarkdownSpan]) {
    parent
        .spawn(Node {
            margin: UiRect::bottom(Val::Px(PARAGRAPH_BOTTOM_MARGIN)),
            ..default()
        })
        .with_children(|wrapper| {
            spawn_rich_text(wrapper, spans, BODY_FONT_SIZE, None);
        });
}

fn spawn_list_item(parent: &mut ChildSpawnerCommands, spans: &[MarkdownSpan]) {
    parent
        .spawn(Node {
            padding: UiRect::left(Val::Px(LIST_INDENT)),
            margin: UiRect::bottom(Val::Px(LIST_ITEM_BOTTOM_MARGIN)),
            ..default()
        })
        .with_children(|wrapper| {
            // Bullet prefix "- " as the root Text, content spans as TextSpan children
            spawn_rich_text(wrapper, spans, BODY_FONT_SIZE, Some(("- ", TEXT_MUTED)));
        });
}

fn spawn_hr(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Node {
            width: Val::Percent(90.0),
            height: Val::Px(HR_HEIGHT),
            margin: UiRect::axes(Val::Auto, Val::Px(HR_VERTICAL_MARGIN)),
            ..default()
        },
        BackgroundColor(CONTENT_BORDER),
    ));
}

/// Spawn a rich text block using a root `Text` with `TextSpan` children.
/// All spans share one text layout so inline bold/links don't force line breaks.
/// An optional `prefix` (e.g. bullet "- ") becomes the root text content.
fn spawn_rich_text(
    parent: &mut ChildSpawnerCommands,
    spans: &[MarkdownSpan],
    font_size: f32,
    prefix: Option<(&str, Color)>,
) {
    // Determine root text content and color
    let (root_text, root_color, child_spans) = if let Some((pfx, pfx_color)) = prefix {
        (pfx.to_string(), pfx_color, spans)
    } else {
        // First span becomes the root Text, rest become TextSpan children
        match spans.first() {
            Some(span) => {
                let (text, color) = span_content_and_color(span);
                (text.to_string(), color, &spans[1..])
            }
            None => return,
        }
    };

    parent
        .spawn((
            Text::new(root_text),
            TextFont::from_font_size(font_size),
            TextColor(root_color),
            TextLayout::linebreak(bevy::text::LineBreak::WordBoundary),
        ))
        .with_children(|text_entity| {
            for span in child_spans {
                let (content, color) = span_content_and_color(span);
                text_entity.spawn((
                    TextSpan::new(content),
                    TextFont::from_font_size(font_size),
                    TextColor(color),
                ));
            }
        });
}

fn span_content_and_color(span: &MarkdownSpan) -> (&str, Color) {
    match span {
        MarkdownSpan::Plain(text) => (text.as_str(), TEXT_BODY),
        MarkdownSpan::Bold(text) => (text.as_str(), TEXT_PRIMARY),
        MarkdownSpan::Link { text } => (text.as_str(), TEXT_BODY),
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_heading_levels() {
        let input = "# Title\n## Section\n### Sub";
        let blocks = parse_markdown(input);
        assert_eq!(blocks.len(), 3);
        assert_eq!(
            blocks[0],
            MarkdownBlock::Heading {
                level: 1,
                text: "Title".to_string()
            }
        );
        assert_eq!(
            blocks[1],
            MarkdownBlock::Heading {
                level: 2,
                text: "Section".to_string()
            }
        );
        assert_eq!(
            blocks[2],
            MarkdownBlock::Heading {
                level: 3,
                text: "Sub".to_string()
            }
        );
    }

    #[test]
    fn parse_four_hashes_is_paragraph() {
        let input = "#### Not a heading";
        let blocks = parse_markdown(input);
        assert_eq!(blocks.len(), 1);
        assert!(matches!(&blocks[0], MarkdownBlock::Paragraph { .. }));
    }

    #[test]
    fn parse_list_items() {
        let input = "- First item\n- Second item";
        let blocks = parse_markdown(input);
        assert_eq!(blocks.len(), 2);
        assert!(matches!(&blocks[0], MarkdownBlock::ListItem { .. }));
        assert!(matches!(&blocks[1], MarkdownBlock::ListItem { .. }));
    }

    #[test]
    fn parse_bold_in_list_item() {
        let input = "- **Feature** -- description here";
        let blocks = parse_markdown(input);
        assert_eq!(blocks.len(), 1);
        if let MarkdownBlock::ListItem { spans } = &blocks[0] {
            assert_eq!(spans.len(), 2);
            assert_eq!(spans[0], MarkdownSpan::Bold("Feature".to_string()));
            assert_eq!(
                spans[1],
                MarkdownSpan::Plain(" -- description here".to_string())
            );
        } else {
            panic!("Expected ListItem");
        }
    }

    #[test]
    fn parse_link() {
        let input = "Built with [Bevy](https://bevyengine.org/) engine";
        let blocks = parse_markdown(input);
        assert_eq!(blocks.len(), 1);
        if let MarkdownBlock::Paragraph { spans } = &blocks[0] {
            assert_eq!(spans.len(), 3);
            assert_eq!(spans[0], MarkdownSpan::Plain("Built with ".to_string()));
            assert_eq!(
                spans[1],
                MarkdownSpan::Link {
                    text: "Bevy".to_string()
                }
            );
            assert_eq!(spans[2], MarkdownSpan::Plain(" engine".to_string()));
        } else {
            panic!("Expected Paragraph");
        }
    }

    #[test]
    fn parse_horizontal_rule() {
        let input = "Above\n\n---\n\nBelow";
        let blocks = parse_markdown(input);
        assert_eq!(blocks.len(), 3);
        assert!(matches!(&blocks[0], MarkdownBlock::Paragraph { .. }));
        assert_eq!(blocks[1], MarkdownBlock::HorizontalRule);
        assert!(matches!(&blocks[2], MarkdownBlock::Paragraph { .. }));
    }

    #[test]
    fn parse_paragraph_joins_lines() {
        let input = "Line one\nLine two\n\nNew paragraph";
        let blocks = parse_markdown(input);
        assert_eq!(blocks.len(), 2);
        if let MarkdownBlock::Paragraph { spans } = &blocks[0] {
            assert_eq!(
                spans[0],
                MarkdownSpan::Plain("Line one Line two".to_string())
            );
        } else {
            panic!("Expected Paragraph");
        }
    }

    #[test]
    fn parse_empty_input() {
        let blocks = parse_markdown("");
        assert!(blocks.is_empty());
    }

    #[test]
    fn parse_bold_only() {
        let spans = parse_inline_spans("**all bold**");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0], MarkdownSpan::Bold("all bold".to_string()));
    }

    #[test]
    fn parse_unclosed_bold_is_plain() {
        let spans = parse_inline_spans("**unclosed bold");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0], MarkdownSpan::Plain("**unclosed bold".to_string()));
    }

    #[test]
    fn parse_unclosed_link_is_plain() {
        let spans = parse_inline_spans("[unclosed link");
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0], MarkdownSpan::Plain("[unclosed link".to_string()));
    }

    #[test]
    fn parse_mixed_changelog_entry() {
        let input = "## [v0.6.48] - 2026-04-10\n\n### Changed\n- **Ogre hitbox** -- bigger\n- **Ogre health** -- lower";
        let blocks = parse_markdown(input);
        assert_eq!(blocks.len(), 4);
        assert_eq!(
            blocks[0],
            MarkdownBlock::Heading {
                level: 2,
                text: "[v0.6.48] - 2026-04-10".to_string()
            }
        );
        assert_eq!(
            blocks[1],
            MarkdownBlock::Heading {
                level: 3,
                text: "Changed".to_string()
            }
        );
        assert!(matches!(&blocks[2], MarkdownBlock::ListItem { .. }));
        assert!(matches!(&blocks[3], MarkdownBlock::ListItem { .. }));
    }
}
