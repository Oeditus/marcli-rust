//! Converts CommonMark Markdown to ANSI-escaped terminal output.
//!
//! Parses Markdown via [comrak](https://docs.rs/comrak) and produces strings
//! with ANSI escape sequences suitable for terminal rendering.
//!
//! # Supported Elements
//!
//! - Headings (h1: bold yellow, h2: bold cyan, h3+: bold white)
//! - Bold, italic, strikethrough, inline code
//! - Bullet lists (triangle markers) and ordered lists (circled numbers)
//! - Code blocks with optional language headers (syntax-highlighted via syntect)
//! - Block quotes (vertical bar prefix)
//! - Thematic breaks (horizontal rules)
//! - Links (underlined blue with dimmed URL)
//! - Images (bracketed alt text with URL)
//! - Task list items (checkbox markers)
//! - Tables with box-drawing borders
//!
//! # Example
//!
//! ```
//! let output = marcli::render("# Hello\n\nSome **bold** text.", &Default::default());
//! assert!(output.contains("Hello"));
//! ```

pub mod formatter;
pub mod theme;

pub use theme::Theme;

use comrak::nodes::{ListType, NodeValue, TableAlignment};
use comrak::{parse_document, Arena};
use once_cell::sync::Lazy;
use regex::Regex;
use syntect::parsing::SyntaxSet;

static ANSI_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\x1b\[[0-9;]*m").unwrap());
static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);

/// Rendering options.
#[derive(Debug, Clone)]
pub struct RenderOptions {
    /// Line ending sequence (default: `"\n"`).
    pub newline: String,
    /// Visual theme controlling all styles.
    pub theme: Theme,
    /// When `false`, strips all ANSI escape sequences from output.
    pub escape_sequences: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            newline: "\n".into(),
            theme: Theme::default(),
            escape_sequences: true,
        }
    }
}

/// Renders a Markdown string as ANSI-escaped terminal output.
///
/// Returns a string with embedded ANSI escape sequences. Line endings
/// default to `"\n"` but can be overridden via [`RenderOptions`].
pub fn render(markdown: &str, opts: &RenderOptions) -> String {
    let arena = Arena::new();
    let mut comrak_opts = comrak::Options::default();
    comrak_opts.extension.strikethrough = true;
    comrak_opts.extension.tasklist = true;
    comrak_opts.extension.table = true;
    comrak_opts.extension.autolink = true;

    let root = parse_document(&arena, markdown, &comrak_opts);
    let result = render_document(root, &opts.newline, &opts.theme);

    if opts.escape_sequences {
        result
    } else {
        strip_ansi(&result)
    }
}

/// Render the document root node.
fn render_document<'a>(
    root: &'a comrak::nodes::AstNode<'a>,
    nl: &str,
    theme: &Theme,
) -> String {
    let blocks: Vec<String> = root
        .children()
        .map(|child| render_block(child, nl, theme))
        .filter(|s| !s.is_empty())
        .collect();

    let sep = format!("{}{}", nl, nl);
    blocks.join(&sep)
}

// -- Block-level nodes -------------------------------------------------------

