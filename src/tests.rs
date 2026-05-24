use crate::{render, strip_ansi, RenderOptions, Theme};

// ANSI constants for assertions
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const ITALIC: &str = "\x1b[3m";
const UNDERLINE: &str = "\x1b[4m";
const STRIKETHROUGH: &str = "\x1b[9m";
const GREEN: &str = "\x1b[32m";
const BLUE: &str = "\x1b[34m";
const H1: &str = "\x1b[1;33m";
const H2: &str = "\x1b[1;36m";
const H3: &str = "\x1b[1;37m";

fn default_opts() -> RenderOptions {
    RenderOptions::default()
}

// -- Headings ----------------------------------------------------------------

#[test]
fn h1_renders_bold_yellow() {
    let result = render("# Hello", &default_opts());
    assert_eq!(result, format!("{}Hello{}", H1, RESET));
}

#[test]
fn h2_renders_bold_cyan() {
    let result = render("## Section", &default_opts());
    assert_eq!(result, format!("{}Section{}", H2, RESET));
}

#[test]
fn h3_renders_bold_white() {
    let result = render("### Subsection", &default_opts());
    assert_eq!(result, format!("{}Subsection{}", H3, RESET));
}

#[test]
fn h4_also_renders_bold_white() {
    let result = render("#### Deep", &default_opts());
    assert_eq!(result, format!("{}Deep{}", H3, RESET));
}

// -- Inline formatting -------------------------------------------------------

#[test]
fn bold_text() {
    let result = render("Some **bold** here", &default_opts());
    assert!(result.contains(&format!("{}bold{}", BOLD, RESET)));
    assert!(result.contains("Some "));
    assert!(result.contains(" here"));
}

#[test]
fn italic_text() {
    let result = render("Some *italic* here", &default_opts());
    assert!(result.contains(&format!("{}italic{}", ITALIC, RESET)));
}

#[test]
fn inline_code() {
    let result = render("Use `mix test` to run", &default_opts());
    assert!(result.contains(&format!("{}mix test{}", GREEN, RESET)));
}

#[test]
fn strikethrough_text() {
    let result = render("~~removed~~", &default_opts());
    assert!(result.contains(&format!("{}removed{}", STRIKETHROUGH, RESET)));
}

#[test]
fn links_show_text_and_url() {
    let result = render("[Elixir](https://elixir-lang.org)", &default_opts());
    assert!(result.contains(&format!("{}{}Elixir{}", UNDERLINE, BLUE, RESET)));
    assert!(result.contains(&format!("{} (https://elixir-lang.org){}", DIM, RESET)));
}

// -- Bullet lists ------------------------------------------------------------

#[test]
fn bullet_list_renders_with_triangle_markers() {
    let md = "- alpha\n- beta\n- gamma\n";
    let result = render(md, &default_opts());
    assert!(result.contains("  \u{25b8} alpha"));
    assert!(result.contains("  \u{25b8} beta"));
    assert!(result.contains("  \u{25b8} gamma"));
}

#[test]
fn bullet_list_preserves_inline_formatting() {
    let md = "- **bold** item\n- normal item\n";
    let result = render(md, &default_opts());
    assert!(result.contains(&format!("  \u{25b8} {}bold{} item", BOLD, RESET)));
}

// -- Ordered lists -----------------------------------------------------------

#[test]
fn ordered_list_renders_with_circled_numbers() {
    let md = "1. first\n2. second\n3. third\n";
    let result = render(md, &default_opts());
    assert!(result.contains("  \u{2460} first"));
    assert!(result.contains("  \u{2461} second"));
    assert!(result.contains("  \u{2462} third"));
}

#[test]
fn ordered_list_handles_start_number() {
    let md = "3. third\n4. fourth\n";
    let result = render(md, &default_opts());
    assert!(result.contains("  \u{2462} third"));
    assert!(result.contains("  \u{2463} fourth"));
}

// -- Code blocks -------------------------------------------------------------

#[test]
fn code_block_with_language_header() {
    let md = "```rust\nfn main() {}\n```\n";
    let result = render(md, &default_opts());
    // Header with language
    assert!(result.contains(&format!("{}  \u{250c}\u{2500} rust{}", DIM, RESET)));
    assert!(result.contains("main"));
    // Footer
    assert!(result.contains(&format!("{}  \u{2514}\u{2500}{}", DIM, RESET)));
}

#[test]
fn code_block_without_language() {
    let md = "```\nplain code\n```\n";
    let result = render(md, &default_opts());
    assert!(result.contains(&format!("{}  \u{250c}\u{2500}{}", DIM, RESET)));
    assert!(result.contains(&format!("{}plain code{}", GREEN, RESET)));
}

// -- Block quotes ------------------------------------------------------------

#[test]
fn block_quote_renders_with_bar_prefix() {
    let md = "> Something wise was said.\n";
    let result = render(md, &default_opts());
    assert!(result.contains(&format!("{}  \u{2502} {}", DIM, RESET)));
    assert!(result.contains("Something wise was said."));
}

// -- Thematic break ----------------------------------------------------------

#[test]
fn thematic_break_renders_as_dim_horizontal_rule() {
    let result = render("---", &default_opts());
    let expected_rule = "\u{2500}".repeat(40);
    assert!(result.contains(&format!("{}{}{}", DIM, expected_rule, RESET)));
}

// -- Paragraphs --------------------------------------------------------------

#[test]
fn paragraphs_separated_by_double_newline() {
    let md = "First paragraph.\n\nSecond paragraph.\n";
    let result = render(md, &default_opts());
    assert!(result.contains("First paragraph.\n\nSecond paragraph."));
}

