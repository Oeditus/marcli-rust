//! Theme configuration for terminal Markdown rendering.
//!
//! Every visual aspect of the rendered output is controlled by fields
//! in [`Theme`]: ANSI escape sequences, marker characters, glyphs,
//! box-drawing characters, and sizing.
//!
//! # Loading from file
//!
//! ```no_run
//! let theme = marcli::Theme::load(".marcli.toml").unwrap_or_default();
//! let output = marcli::render("# Hello", &marcli::RenderOptions { theme, ..Default::default() });
//! ```
//!
//! The file must be valid TOML. Keys that match struct fields override
//! the defaults; unknown keys are silently ignored.

use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// Box-drawing characters used for table rendering.
#[derive(Debug, Clone, Deserialize)]
pub struct TableChars {
    /// Top-left corner
    pub tl: String,
    /// Top-right corner
    pub tr: String,
    /// Bottom-left corner
    pub bl: String,
    /// Bottom-right corner
    pub br: String,
    /// Horizontal line
    pub h: String,
    /// Vertical line
    pub v: String,
    /// Top-mid junction
    pub tm: String,
    /// Bottom-mid junction
    pub bm: String,
    /// Left-mid junction
    pub lm: String,
    /// Right-mid junction
    pub rm: String,
    /// Cross junction
    pub x: String,
}

impl Default for TableChars {
    fn default() -> Self {
        Self {
            tl: "\u{250c}".into(),
            tr: "\u{2510}".into(),
            bl: "\u{2514}".into(),
            br: "\u{2518}".into(),
            h: "\u{2500}".into(),
            v: "\u{2502}".into(),
            tm: "\u{252c}".into(),
            bm: "\u{2534}".into(),
            lm: "\u{251c}".into(),
            rm: "\u{2524}".into(),
            x: "\u{253c}".into(),
        }
    }
}

/// Token-type to ANSI escape mapping for syntax highlighting.
///
/// When a specific token type (e.g. `keyword_constant`) is not present,
/// the formatter walks up the hierarchy by stripping trailing segments:
/// `keyword_constant` -> `keyword` -> unstyled.
pub fn default_syntax() -> HashMap<String, String> {
    let mut m = HashMap::new();
    // Comments
    m.insert("comment".into(), "\x1b[3;90m".into());
    // Errors
    m.insert("error".into(), "\x1b[1;31m".into());
    // Keywords
    m.insert("keyword".into(), "\x1b[35m".into());
    m.insert("keyword_constant".into(), "\x1b[36m".into());
    m.insert("keyword_declaration".into(), "\x1b[35m".into());
    m.insert("keyword_namespace".into(), "\x1b[35m".into());
    m.insert("keyword_pseudo".into(), "\x1b[35m".into());
    m.insert("keyword_reserved".into(), "\x1b[35m".into());
    m.insert("keyword_type".into(), "\x1b[36m".into());
    // Names
    m.insert("name_attribute".into(), "\x1b[33m".into());
    m.insert("name_builtin".into(), "\x1b[36m".into());
    m.insert("name_builtin_pseudo".into(), "\x1b[36m".into());
    m.insert("name_class".into(), "\x1b[1;36m".into());
    m.insert("name_constant".into(), "\x1b[1;33m".into());
    m.insert("name_decorator".into(), "\x1b[33m".into());
    m.insert("name_entity".into(), "\x1b[1;33m".into());
    m.insert("name_exception".into(), "\x1b[1;31m".into());
    m.insert("name_function".into(), "\x1b[33m".into());
    m.insert("name_function_magic".into(), "\x1b[33m".into());
    m.insert("name_label".into(), "\x1b[36m".into());
    m.insert("name_namespace".into(), "\x1b[1;36m".into());
    m.insert("name_tag".into(), "\x1b[1;35m".into());
    m.insert("name_variable".into(), "\x1b[37m".into());
    // Strings
    m.insert("string".into(), "\x1b[32m".into());
    m.insert("string_char".into(), "\x1b[32m".into());
    m.insert("string_delimiter".into(), "\x1b[32m".into());
    m.insert("string_doc".into(), "\x1b[3;32m".into());
    m.insert("string_escape".into(), "\x1b[1;32m".into());
    m.insert("string_interpol".into(), "\x1b[1;32m".into());
    m.insert("string_regex".into(), "\x1b[31m".into());
    m.insert("string_sigil".into(), "\x1b[32m".into());
    m.insert("string_symbol".into(), "\x1b[36m".into());
    // Numbers
    m.insert("number".into(), "\x1b[34m".into());
    // Operators
    m.insert("operator".into(), String::new());
    m.insert("operator_word".into(), "\x1b[35m".into());
    // Punctuation
    m.insert("punctuation".into(), String::new());
    // Generic
    m.insert("generic_deleted".into(), "\x1b[31m".into());
    m.insert("generic_emph".into(), "\x1b[3m".into());
    m.insert("generic_error".into(), "\x1b[31m".into());
    m.insert("generic_heading".into(), "\x1b[1m".into());
    m.insert("generic_inserted".into(), "\x1b[32m".into());
    m.insert("generic_output".into(), "\x1b[90m".into());
    m.insert("generic_prompt".into(), "\x1b[1m".into());
    m.insert("generic_strong".into(), "\x1b[1m".into());
    m.insert("generic_subheading".into(), "\x1b[1;35m".into());
    m.insert("generic_traceback".into(), "\x1b[31m".into());
    m
}