fn render_block<'a>(node: &'a comrak::nodes::AstNode<'a>, nl: &str, theme: &Theme) -> String {
    let val = node.data.borrow().value.clone();
    match val {
        NodeValue::Heading(heading) => {
            let style = match heading.level {
                1 => &theme.h1,
                2 => &theme.h2,
                _ => &theme.h3,
            };
            format!(
                "{}{}{}",
                style,
                render_inline_children(node, theme),
                theme.reset
            )
        }

        NodeValue::Paragraph => render_inline_children(node, theme),

        NodeValue::List(list) => match list.list_type {
            ListType::Bullet => {
                let items: Vec<String> = node
                    .children()
                    .map(|child| render_bullet_item(child, nl, theme))
                    .collect();
                items.join(nl)
            }
            ListType::Ordered => {
                let start = list.start;
                let items: Vec<String> = node
                    .children()
                    .enumerate()
                    .map(|(i, child)| render_ordered_item(child, start + i, nl, theme))
                    .collect();
                items.join(nl)
            }
        },

        NodeValue::CodeBlock(code_block) => {
            let code = code_block.literal.trim_end_matches('\n');
            let info = &code_block.info;
            let highlighted = maybe_highlight(code, info, theme);
            let source = highlighted.as_deref().unwrap_or(code);
            let lines: Vec<&str> = source.split('\n').collect();
            let prefix = format!("{}{}{}", theme.code_border, theme.code_left, theme.reset);

            let header = if !info.is_empty() {
                format!(
                    "{}{} {}{}{}",
                    theme.code_border, theme.code_top, info, theme.reset, nl
                )
            } else {
                format!("{}{}{}{}", theme.code_border, theme.code_top, theme.reset, nl)
            };

            let body = lines
                .iter()
                .map(|line| {
                    if highlighted.is_some() {
                        format!("{}{}", prefix, line)
                    } else {
                        format!("{}{}{}{}", prefix, theme.code_text, line, theme.reset)
                    }
                })
                .collect::<Vec<_>>()
                .join(nl);

            let footer = format!(
                "{}{}{}{}",
                nl, theme.code_border, theme.code_bottom, theme.reset
            );

            format!("{}{}{}", header, body, footer)
        }

        NodeValue::BlockQuote => {
            let inner: Vec<String> = node
                .children()
                .map(|child| render_block(child, nl, theme))
                .collect();
            let joined = inner.join(nl);
            joined
                .split(nl)
                .map(|line| {
                    format!(
                        "{}{}{}{}",
                        theme.block_quote, theme.block_quote_prefix, theme.reset, line
                    )
                })
                .collect::<Vec<_>>()
                .join(nl)
        }

        NodeValue::ThematicBreak => {
            format!(
                "{}{}{}",
                theme.thematic_break,
                theme.thematic_break_char.repeat(theme.thematic_break_width),
                theme.reset
            )
        }

        NodeValue::HtmlBlock(block) => {
            let text = block.literal.trim_end_matches('\n');
            format!("{}{}{}", theme.html_block, text, theme.reset)
        }

        NodeValue::Table(ref table) => render_table(node, &table.alignments, nl, theme),

        // Catch-all: try to render children, or return empty
        _ => {
            let children: Vec<String> = node
                .children()
                .map(|child| render_block(child, nl, theme))
                .collect();
            if children.is_empty() {
                String::new()
            } else {
                children.join(nl)
            }
        }
    }
}

// -- List items --------------------------------------------------------------

fn render_bullet_item<'a>(
    node: &'a comrak::nodes::AstNode<'a>,
    nl: &str,
    theme: &Theme,
) -> String {
    let val = node.data.borrow().value.clone();
    match val {
        NodeValue::TaskItem(checked) => {
            let marker = if checked.is_some() {
                &theme.task_checked
            } else {
                &theme.task_unchecked
            };
            format!("{}{}", marker, render_item_content(node, nl, theme))
        }
        NodeValue::Item(_) => {
            format!(
                "{}{}",
                theme.bullet_marker,
                render_item_content(node, nl, theme)
            )
        }
        _ => render_block(node, nl, theme),
    }
}

fn render_ordered_item<'a>(
    node: &'a comrak::nodes::AstNode<'a>,
    idx: usize,
    nl: &str,
    theme: &Theme,
) -> String {
    let val = node.data.borrow().value.clone();
    match val {
        NodeValue::Item(_) => {
            format!(
                "{}{} {}",
                theme.ordered_indent,
                glyph_string(idx, theme),
                render_item_content(node, nl, theme)
            )
        }
        _ => render_block(node, nl, theme),
    }
}

/// Tight list items contain a single paragraph; loose items may contain several blocks.
fn render_item_content<'a>(
    node: &'a comrak::nodes::AstNode<'a>,
    nl: &str,
    theme: &Theme,
) -> String {
    let children: Vec<&comrak::nodes::AstNode<'a>> = node.children().collect();

    if children.len() == 1 {
        if let NodeValue::Paragraph = &children[0].data.borrow().value {
            return render_inline_children(children[0], theme);
        }
    }

    let cont = &theme.list_continuation;
    children
        .iter()
        .enumerate()
        .map(|(i, child)| {
            let rendered = match &child.data.borrow().value {
                NodeValue::Paragraph => render_inline_children(child, theme),
                _ => {
                    let block = render_block(child, nl, theme);
                    indent_continuation(&block, nl, cont)
                }
            };
            if i > 0 {
                format!("{}{}", cont, rendered)
            } else {
                rendered
            }
        })
        .collect::<Vec<_>>()
        .join(&format!("{}{}", nl, cont))
}

