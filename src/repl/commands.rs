use crate::md_elem::MdDoc;
use crate::run::OutputFormat;
use crate::select::Selector;
use crate::output::MdWriterOptions;
use std::io::{self, Write};

/// Built-in REPL commands
#[derive(Debug, Clone, PartialEq)]
pub enum ReplCommand {
    /// Execute a selector query
    Query(String),
    
    /// Load a document from file
    Load(String),
    
    /// Reload current document
    Reload,
    
    /// Change output format
    Format(OutputFormat),
    
    /// Set a variable
    Set(String, String),
    
    /// Get a variable
    Get(String),
    
    /// List all variables
    Variables,
    
    /// Show help
    Help,
    
    /// Show document info
    Info,
    
    /// Clear document
    Clear,
    
    /// Exit REPL
    Exit,
    
    /// Unknown command
    Unknown(String),
}

impl ReplCommand {
    /// Parses a command string into a ReplCommand
    pub fn parse(input: &str) -> Self {
        let input = input.trim();
        
        if input.is_empty() {
            return ReplCommand::Unknown(input.to_string());
        }
        
        // Check for built-in commands
        if let Some(stripped) = input.strip_prefix('.') {
            let parts: Vec<&str> = stripped.split_whitespace().collect();
            if parts.is_empty() {
                return ReplCommand::Unknown(input.to_string());
            }
            
            match parts[0] {
                "load" => {
                    if parts.len() == 2 {
                        ReplCommand::Load(parts[1].to_string())
                    } else {
                        ReplCommand::Unknown(input.to_string())
                    }
                }
                "reload" => ReplCommand::Reload,
                "format" => {
                    if parts.len() == 2 {
                        match parts[1] {
                            "md" | "markdown" => ReplCommand::Format(OutputFormat::Markdown),
                            "json" => ReplCommand::Format(OutputFormat::Json),
                            "plain" => ReplCommand::Format(OutputFormat::Plain),
                            _ => ReplCommand::Unknown(input.to_string()),
                        }
                    } else {
                        ReplCommand::Unknown(input.to_string())
                    }
                }
                "set" => {
                    if parts.len() >= 3 {
                        let name = parts[1].to_string();
                        let value = parts[2..].join(" ");
                        ReplCommand::Set(name, value)
                    } else {
                        ReplCommand::Unknown(input.to_string())
                    }
                }
                "get" => {
                    if parts.len() == 2 {
                        ReplCommand::Get(parts[1].to_string())
                    } else {
                        ReplCommand::Unknown(input.to_string())
                    }
                }
                "vars" | "variables" => ReplCommand::Variables,
                "help" => ReplCommand::Help,
                "info" => ReplCommand::Info,
                "clear" => ReplCommand::Clear,
                "exit" | "quit" => ReplCommand::Exit,
                _ => ReplCommand::Unknown(input.to_string()),
            }
        } else {
            // Treat as a query
            ReplCommand::Query(input.to_string())
        }
    }
}

