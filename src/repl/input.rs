use std::io::{self, Write, BufRead, BufReader};
use std::collections::VecDeque;

/// Manages REPL input including history and line editing
pub struct ReplInput {
    /// Command history
    history: VecDeque<String>,
    
    /// Maximum history size
    max_history: usize,
    
    /// Current position in history (for navigation)
    history_pos: Option<usize>,
}

impl ReplInput {
    /// Creates a new REPL input handler
    pub fn new(max_history: usize) -> Self {
        Self {
            history: VecDeque::new(),
            max_history,
            history_pos: None,
        }
    }

    /// Reads a line of input from stdin
    pub fn read_line(&mut self, prompt: &str) -> io::Result<Option<String>> {
        let stdout = io::stdout();
        let mut stdout = stdout.lock();
        
        // Print prompt
        write!(stdout, "{}", prompt)?;
        stdout.flush()?;
        drop(stdout);
        
        // Read input
        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();
        
        match reader.read_line(&mut line) {
            Ok(0) => Ok(None), // EOF (Ctrl+D)
            Ok(_) => {
                let line = line.trim().to_string();
                if !line.is_empty() {
                    self.add_to_history(line.clone());
                }
                Ok(Some(line))
            }
            Err(e) => Err(e),
        }
    }

    /// Adds a command to history
    pub fn add_to_history(&mut self, command: String) {
        // Don't add empty commands or duplicates
        if command.is_empty() || self.history.back() == Some(&command) {
            return;
        }
        
        self.history.push_back(command);
        
        // Maintain history size limit
        if self.history.len() > self.max_history {
            self.history.pop_front();
        }
        
        // Reset history position
        self.history_pos = None;
    }
}

impl Default for ReplInput {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_management() {
        let mut input = ReplInput::new(3);
        
        // Add commands
        input.add_to_history("cmd1".to_string());
        input.add_to_history("cmd2".to_string());
        input.add_to_history("cmd3".to_string());
        
        // Test history size limit
        assert_eq!(input.history.len(), 3);
        input.add_to_history("cmd4".to_string());
        assert_eq!(input.history.len(), 3);
        assert_eq!(input.history.front(), Some(&"cmd2".to_string()));
    }

    #[test]
    fn test_duplicate_prevention() {
        let mut input = ReplInput::new(5);
        
        input.add_to_history("cmd1".to_string());
        input.add_to_history("cmd1".to_string()); // Duplicate
        
        assert_eq!(input.history.len(), 1);
    }

    #[test]
    fn test_empty_command_handling() {
        let mut input = ReplInput::new(5);
        
        input.add_to_history("".to_string());
        assert_eq!(input.history.len(), 0);
    }
}