// -- Inline nodes ------------------------------------------------------------

fn render_inline_children<'a>(node: &'a comrak::nodes::AstNode<'a>, theme: &Theme) -> String {
    node.children()
        .map(|child| render_inline_node(child, theme))
        .collect::<Vec<_>>()
        .join("")
}

fn render_inline_node<'a>(node: &'a comrak::nodes::AstNode<'a>, theme: &Theme) -> String {
    let val = node.data.borrow().value.clone();
    match val {
        NodeValue::Text(ref text) => text.clone(),

        NodeValue::Strong => {
            format!(
                "{}{}{}",
                theme.bold,
                render_inline_children(node, theme),
                theme.reset
            )
        }

        NodeValue::Emph => {
            format!(
                "{}{}{}",
                theme.italic,
                render_inline_children(node, theme),
                theme.reset
            )
        }

        NodeValue::Strikethrough => {
            format!(
                "{}{}{}",
                theme.strikethrough,
                render_inline_children(node, theme),
                theme.reset
            )
        }

        NodeValue::Code(ref code) => {
            format!("{}{}{}", theme.inline_code, code.literal, theme.reset)
        }

        NodeValue::Link(ref link) => {
            format!(
                "{}{}{}{} ({}){}",
                theme.link_text,
                render_inline_children(node, theme),
                theme.reset,
                theme.link_url,
                link.url,
                theme.reset
            )
        }

        NodeValue::Image(ref link) => {
            format!(
                "{}{}{}{}{}{} ({}){}",
                theme.image_text,
                theme.image_prefix,
                render_inline_children(node, theme),
                theme.image_suffix,
                theme.reset,
                theme.image_url,
                link.url,
                theme.reset
            )
        }

        NodeValue::SoftBreak => " ".into(),
        NodeValue::LineBreak => "\n".into(),

        NodeValue::HtmlInline(ref html) => html.clone(),

        _ => {
            let children_text = render_inline_children(node, theme);
            if !children_text.is_empty() {
                children_text
            } else {
                String::new()
            }
        }
    }
}

// -- Table rendering ---------------------------------------------------------

fn render_table<'a>(
    node: &'a comrak::nodes::AstNode<'a>,
    alignments: &[TableAlignment],
    nl: &str,
    theme: &Theme,
) -> String {
    let mut header_data: Vec<Vec<String>> = Vec::new();
    let mut body_data: Vec<Vec<String>> = Vec::new();

    for row_node in node.children() {
        if let NodeValue::TableRow(is_header) = &row_node.data.borrow().value {
            let is_header = *is_header;
            let cells: Vec<String> = row_node
                .children()
                .map(|cell| {
                    if let NodeValue::TableCell = &cell.data.borrow().value {
                        render_inline_children(cell, theme)
                    } else {
                        String::new()
                    }
                })
                .collect();

            if is_header {
                header_data.push(cells);
            } else {
                body_data.push(cells);
            }
        }
    }

    let all_data: Vec<&Vec<String>> = header_data.iter().chain(body_data.iter()).collect();

    if all_data.is_empty() {
        return String::new();
    }

    let num_cols = all_data.iter().map(|r| r.len()).max().unwrap_or(0);
    if num_cols == 0 {
        return String::new();
    }

    let col_widths: Vec<usize> = (0..num_cols)
        .map(|col| {
            all_data
                .iter()
                .map(|row| row.get(col).map(|c| visual_width(c)).unwrap_or(0))
                .max()
                .unwrap_or(0)
                .max(1)
        })
        .collect();

    let c = &theme.table_chars;
    let b = &theme.table_border;
    let r = &theme.reset;
    let v = format!("{}{}{}", b, c.v, r);

    let top = format!(
        "{}{}{}",
        b,
        table_border_line(&c.tl, &c.h, &c.tm, &c.tr, &col_widths),
        r
    );
    let sep = format!(
        "{}{}{}",
        b,
        table_border_line(&c.lm, &c.h, &c.x, &c.rm, &col_widths),
        r
    );
    let bot = format!(
        "{}{}{}",
        b,
        table_border_line(&c.bl, &c.h, &c.bm, &c.br, &col_widths),
        r
    );

    let header_lines: Vec<String> = header_data
        .iter()
        .map(|row| render_table_row(row, &col_widths, alignments, &v, theme, true))
        .collect();

    let body_lines: Vec<String> = body_data
        .iter()
        .map(|row| render_table_row(row, &col_widths, alignments, &v, theme, false))
        .collect();

    let mut parts = vec![top];
    parts.extend(header_lines);
    if !header_data.is_empty() {
        parts.push(sep);
    }
    parts.extend(body_lines);
    parts.push(bot);

    parts.join(nl)
}