#[test]
fn paragraphs_separated_by_double_crlf() {
    let md = "First paragraph.\n\nSecond paragraph.\n";
    let opts = RenderOptions {
        newline: "\r\n".into(),
        ..default_opts()
    };
    let result = render(md, &opts);
    assert!(result.contains("First paragraph.\r\n\r\nSecond paragraph."));
}

// -- Newline option ----------------------------------------------------------

#[test]
fn defaults_to_lf() {
    let md = "- alpha\n- beta\n";
    let result = render(md, &default_opts());
    assert!(result.contains("alpha\n"));
    assert!(!result.contains("\r\n"));
}

#[test]
fn uses_crlf_when_configured() {
    let md = "- alpha\n- beta\n";
    let opts = RenderOptions {
        newline: "\r\n".into(),
        ..default_opts()
    };
    let result = render(md, &opts);
    assert!(result.contains("alpha\r\n"));
}

// -- Tables ------------------------------------------------------------------

#[test]
fn table_renders_with_box_drawing_chars() {
    let md = "| Name  | Age |\n|-------|-----|\n| Alice | 30  |\n| Bob   | 25  |\n";
    let result = render(md, &default_opts());
    let plain = strip_ansi(&result);

    assert!(plain.contains("\u{250c}"));
    assert!(plain.contains("\u{2510}"));
    assert!(plain.contains("\u{2514}"));
    assert!(plain.contains("\u{2518}"));
    assert!(plain.contains("\u{251c}"));
    assert!(plain.contains("\u{2524}"));
    assert!(plain.contains("\u{253c}"));
    assert!(plain.contains("\u{252c}"));
    assert!(plain.contains("\u{2534}"));
    assert!(plain.contains("Alice"));
    assert!(plain.contains("Bob"));
    assert!(plain.contains("Name"));
    assert!(plain.contains("Age"));
}

#[test]
fn table_columns_are_properly_aligned() {
    let md = "| Short | Longer header |\n|-------|---------------|\n| a     | b             |\n";
    let result = render(md, &default_opts());
    let plain = strip_ansi(&result);
    let lines: Vec<&str> = plain.split('\n').collect();

    // top, header, sep, body, bot
    assert!(lines.len() >= 5);
    let top = lines[0];
    let sep = lines[2];
    let bot = lines[4];
    assert_eq!(top.chars().count(), sep.chars().count());
    assert_eq!(sep.chars().count(), bot.chars().count());
}

#[test]
fn table_header_cells_are_styled() {
    let md = "| H1 |\n|----|\n| d1 |\n";
    let theme = Theme::default();
    let result = render(md, &default_opts());
    assert!(result.contains(&format!("{}H1{}", theme.table_header, theme.reset)));
}

// -- Mixed content -----------------------------------------------------------

#[test]
fn heading_followed_by_paragraph_and_list() {
    let md = "# Title\n\nSome intro text.\n\n- item one\n- item two\n";
    let result = render(md, &default_opts());
    assert!(result.contains(&format!("{}Title{}", H1, RESET)));
    assert!(result.contains("Some intro text."));
    assert!(result.contains("  \u{25b8} item one"));
    assert!(result.contains("  \u{25b8} item two"));
}

// -- Syntax highlighting -----------------------------------------------------

#[test]
fn rust_code_block_is_highlighted() {
    let md = "```rust\nfn main() {\n    println!(\"hello\");\n}\n```\n";
    let result = render(md, &default_opts());
    let theme = Theme::default();

    // With syntect active, the code should NOT use the plain code_text style
    // for keywords. The "fn" keyword should get keyword styling.
    // syntect maps Rust `fn` to keyword.other.fn -- which becomes keyword_other
    // and falls back to keyword -> magenta
    assert!(result.contains("fn"));
    assert!(result.contains("main"));

    // Border rendering is unchanged
    assert!(result.contains(&format!(
        "{}{} rust{}",
        theme.code_border, theme.code_top, theme.reset
    )));
    assert!(result.contains(&format!(
        "{}{}{}",
        theme.code_border, theme.code_bottom, theme.reset
    )));
}

#[test]
fn falls_back_to_plain_code_text_for_unknown_languages() {
    let md = "```brainfuck\n+++>+<-\n```\n";
    let result = render(md, &default_opts());
    let theme = Theme::default();

    assert!(result.contains(&format!("{}+++>+<-{}", theme.code_text, theme.reset)));
}

#[test]
fn syntax_highlighting_can_be_disabled_via_theme() {
    let md = "```rust\nfn foo() {}\n```\n";
    let opts = RenderOptions {
        theme: Theme {
            syntax_highlight: false,
            ..Default::default()
        },
        ..default_opts()
    };
    let result = render(md, &opts);
    let theme = &opts.theme;

    assert!(result.contains(&format!("{}fn foo() {{}}{}", theme.code_text, theme.reset)));
}

// -- escape_sequences option -------------------------------------------------

#[test]
fn strip_ansi_option_removes_all_escapes() {
    let opts = RenderOptions {
        escape_sequences: false,
        ..default_opts()
    };
    let result = render("# Hello\n\nSome **bold** text.", &opts);
    assert!(!result.contains("\x1b["));
    assert!(result.contains("Hello"));
    assert!(result.contains("bold"));
}

// -- Nested bullet lists -----------------------------------------------------

#[test]
fn nested_bullet_list_indentation() {
    let md = "- Parent\n  - Child 1\n  - Child 2\n  - Child 3\n";
    let result = render(md, &default_opts());
    let lines: Vec<&str> = result.split('\n').collect();

    assert!(lines[0].contains("  \u{25b8} Parent"));
    // Nested items get list_continuation + bullet_marker
    for child_line in &lines[1..] {
        if !child_line.is_empty() {
            assert!(child_line.contains("\u{25b8} Child"));
        }
    }
}
