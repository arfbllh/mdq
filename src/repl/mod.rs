//! REPL (Read-Eval-Print Loop) mode for mdq.
//!
//! This module provides an interactive interface for querying and manipulating
//! Markdown documents without repeatedly invoking the command line.

mod commands;
mod engine;
mod input;
mod session;
mod state;

pub use engine::ReplEngine;
pub use session::ReplSession;
pub use state::ReplState;

use crate::run::{Error, RunOptions};
use std::io;

/// Main REPL engine that manages the interactive session
pub struct Repl {
    engine: ReplEngine,
    session: ReplSession,
}

impl Repl {
    /// Creates a new REPL instance with the given options
    pub fn new(options: RunOptions) -> io::Result<Self> {
        let engine = ReplEngine::new(options)?;
        let session = ReplSession::new();
        
        Ok(Self { engine, session })
    }

    /// Starts the REPL session
    pub fn run(&mut self) -> io::Result<()> {
        self.engine.run(&mut self.session)
    }

    /// Loads a document into the REPL session
    pub fn load_document(&mut self, content: String) -> Result<(), Error> {
        self.session.load_document(content)
    }

    /// Gets the current session state
    pub fn session(&self) -> &ReplSession {
        &self.session
    }

    /// Gets a mutable reference to the session
    pub fn session_mut(&mut self) -> &mut ReplSession {
        &mut self.session
    }
}
