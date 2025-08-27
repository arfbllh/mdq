//! This crate is the library behind the [mdq] CLI tool.
//!
//! <div class="warning">
//!
//! **This is a preview API**. While I'll try to keep it as stable as possible, some breaking changes may occur.
//!
//! I will note any such changes in the [release notes on GitHub]. You can also find them searching the
//! [`breaking change` label] in the project's issue tracker.
//!
//! [release notes on GitHub]: https://github.com/yshavit/mdq/releases
//! [`breaking change` label]: https://github.com/yshavit/mdq/issues?q=label%3A%22breaking%20change%22
//!
//! </div>
//!
//! The general flow to use this crate is:
//!
//! 1. Parse Markdown into [`md_elem::MdElem`]s via [`md_elem::MdDoc::parse`]
//! 2. Parse a query via [`select::Selector`'s `TryFrom::<&str>`][selector-parse]
//! 3. Use [`select::Selector::find_nodes`] to filter the `MdElem`s down
//! 4. Use [`output`] to write the results
//!
//! The [`run`] module implements this workflow using options similar to the CLI's flags and a facade for I/O. You can
//! also do it yourself. See that module's documentation for an example.
//!
//! ## Example: End-to-end parsing and selection
//!
//! To parse some Markdown and a query string and output the result as Markdown to stdout:
//!
//! ```
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use indoc::indoc;
//!
//! // Define some markdown
//! let markdown_text = indoc! {r##"
//! ## First section
//!
//! - hello
//! - world
//!
//! ## Second section
//!
//! - foo
//! - bar
//! "##};
//! let parsed_md = mdq::md_elem::MdDoc::parse(markdown_text, &mdq::md_elem::ParseOptions::default())?;
//!
//! // Parse a selector that looks for a section with title containing "second", and
//! // then looks for list items within it
//! let query_text = "# second | - *";
//! let selector: mdq::select::Selector = query_text.try_into()?;
//!
//! // Run the selector against the parsed Markdown
//! let (found_nodes, found_nodes_ctx) = selector.find_nodes(parsed_md)?;
//!
//! // Output. Note our use of
//! let mut output_string = String::new();
//! let writer = mdq::output::MdWriter::default();
//! writer.write(&found_nodes_ctx, &found_nodes, &mut output_string);
//!
//! assert_eq!(
//!     output_string,
//!     indoc! {r"
//!     - foo
//!
//!     - bar
//! "});
//! #
//! #     Ok(())
//! # }
//! ```
//!
//! [mdq]: https://github.com/yshavit/mdq
//! [selector-parse]: select::Selector#impl-TryFrom<%26str>-for-Selector

pub mod md_elem;
pub mod output;
mod query;
pub mod run;
pub mod select;
mod util;

#[cfg(test)]
mod tests {

    use crate::run::{RunOptions, OsFacade};
    use std::io;

    /// Mock OS facade for testing that captures error output
    struct MockOsFacade {
        pub error_output: String,
        pub stdout_output: Vec<u8>,
    }

    impl MockOsFacade {
        fn new() -> Self {
            Self {
                error_output: String::new(),
                stdout_output: Vec::new(),
            }
        }

        fn get_error_output(&self) -> &str {
            &self.error_output
        }


    }

    impl OsFacade for MockOsFacade {
        fn read_stdin(&self) -> io::Result<String> {
            Ok("# Test Document\n\nThis is a test document.\n".to_string())
        }

        fn read_file(&self, _path: &str) -> io::Result<String> {
            Ok("# Test Document\n\nThis is a test document.\n".to_string())
        }

        fn stdout(&mut self) -> impl std::io::Write {
            &mut self.stdout_output
        }

        fn write_error(&mut self, err: crate::run::Error) {
            self.error_output = err.to_string();
        }
    }

    /// Test helper function to run mdq with specific options and capture output
    fn run_mdq_with_options(selectors: &str, enhanced_errors: bool) -> MockOsFacade {
        let mut os = MockOsFacade::new();
        
        let options = RunOptions {
            selectors: selectors.to_string(),
            enhanced_errors,
            ..Default::default()
        };
        
        let _ = crate::run::run(&options, &mut os);
        os
    }