fn table_border_line(
    left: &str,
    h: &str,
    mid: &str,
    right: &str,
    col_widths: &[usize],
) -> String {
    let inner: Vec<String> = col_widths.iter().map(|w| h.repeat(w + 2)).collect();
    format!("{}{}{}", left, inner.join(mid), right)
}

fn render_table_row(
    cells: &[String],
    col_widths: &[usize],
    alignments: &[TableAlignment],
    v: &str,
    theme: &Theme,
    is_header: bool,
) -> String {
    let inner: Vec<String> = col_widths
        .iter()
        .enumerate()
        .map(|(idx, &width)| {
            let content = cells.get(idx).cloned().unwrap_or_default();
            let alignment = alignments.get(idx).copied().unwrap_or(TableAlignment::None);
            let padded = pad_cell(&content, width, alignment);

            if is_header {
                format!(" {}{}{} ", theme.table_header, padded, theme.reset)
            } else {
                format!(" {} ", padded)
            }
        })
        .collect();

    format!("{}{}{}", v, inner.join(v), v)
}

fn pad_cell(content: &str, width: usize, alignment: TableAlignment) -> String {
    let vis_w = visual_width(content);
    let padding = width.saturating_sub(vis_w);

    match alignment {
        TableAlignment::Right => format!("{}{}", " ".repeat(padding), content),
        TableAlignment::Center => {
            let left = padding / 2;
            let right = padding - left;
            format!("{}{}{}", " ".repeat(left), content, " ".repeat(right))
        }
        _ => format!("{}{}", content, " ".repeat(padding)),
    }
}

// -- Helpers -----------------------------------------------------------------

fn glyph_string(n: usize, theme: &Theme) -> String {
    if n >= 1 {
        theme
            .ordered_glyphs
            .get(n - 1)
            .cloned()
            .unwrap_or_else(|| format!("({})", n))
    } else {
        format!("({})", n)
    }
}

fn indent_continuation(text: &str, nl: &str, indent: &str) -> String {
    let parts: Vec<&str> = text.split(nl).collect();
    if parts.len() <= 1 {
        return text.to_string();
    }
    let mut result = parts[0].to_string();
    for part in &parts[1..] {
        result.push_str(nl);
        result.push_str(indent);
        result.push_str(part);
    }
    result
}

fn visual_width(text: &str) -> usize {
    strip_ansi(text).chars().count()
}

/// Strip all ANSI escape sequences from text.
pub fn strip_ansi(text: &str) -> String {
    ANSI_RE.replace_all(text, "").into_owned()
}

/// Attempt syntax highlighting via syntect when a language is specified.
fn maybe_highlight(code: &str, info: &str, theme: &Theme) -> Option<String> {
    if !theme.syntax_highlight {
        return None;
    }
    let lang = extract_language(info)?;
    if lang.is_empty() {
        return None;
    }
    formatter::highlight(code, &lang, &theme.syntax, &theme.reset, &SYNTAX_SET)
}

fn extract_language(info: &str) -> Option<String> {
    let lang = info.split_whitespace().next()?.to_lowercase();
    if lang.is_empty() {
        None
    } else {
        Some(lang)
    }
}

#[cfg(test)]
mod tests;