/// Executes a REPL command
pub fn execute_command<W: Write>(
    command: &ReplCommand,
    document: Option<&MdDoc>,
    _options: &mut MdWriterOptions,
    variables: &mut std::collections::HashMap<String, String>,
    output: &mut W,
) -> io::Result<bool> {
    match command {
        ReplCommand::Query(selector_str) => {
            execute_query(selector_str, document, _options, output)
        }
        ReplCommand::Load(path) => {
            writeln!(output, "Loading document from: {}", path)?;
            Ok(true) // Signal that document should be loaded
        }
        ReplCommand::Reload => {
            writeln!(output, "Reloading document...")?;
            Ok(true) // Signal that document should be reloaded
        }
        ReplCommand::Format(format) => {
            writeln!(output, "Setting output format to: {:?}", format)?;
            Ok(false)
        }
        ReplCommand::Set(name, value) => {
            variables.insert(name.clone(), value.clone());
            writeln!(output, "Set variable '{}' = '{}'", name, value)?;
            Ok(false)
        }
        ReplCommand::Get(name) => {
            if let Some(value) = variables.get(name) {
                writeln!(output, "{} = {}", name, value)?;
            } else {
                writeln!(output, "Variable '{}' not found", name)?;
            }
            Ok(false)
        }
        ReplCommand::Variables => {
            if variables.is_empty() {
                writeln!(output, "No variables set")?;
            } else {
                writeln!(output, "Variables:")?;
                for (name, value) in variables {
                    writeln!(output, "  {} = {}", name, value)?;
                }
            }
            Ok(false)
        }
        ReplCommand::Help => {
            show_help(output)?;
            Ok(false)
        }
        ReplCommand::Info => {
            if let Some(doc) = document {
                writeln!(output, "Document loaded with {} root elements", doc.roots.len())?;
            } else {
                writeln!(output, "No document loaded")?;
            }
            Ok(false)
        }
        ReplCommand::Clear => {
            writeln!(output, "Document cleared")?;
            Ok(false)
        }
        ReplCommand::Exit => {
            writeln!(output, "Exiting REPL...")?;
            Ok(false)
        }
        ReplCommand::Unknown(cmd) => {
            writeln!(output, "Unknown command: {}", cmd)?;
            writeln!(output, "Use .help for available commands")?;
            Ok(false)
        }
    }
}

/// Executes a selector query
fn execute_query<W: Write>(
    selector_str: &str,
    document: Option<&MdDoc>,
    _options: &MdWriterOptions,
    output: &mut W,
) -> io::Result<bool> {
    if document.is_none() {
        writeln!(output, "Error: No document loaded. Use .load <file> first.")?;
        return Ok(false);
    }
    
    let doc = document.unwrap();
    
    // Parse the selector
    let selector = match Selector::try_parse(selector_str) {
        Ok(s) => s,
        Err(e) => {
            writeln!(output, "Error parsing selector: {}", e)?;
            return Ok(false);
        }
    };
    
    // Execute the selector
    let (pipeline_nodes, _ctx) = match selector.find_nodes(doc.clone()) {
        Ok(result) => result,
        Err(e) => {
            writeln!(output, "Error executing selector: {}", e)?;
            return Ok(false);
        }
    };
    
    if pipeline_nodes.is_empty() {
        writeln!(output, "No elements matched the selector")?;
        return Ok(false);
    }
    
    // For now, just show the count of matching elements
    // TODO: Implement proper output formatting based on the current format setting
    writeln!(output, "Found {} matching elements", pipeline_nodes.len())?;
    writeln!(output, "Output formatting not yet implemented in REPL mode")?;
    
    Ok(false)
}

/// Shows help information
fn show_help<W: Write>(output: &mut W) -> io::Result<()> {
    writeln!(output, "mdq REPL - Interactive Markdown Query Tool")?;
    writeln!(output)?;
    writeln!(output, "Available commands:")?;
    writeln!(output, "  <selector>     Execute a selector query")?;
    writeln!(output, "  .load <file>   Load a document from file")?;
    writeln!(output, "  .reload        Reload the current document")?;
    writeln!(output, "  .format <fmt>  Change output format (md|json|plain)")?;
    writeln!(output, "  .set <n> <v>   Set a variable")?;
    writeln!(output, "  .get <n>       Get a variable value")?;
    writeln!(output, "  .vars          List all variables")?;
    writeln!(output, "  .info          Show document information")?;
    writeln!(output, "  .clear         Clear current document")?;
    writeln!(output, "  .help          Show this help")?;
    writeln!(output, "  .exit          Exit REPL")?;
    writeln!(output)?;
    writeln!(output, "Selector examples:")?;
    writeln!(output, "  # Section      - Select sections with title containing 'Section'")?;
    writeln!(output, "  - List item    - Select list items containing 'List item'")?;
    writeln!(output, "  [text](url)    - Select links with display text 'text'")?;
    writeln!(output, "  > Quote        - Select blockquotes containing 'Quote'")?;
    writeln!(output, "  ```rust        - Select code blocks with language 'rust'")?;
    Ok(())
}