/// Full theme controlling all visual aspects of Markdown rendering.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Theme {
    /// ANSI reset sequence
    pub reset: String,

    // -- Headings
    pub h1: String,
    pub h2: String,
    pub h3: String,

    // -- Inline styles
    pub bold: String,
    pub italic: String,
    pub strikethrough: String,
    pub inline_code: String,

    // -- Links
    pub link_text: String,
    pub link_url: String,

    // -- Images
    pub image_text: String,
    pub image_prefix: String,
    pub image_suffix: String,
    pub image_url: String,

    // -- Code blocks
    pub code_border: String,
    pub code_text: String,
    pub code_top: String,
    pub code_left: String,
    pub code_bottom: String,

    // -- Block quotes
    pub block_quote: String,
    pub block_quote_prefix: String,

    // -- Bullet lists
    pub bullet_marker: String,
    pub task_checked: String,
    pub task_unchecked: String,

    // -- Ordered lists
    pub ordered_indent: String,
    pub ordered_glyphs: Vec<String>,

    // -- List continuation indent (loose list items)
    pub list_continuation: String,

    // -- Thematic breaks
    pub thematic_break: String,
    pub thematic_break_char: String,
    pub thematic_break_width: usize,

    // -- HTML blocks
    pub html_block: String,

    // -- Tables
    pub table_border: String,
    pub table_header: String,
    pub table_chars: TableChars,

    // -- Syntax highlighting
    pub syntax_highlight: bool,
    pub syntax: HashMap<String, String>,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            reset: "\x1b[0m".into(),

            h1: "\x1b[1;33m".into(),
            h2: "\x1b[1;36m".into(),
            h3: "\x1b[1;37m".into(),

            bold: "\x1b[1m".into(),
            italic: "\x1b[3m".into(),
            strikethrough: "\x1b[9m".into(),
            inline_code: "\x1b[32m".into(),

            link_text: "\x1b[4m\x1b[34m".into(),
            link_url: "\x1b[2m".into(),

            image_text: "\x1b[2m".into(),
            image_prefix: "[image: ".into(),
            image_suffix: "]".into(),
            image_url: "\x1b[2m".into(),

            code_border: "\x1b[2m".into(),
            code_text: "\x1b[32m".into(),
            code_top: "  \u{250c}\u{2500}".into(),
            code_left: "  \u{2502} ".into(),
            code_bottom: "  \u{2514}\u{2500}".into(),

            block_quote: "\x1b[2m".into(),
            block_quote_prefix: "  \u{2502} ".into(),

            bullet_marker: "  \u{25b8} ".into(),
            task_checked: "  \u{2611} ".into(),
            task_unchecked: "  \u{2610} ".into(),

            ordered_indent: "  ".into(),
            ordered_glyphs: vec![
                "\u{2460}".into(),
                "\u{2461}".into(),
                "\u{2462}".into(),
                "\u{2463}".into(),
                "\u{2464}".into(),
                "\u{2465}".into(),
                "\u{2466}".into(),
                "\u{2467}".into(),
                "\u{2468}".into(),
                "\u{2469}".into(),
                "\u{246a}".into(),
                "\u{246b}".into(),
                "\u{246c}".into(),
                "\u{246d}".into(),
                "\u{246e}".into(),
                "\u{246f}".into(),
                "\u{2470}".into(),
                "\u{2471}".into(),
                "\u{2472}".into(),
                "\u{2473}".into(),
            ],

            list_continuation: "    ".into(),

            thematic_break: "\x1b[2m".into(),
            thematic_break_char: "\u{2500}".into(),
            thematic_break_width: 40,

            html_block: "\x1b[2m".into(),

            table_border: "\x1b[2m".into(),
            table_header: "\x1b[1m".into(),
            table_chars: TableChars::default(),

            syntax_highlight: true,
            syntax: default_syntax(),
        }
    }
}

impl Theme {
    /// Loads a theme from a TOML file at the given path.
    ///
    /// Keys that match struct fields override the defaults; unknown keys
    /// are silently ignored. Returns `Ok(default())` when the file does
    /// not exist.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = std::fs::read_to_string(path)?;
        let theme: Theme = toml::from_str(&contents)?;
        Ok(theme)
    }
}
