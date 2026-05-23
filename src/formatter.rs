//! Syntax highlighting formatter producing ANSI-escaped terminal output.
//!
//! Maps syntect scope selectors to ANSI escape sequences using a
//! configurable style map from [`crate::Theme`].
//!
//! ## Token type fallback
//!
//! When a specific token type (e.g. `keyword_constant`) is not present
//! in the style map, the formatter walks up the hierarchy by stripping
//! trailing segments: `keyword_constant` -> `keyword` -> unstyled.

use std::collections::HashMap;

use syntect::parsing::{SyntaxReference, SyntaxSet};

/// Highlights `code` using syntect and maps the resulting tokens to ANSI
/// escape sequences via the theme's syntax map.
///
/// Returns `Some(highlighted_string)` on success, `None` if no syntax
/// definition is found for `lang` or highlighting fails.
pub fn highlight(
    code: &str,
    lang: &str,
    syntax_map: &HashMap<String, String>,
    reset: &str,
    ss: &SyntaxSet,
) -> Option<String> {
    let syntax = find_syntax(ss, lang)?;
    let mut state = syntect::parsing::ParseState::new(syntax);
    let mut result = String::new();
    let lines: Vec<&str> = code.split('\n').collect();

    for (i, line) in lines.iter().enumerate() {
        if i > 0 {
            result.push('\n');
        }
        let ops = state.parse_line(line, ss).ok()?;
        let regions = scope_regions(line, &ops);

        for (text, scope_stack) in &regions {
            if text.is_empty() {
                continue;
            }
            let token_type = scope_to_token_type(scope_stack);
            match lookup_style(syntax_map, &token_type) {
                Some(ansi) if !ansi.is_empty() => {
                    result.push_str(ansi);
                    result.push_str(text);
                    result.push_str(reset);
                }
                _ => {
                    result.push_str(text);
                }
            }
        }
    }

    Some(result)
}

/// Extract (text, scope_stack) regions from syntect parse operations.
fn scope_regions<'a>(
    line: &'a str,
    ops: &[(usize, syntect::parsing::ScopeStackOp)],
) -> Vec<(&'a str, Vec<syntect::parsing::Scope>)> {
    use syntect::parsing::ScopeStackOp;

    let mut regions = Vec::new();
    let mut stack = Vec::new();
    let mut pos = 0;

    for &(offset, ref op) in ops {
        if offset > pos {
            let text = &line[pos..offset];
            regions.push((text, stack.clone()));
            pos = offset;
        }
        match op {
            ScopeStackOp::Push(scope) => {
                stack.push(*scope);
            }
            ScopeStackOp::Pop(count) => {
                for _ in 0..*count {
                    stack.pop();
                }
            }
            ScopeStackOp::Restore => {
                stack.clear();
            }
            _ => {}
        }
    }

    if pos < line.len() {
        regions.push((&line[pos..], stack));
    }

    regions
}

/// Convert a syntect scope stack to our underscore-separated token type.
///
/// Syntect scopes look like `keyword.control.elixir` or `string.quoted.double`.
/// We map them to our flat token types: `keyword_control`, `string_quoted`, etc.
fn scope_to_token_type(scopes: &[syntect::parsing::Scope]) -> String {
    // Use the most specific (last) scope, skipping `source.*` root scopes
    let scope = scopes
        .iter()
        .rev()
        .find(|s| {
            let s_str = s.build_string();
            !s_str.starts_with("source.") && !s_str.starts_with("meta.")
        })
        .or_else(|| scopes.last());

    match scope {
        Some(s) => {
            let s_str = s.build_string();
            // Take only the first two meaningful segments for our token type
            let parts: Vec<&str> = s_str.split('.').collect();
            match parts.len() {
                0 => String::new(),
                1 => parts[0].to_string(),
                _ => format!("{}_{}", parts[0], parts[1]),
            }
        }
        None => String::new(),
    }
}

/// Walk up the token type hierarchy:
///   `keyword_constant` -> `keyword` -> None
///   `name_builtin_pseudo` -> `name_builtin` -> `name` -> None
fn lookup_style<'a>(syntax_map: &'a HashMap<String, String>, token_type: &str) -> Option<&'a String> {
    if let Some(style) = syntax_map.get(token_type) {
        return Some(style);
    }
    // Walk up: strip last underscore segment
    if let Some(pos) = token_type.rfind('_') {
        lookup_style(syntax_map, &token_type[..pos])
    } else {
        None
    }
}

/// Find a syntax definition by language name, case-insensitive.
fn find_syntax<'a>(ss: &'a SyntaxSet, lang: &str) -> Option<&'a SyntaxReference> {
    ss.find_syntax_by_token(lang)
        .or_else(|| ss.find_syntax_by_name(lang))
        .or_else(|| {
            // Try case-insensitive match
            let lower = lang.to_lowercase();
            ss.syntaxes()
                .iter()
                .find(|s| s.name.to_lowercase() == lower)
        })
}
