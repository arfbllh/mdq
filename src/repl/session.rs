use crate::md_elem::{MdDoc, ParseOptions, InvalidMd};
use crate::run::Error;

/// Manages the REPL session including document loading and parsing
#[derive(Debug)]
pub struct ReplSession {
    /// Current document content as string
    content: Option<String>,
    
    /// Current document path (if loaded from file)
    path: Option<String>,
}

impl ReplSession {
    /// Creates a new REPL session
    pub fn new() -> Self {
        Self {
            content: None,
            path: None,
        }
    }

    /// Loads a document from string content
    pub fn load_document(&mut self, content: String) -> Result<(), Error> {
        self.content = Some(content);
        self.path = None;
        Ok(())
    }

    /// Loads a document from a file path
    pub fn load_document_from_file(&mut self, path: String) -> Result<(), Error> {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| Error::FileReadError(crate::run::Input::FilePath(path.clone()), e))?;
        
        self.content = Some(content);
        self.path = Some(path);
        Ok(())
    }

    /// Gets the current document content
    pub fn content(&self) -> Option<&String> {
        self.content.as_ref()
    }

    /// Gets the current document path
    pub fn path(&self) -> Option<&String> {
        self.path.as_ref()
    }

    /// Parses the current document content
    pub fn parse_document(&self, allow_unknown_markdown: bool) -> Result<MdDoc, InvalidMd> {
        let content = self.content.as_ref()
            .ok_or_else(|| InvalidMd::ParseError("No document loaded".to_string()))?;
        
        let options = ParseOptions {
            allow_unknown_markdown,
            ..ParseOptions::default()
        };
        
        MdDoc::parse(content, &options)
    }

    /// Reloads the current document from file (if it was loaded from a file)
    pub fn reload(&mut self) -> Result<(), Error> {
        if let Some(path) = &self.path {
            self.load_document_from_file(path.clone())
        } else {
            Err(Error::Other("No file path available for reloading".to_string()))
        }
    }

    /// Clears the current document
    pub fn clear_document(&mut self) {
        self.content = None;
        self.path = None;
    }

    /// Checks if a document is loaded
    pub fn has_document(&self) -> bool {
        self.content.is_some()
    }

    /// Gets document info for display
    pub fn document_info(&self) -> String {
        match (&self.content, &self.path) {
            (Some(content), Some(path)) => {
                format!("Document: {} ({} bytes)", path, content.len())
            }
            (Some(content), None) => {
                format!("Document: stdin ({} bytes)", content.len())
            }
            (None, _) => "No document loaded".to_string(),
        }
    }
}

impl Default for ReplSession {
    fn default() -> Self {
        Self::new()
    }
}