    // Individual test functions for each error case
    #[test]
    fn test_error_exclamation_mark() {
        let os = run_mdq_with_options("!invalid", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_invalid_hash() {
        let os = run_mdq_with_options("invalid#", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_invalid_dash() {
        let os = run_mdq_with_options("invalid-", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use - for list items"));
    }

    #[test]
    fn test_error_invalid_brackets() {
        let os = run_mdq_with_options("invalid[]", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use [] for links"));
    }

    #[test]
    fn test_error_invalid_greater_than() {
        let os = run_mdq_with_options("invalid>", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use > for blockquotes"));
    }

    #[test]
    fn test_error_invalid_code_block() {
        let os = run_mdq_with_options("invalid```", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use ``` for code blocks"));
    }

    #[test]
    fn test_error_invalid_front_matter() {
        let os = run_mdq_with_options("invalid+++", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use +++ for front matter"));
    }

    #[test]
    fn test_error_invalid_html() {
        let os = run_mdq_with_options("invalid</>", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use </> for HTML"));
    }

    #[test]
    fn test_error_invalid_paragraph() {
        let os = run_mdq_with_options("invalidP:", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use P: for paragraphs"));
    }

    #[test]
    fn test_error_invalid_table() {
        let os = run_mdq_with_options("invalid:-:", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use :-: for tables"));
    }

    #[test]
    fn test_error_invalid_pipe() {
        let os = run_mdq_with_options("invalid|", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use | to separate multiple selectors"));
    }

    #[test]
    fn test_error_invalid_asterisk() {
        let os = run_mdq_with_options("invalid*", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_invalid_caret() {
        let os = run_mdq_with_options("invalid^", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_invalid_dollar() {
        let os = run_mdq_with_options("invalid$", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_invalid_question() {
        let os = run_mdq_with_options("invalid?", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_invalid_plus() {
        let os = run_mdq_with_options("invalid+", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_at_symbol() {
        let os = run_mdq_with_options("@invalid", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_ampersand() {
        let os = run_mdq_with_options("&invalid", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_percent() {
        let os = run_mdq_with_options("%invalid", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_equals() {
        let os = run_mdq_with_options("=invalid", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_tilde() {
        let os = run_mdq_with_options("~invalid", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_numbers_prefix() {
        let os = run_mdq_with_options("123invalid", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_numbers_suffix() {
        let os = run_mdq_with_options("invalid123", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_mixed_hash() {
        let os = run_mdq_with_options("abc#def", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_mixed_dash() {
        let os = run_mdq_with_options("abc-123", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use - for list items"));
    }

    #[test]
    fn test_error_mixed_brackets() {
        let os = run_mdq_with_options("abc[123]", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use [] for links"));
    }

    #[test]
    fn test_error_mixed_greater_than() {
        let os = run_mdq_with_options("abc>def", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use > for blockquotes"));
    }

    #[test]
    fn test_error_mixed_code_block() {
        let os = run_mdq_with_options("abc```def", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use ``` for code blocks"));
    }

    #[test]
    fn test_error_mixed_front_matter() {
        let os = run_mdq_with_options("abc+++def", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use +++ for front matter"));
    }

    #[test]
    fn test_error_mixed_html() {
        let os = run_mdq_with_options("abc</>def", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use </> for HTML"));
    }

    #[test]
    fn test_error_exclamation_mixed() {
        let os = run_mdq_with_options("xyz!abc", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_at_mixed() {
        let os = run_mdq_with_options("abc@xyz", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_hash_mixed() {
        let os = run_mdq_with_options("xyz#abc", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_dollar_mixed() {
        let os = run_mdq_with_options("abc$xyz", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_percent_mixed() {
        let os = run_mdq_with_options("xyz%abc", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_caret_mixed() {
        let os = run_mdq_with_options("abc^xyz", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_ampersand_mixed() {
        let os = run_mdq_with_options("xyz&abc", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_asterisk_mixed() {
        let os = run_mdq_with_options("abc*xyz", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_parentheses() {
        let os = run_mdq_with_options("xyz(abc", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_parentheses_close() {
        let os = run_mdq_with_options("abc)xyz", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_brackets_open() {
        let os = run_mdq_with_options("xyz[abc", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use [] for links"));
    }

    #[test]
    fn test_error_brackets_close() {
        let os = run_mdq_with_options("abc]xyz", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use [] for links"));
    }

    #[test]
    fn test_error_braces_open() {
        let os = run_mdq_with_options("xyz{abc", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_braces_close() {
        let os = run_mdq_with_options("abc}xyz", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_backslash() {
        let os = run_mdq_with_options("xyz\\abc", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_forward_slash() {
        let os = run_mdq_with_options("abc/xyz", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_pipe_mixed() {
        let os = run_mdq_with_options("xyz|abc", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use | to separate multiple selectors"));
    }

    #[test]
    fn test_error_tilde_mixed() {
        let os = run_mdq_with_options("abc~xyz", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_backtick() {
        let os = run_mdq_with_options("xyz`abc", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_single_quote() {
        let os = run_mdq_with_options("abc'xyz", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_double_quote() {
        let os = run_mdq_with_options("xyz\"abc", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_semicolon() {
        let os = run_mdq_with_options("abc;xyz", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_colon() {
        let os = run_mdq_with_options("xyz:abc", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_comma() {
        let os = run_mdq_with_options("abc,xyz", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_period() {
        let os = run_mdq_with_options("xyz.abc", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_less_than() {
        let os = run_mdq_with_options("abc<xyz", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_greater_than_mixed() {
        let os = run_mdq_with_options("xyz>abc", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use > for blockquotes"));
    }

    #[test]
    fn test_error_equals_mixed() {
        let os = run_mdq_with_options("abc=xyz", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_plus_mixed() {
        let os = run_mdq_with_options("xyz+abc", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_dash_mixed() {
        let os = run_mdq_with_options("abc-xyz", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use - for list items"));
    }

    #[test]
    fn test_error_underscore() {
        let os = run_mdq_with_options("xyz_abc", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }

    #[test]
    fn test_error_hash_mixed_final() {
        let os = run_mdq_with_options("abc#xyz", true);
        let error_output = os.get_error_output();
        assert!(error_output.contains("Suggestions:"));
        assert!(error_output.contains("Use # for sections"));
    }
}
