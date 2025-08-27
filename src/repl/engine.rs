use crate::repl::{ReplSession, ReplState};
use crate::repl::commands::{ReplCommand, execute_command};
use crate::repl::input::ReplInput;
use crate::run::RunOptions;
use crate::output::MdWriterOptions;
use std::io::{self, Write};

/// The main REPL engine that coordinates the interactive session
pub struct ReplEngine {
    /// Input handler for reading commands
    input: ReplInput,
    
    /// Current run options
    options: RunOptions,
}

impl ReplEngine {
    /// Creates a new REPL engine
    pub fn new(options: RunOptions) -> io::Result<Self> {
        Ok(Self {
            input: ReplInput::default(),
            options,
        })
    }

    /// Runs the main REPL loop
    pub fn run(&mut self, session: &mut ReplSession) -> io::Result<()> {
        let mut state = ReplState::new(self.options.clone());
        let mut variables = std::collections::HashMap::new();
        
        // Show welcome message
        self.show_welcome()?;
        
        // Main REPL loop
        loop {
            // Read command
            let input = match self.input.read_line("mdq> ") {
                Ok(Some(input)) => input,
                Ok(None) => break, // EOF (Ctrl+D)
                Err(e) => {
                    writeln!(io::stderr(), "Error reading input: {}", e)?;
                    continue;
                }
            };
            
            // Parse and execute command
            let command = ReplCommand::parse(&input);
            let should_continue = self.execute_command(&command, session, &mut state, &mut variables)?;
            
            if !should_continue {
                break;
            }
        }
        
        writeln!(io::stdout(), "Goodbye!")?;
        Ok(())
    }

    /// Executes a REPL command
    fn execute_command(
        &self,
        command: &ReplCommand,
        session: &mut ReplSession,
        state: &mut ReplState,
        variables: &mut std::collections::HashMap<String, String>,
    ) -> io::Result<bool> {
        let mut output = io::stdout();
        
        match command {
            ReplCommand::Query(_) => {
                // Execute query against current document
                let document = state.document();
                let mut options = self.build_writer_options(state);
                
                let should_continue = execute_command(
                    command,
                    document,
                    &mut options,
                    variables,
                    &mut output,
                )?;
                
                // Update state with new options - we don't need to update output format here
                // since it's handled by the state management
                
                Ok(should_continue)
            }
            ReplCommand::Load(path) => {
                // Load document from file
                match session.load_document_from_file(path.clone()) {
                    Ok(()) => {
                        // Parse the document
                        match session.parse_document(self.options.allow_unknown_markdown) {
                            Ok(doc) => {
                                state.set_document(doc);
                                writeln!(output, "Document loaded successfully: {}", path)?;
                                writeln!(output, "{}", session.document_info())?;
                            }
                            Err(e) => {
                                writeln!(output, "Error parsing document: {}", e)?;
                            }
                        }
                    }
                    Err(e) => {
                        writeln!(output, "Error loading document: {}", e)?;
                    }
                }
                Ok(true)
            }
            ReplCommand::Reload => {
                // Reload current document
                match session.reload() {
                    Ok(()) => {
                        match session.parse_document(self.options.allow_unknown_markdown) {
                            Ok(doc) => {
                                state.set_document(doc);
                                writeln!(output, "Document reloaded successfully")?;
                                writeln!(output, "{}", session.document_info())?;
                            }
                            Err(e) => {
                                writeln!(output, "Error parsing reloaded document: {}", e)?;
                            }
                        }
                    }
                    Err(e) => {
                        writeln!(output, "Error reloading document: {}", e)?;
                    }
                }
                Ok(true)
            }
            ReplCommand::Format(format) => {
                // Change output format
                state.set_output_format(*format);
                writeln!(output, "Output format set to: {:?}", format)?;
                Ok(true)
            }
            ReplCommand::Clear => {
                // Clear current document
                session.clear_document();
                state.clear_document();
                writeln!(output, "Document cleared")?;
                Ok(true)
            }
            ReplCommand::Exit => {
                // Exit REPL
                Ok(false)
            }
            _ => {
                // Handle other commands
                let document = state.document();
                let mut options = self.build_writer_options(state);
                
                let should_continue = execute_command(
                    command,
                    document,
                    &mut options,
                    variables,
                    &mut output,
                )?;
                
                Ok(should_continue)
            }
        }
    }

    /// Builds writer options from current state
    fn build_writer_options(&self, state: &ReplState) -> MdWriterOptions {
        let run_options = state.options();
        
        MdWriterOptions {
            link_reference_placement: run_options.link_pos,
            footnote_reference_placement: run_options.footnote_pos.unwrap_or(run_options.link_pos),
            inline_options: crate::output::InlineElemOptions {
                link_format: run_options.link_format,
                renumber_footnotes: run_options.renumber_footnotes,
            },
            include_thematic_breaks: run_options.add_breaks.unwrap_or(true),
            text_width: run_options.wrap_width,
        }
    }

    /// Shows the welcome message
    fn show_welcome(&self) -> io::Result<()> {
        let mut output = io::stdout();
        writeln!(output, "mdq REPL - Interactive Markdown Query Tool")?;
        writeln!(output, "Type '.help' for available commands, or enter a selector query")?;
        writeln!(output, "Press Ctrl+D or type '.exit' to quit")?;
        writeln!(output)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run::OutputFormat;

    #[test]
    fn test_command_parsing() {
        let command = ReplCommand::parse("# Section");
        assert!(matches!(command, ReplCommand::Query(_)));
        
        let command = ReplCommand::parse(".help");
        assert!(matches!(command, ReplCommand::Help));
        
        let command = ReplCommand::parse(".load test.md");
        assert!(matches!(command, ReplCommand::Load(_)));
        
        let command = ReplCommand::parse(".format json");
        assert!(matches!(command, ReplCommand::Format(OutputFormat::Json)));
    }
}
