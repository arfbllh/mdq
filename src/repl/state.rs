use crate::md_elem::MdDoc;
use crate::run::{RunOptions, OutputFormat};
use std::collections::HashMap;

/// Represents the current state of a REPL session
#[derive(Debug)]
pub struct ReplState {
    /// Current document being worked with
    document: Option<MdDoc>,
    
    /// Current run options (output format, link placement, etc.)
    options: RunOptions,
    
    /// Variables stored during the session
    variables: HashMap<String, String>,
    
    /// Command history
    history: Vec<String>,
    
    /// Current output format
    current_format: OutputFormat,
}

impl ReplState {
    /// Creates a new REPL state with default options
    pub fn new(options: RunOptions) -> Self {
        let current_format = options.output;
        
        Self {
            document: None,
            options,
            variables: HashMap::new(),
            history: Vec::new(),
            current_format,
        }
    }

    /// Sets the current document
    pub fn set_document(&mut self, doc: MdDoc) {
        self.document = Some(doc);
    }

    /// Gets a reference to the current document
    pub fn document(&self) -> Option<&MdDoc> {
        self.document.as_ref()
    }

    /// Gets a mutable reference to the current document
    pub fn document_mut(&mut self) -> Option<&mut MdDoc> {
        self.document.as_mut()
    }

    /// Gets the current options
    pub fn options(&self) -> &RunOptions {
        &self.options
    }

    /// Gets a mutable reference to the options
    pub fn options_mut(&mut self) -> &mut RunOptions {
        &mut self.options
    }

    /// Sets a variable
    pub fn set_variable(&mut self, name: String, value: String) {
        self.variables.insert(name, value);
    }

    /// Gets a variable
    pub fn get_variable(&self, name: &str) -> Option<&String> {
        self.variables.get(name)
    }

    /// Adds a command to history
    pub fn add_to_history(&mut self, command: String) {
        self.history.push(command);
        // Keep only last 1000 commands
        if self.history.len() > 1000 {
            self.history.remove(0);
        }
    }

    /// Gets the command history
    pub fn history(&self) -> &[String] {
        &self.history
    }

    /// Sets the current output format
    pub fn set_output_format(&mut self, format: OutputFormat) {
        self.current_format = format;
        self.options.output = format;
    }

    /// Gets the current output format
    pub fn current_format(&self) -> OutputFormat {
        self.current_format
    }

    /// Clears all variables
    pub fn clear_variables(&mut self) {
        self.variables.clear();
    }

    /// Clears the document
    pub fn clear_document(&mut self) {
        self.document = None;
    }

    /// Checks if a document is loaded
    pub fn has_document(&self) -> bool {
        self.document.is_some()
    }
}
