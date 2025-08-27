use clap::Parser;
use mdq::run::{CliOptions, Error, OsFacade};
use mdq::repl::Repl;
use std::io;
use std::io::{stdin, stdout, Read};
use std::process::ExitCode;

struct RealOs;

#[doc(hidden)]
impl OsFacade for RealOs {
    fn read_stdin(&self) -> io::Result<String> {
        let mut contents = String::new();
        stdin().read_to_string(&mut contents)?;
        Ok(contents)
    }

    fn read_file(&self, path: &str) -> io::Result<String> {
        std::fs::read_to_string(path)
    }

    fn stdout(&mut self) -> impl io::Write {
        stdout().lock()
    }

    fn write_error(&mut self, err: Error) {
        eprint!("{err}")
    }
}

fn main() -> ExitCode {
    let cli = CliOptions::parse();

    if !cli.extra_validation() {
        return ExitCode::FAILURE;
    }

    // Check if REPL mode is requested
    if cli.repl() {
        return match run_repl_mode(&cli) {
            Ok(_) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("REPL error: {}", e);
                ExitCode::FAILURE
            }
        };
    }

    if mdq::run::run(&cli.into(), &mut RealOs) {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Runs the REPL mode
fn run_repl_mode(cli: &CliOptions) -> io::Result<()> {
    let run_options = cli.clone().into();
    let mut repl = Repl::new(run_options)?;
    
    // If files are provided, load the first one
    if !cli.markdown_file_paths().is_empty() {
        let first_file = &cli.markdown_file_paths()[0];
        if first_file != "-" {
            // Load from file
            let content = std::fs::read_to_string(first_file)
                .map_err(|e| io::Error::other(format!("Failed to read file {}: {}", first_file, e)))?;
            repl.load_document(content)
                .map_err(|e| io::Error::other(format!("Failed to load document: {}", e)))?;
        } else {
            // Load from stdin
            let mut content = String::new();
            stdin().read_to_string(&mut content)?;
            repl.load_document(content)
                .map_err(|e| io::Error::other(format!("Failed to load document: {}", e)))?;
        }
    }
    
    // Start REPL
    repl.run()
}
