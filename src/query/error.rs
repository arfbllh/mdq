use pest::Span;
use std::fmt::{Display, Formatter};

/// Converts a pest Rule to a human-readable string.
fn rule_to_string(rule: &crate::query::pest::Rule) -> &'static str {
    match rule {
        crate::query::pest::Rule::EOI => "end of input",
        crate::query::pest::Rule::WHITESPACE => "whitespace",
        crate::query::pest::Rule::top => "valid query",
        crate::query::pest::Rule::selector_chain => "one or more selectors",
        crate::query::pest::Rule::selector => "selector",
        crate::query::pest::Rule::selector_delim | crate::query::pest::Rule::explicit_space => "space",
        crate::query::pest::Rule::select_section | crate::query::pest::Rule::section_start => "#",
        crate::query::pest::Rule::select_list_item | crate::query::pest::Rule::list_start => "- or 1.",
        crate::query::pest::Rule::list_ordered => "-",
        crate::query::pest::Rule::list_task_options => "[ ], [x], or [?]",
        crate::query::pest::Rule::task_checked => "[x]",
        crate::query::pest::Rule::task_unchecked => "[ ]",
        crate::query::pest::Rule::task_either => "[?]",
        crate::query::pest::Rule::task_end => "]",
        crate::query::pest::Rule::select_link | crate::query::pest::Rule::link_start => "[ or ![",
        crate::query::pest::Rule::image_start => "![",
        crate::query::pest::Rule::select_block_quote | crate::query::pest::Rule::select_block_quote_start => ">",
        crate::query::pest::Rule::select_code_block | crate::query::pest::Rule::code_block_start => "```",
        crate::query::pest::Rule::select_front_matter | crate::query::pest::Rule::front_matter_start => "+++",
        crate::query::pest::Rule::select_html | crate::query::pest::Rule::html_start => "</>",
        crate::query::pest::Rule::select_paragraph | crate::query::pest::Rule::select_paragraph_start => "P:",
        crate::query::pest::Rule::select_table | crate::query::pest::Rule::table_start => ":-:",
        crate::query::pest::Rule::string
        | crate::query::pest::Rule::string_for_unit_tests__do_not_use_angle
        | crate::query::pest::Rule::string_for_unit_tests__do_not_use_pipe => "string",
        crate::query::pest::Rule::unquoted_string => "unquoted string",
        crate::query::pest::Rule::regex => "regex",
        crate::query::pest::Rule::regex_char => "regex character",
        crate::query::pest::Rule::regex_escaped_slash => "/",
        crate::query::pest::Rule::regex_normal_char => "regex character",
        crate::query::pest::Rule::regex_replacement_segment => "regex replacement",
        crate::query::pest::Rule::quoted_string => "quoted string",
        crate::query::pest::Rule::quoted_char => "character in quoted string",
        crate::query::pest::Rule::asterisk => "*",
        crate::query::pest::Rule::anchor_start => "^",
        crate::query::pest::Rule::anchor_end => "$",
        crate::query::pest::Rule::quoted_plain_chars => "character in quoted string",
        crate::query::pest::Rule::escaped_char => "escape sequence",
        crate::query::pest::Rule::unicode_seq => "unicode sequence",
    }
}

/// An error representing an invalid selector query.
///
/// <div class="warning">
/// This struct's <code>source()</code> is not part of the public contract, and may change at any time without that change being
/// marked as a breaking change.
/// </div>
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ParseError {
    pub(crate) inner: InnerParseError,
}

impl ParseError {
    /// Creates a new ParseError from an [InnerParseError].
    ///
    /// This is intentionally not a [From] impl, because we want to keep it `pub(crate)`.
    pub(crate) fn new(inner: InnerParseError) -> Self {
        Self { inner }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl std::error::Error for ParseError {
    /// This method gets the error's source, if available. **Not part of the public API contract.**
    ///
    /// Please see the warning on [this struct's main documentation](ParseError).
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum InnerParseError {
    Pest(crate::query::Error),
    Other(DetachedSpan, String),
}

impl Display for InnerParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            InnerParseError::Pest(error) => Display::fmt(error, f),
            InnerParseError::Other(_, message) => Display::fmt(message, f),
        }
    }
}

impl std::error::Error for InnerParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            InnerParseError::Pest(err) => Some(err),
            InnerParseError::Other(_, _) => None,
        }
    }
}

impl ParseError {
    /// Gets a string suitable for displaying to a user, given the original query string.
    ///
    /// ```
    /// use mdq::select::Selector;
    /// let query_text = "$ ! invalid query string ! $";
    /// let parse_error = Selector::try_from(query_text).expect_err("expected an error");
    /// let expected_error = r" --> 1:1
    ///   |
    /// 1 | $ ! invalid query string ! $
    ///   | ^---
    ///   |
    ///   = expected valid query";
    /// assert_eq!(parse_error.to_string(query_text), expected_error);
    /// ```
    pub fn to_string(&self, query_text: &str) -> String {
        match &self.inner {
            InnerParseError::Pest(e) => format!("{e}"),
            InnerParseError::Other(span, message) => match Span::new(query_text, span.start, span.end) {
                None => message.to_string(),
                Some(span) => {
                    let pest_err = crate::query::Error::new_from_span(span, message.to_string());
                    pest_err.to_string()
                }
            },
        }
    }

    /// Gets a string suitable for displaying to a user, including suggestions for the query.
    ///
    /// This is useful for providing context when the error is related to a specific query.
    ///
    /// ```
    /// use mdq::select::Selector;
    /// let query_text = "$ ! invalid query string ! $";
    /// let parse_error = Selector::try_from(query_text).expect_err("expected an error");
    /// let output = parse_error.to_string_with_suggestions(query_text);
    /// assert!(output.contains("expected valid query"));
    /// assert!(output.contains("Suggestions:"));
    /// assert!(output.contains("Use # for sections"));
    /// ```
    pub fn to_string_with_suggestions(&self, query_text: &str) -> String {
        match &self.inner {
            InnerParseError::Pest(e) => {
                let rule = extract_failed_rule_from_pest_error(e);
                let mut error_string = format!("{e}");
                if let Some(rule_name) = rule {
                    error_string.push_str(&format!("\n\nExpected: `{}`", rule_name));
                } else {
                    // For custom errors, provide general suggestions
                    error_string.push_str("\n\nSuggestions:");
                    error_string.push_str("\n  • Use # for sections (e.g., '# My Section')");
                    error_string.push_str("\n  • Use - for list items (e.g., '- List item')");
                    error_string.push_str("\n  • Use [] for links (e.g., '[text](url)')");
                    error_string.push_str("\n  • Use > for blockquotes (e.g., '> Quote text')");
                    error_string.push_str("\n  • Use ``` for code blocks (e.g., '```rust code')");
                    error_string.push_str("\n  • Use +++ for front matter (e.g., '+++ toml')");
                    error_string.push_str("\n  • Use </> for HTML (e.g., '</> <div>')");
                    error_string.push_str("\n  • Use P: for paragraphs (e.g., 'P: paragraph text')");
                    error_string.push_str("\n  • Use :-: for tables (e.g., ':-: column | row')");
                    error_string.push_str("\n  • Use | to separate multiple selectors (e.g., '# Section | - List item')");
                }
                error_string
            }
            InnerParseError::Other(span, message) => match Span::new(query_text, span.start, span.end) {
                None => message.to_string(),
                Some(span) => {
                    let pest_err = crate::query::Error::new_from_span(span, message.to_string());
                    let rule = extract_failed_rule_from_pest_error(&pest_err);
                    let mut error_string = pest_err.to_string();
                    if let Some(rule_name) = rule {
                        error_string.push_str(&format!("\n\nExpected: `{}`", rule_name));
                    } else {
                        error_string.push_str("\n\n[No rule extracted]");
                    }
                    error_string
                }
            },
        }
    }
}

impl From<crate::query::Error> for InnerParseError {
    fn from(err: crate::query::Error) -> Self {
        Self::Pest(err)
    }
}

/// Like a [pest::Span], but without a reference to the underlying `&str`, and thus cheaply Copyable.
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, Hash)]
pub(crate) struct DetachedSpan {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl From<pest::Span<'_>> for DetachedSpan {
    fn from(value: pest::Span) -> Self {
        Self {
            start: value.start(),
            end: value.end(),
        }
    }
}

impl From<&crate::query::Pair<'_>> for DetachedSpan {
    fn from(value: &crate::query::Pair<'_>) -> Self {
        value.as_span().into()
    }
}

/// Extracts the failed rule name from a pest error for better error reporting.
fn extract_failed_rule_from_pest_error(error: &crate::query::Error) -> Option<&str> {
    // Access the inner pest error to extract rule information
    let pest_error = &error.pest_error;
    
    // Try to extract the expected rule from the error variant
    match &pest_error.variant {
        pest::error::ErrorVariant::ParsingError { positives, negatives: _ } => {
            // Return the first positive rule that was expected
            positives.first().map(|rule| rule_to_string(rule))
        }
        pest::error::ErrorVariant::CustomError { .. } => {
            // For custom errors, we can't easily determine the rule
            None
        }
    }
}
