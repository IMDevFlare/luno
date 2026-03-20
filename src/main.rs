mod ast;
mod interpreter;
mod lexer;
mod parser;

use clap::Parser;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "luno")]
#[command(about = "The Luno Programming Language", long_about = None)]
struct Cli {
    /// The .lo file to run
    file: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    if let Some(file_path) = cli.file {
        run_file(file_path);
    } else {
        run_repl();
    }
}

fn run_file(path: PathBuf) {
    let source = fs::read_to_string(&path).expect("Could not read file");
    if let Err(e) = run(&source) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run_repl() {
    println!("Luno 0.1.0 (REPL)");
    println!("Type 'exit' to quit");
    // REPL implementation could go here
    println!("REPL not yet implemented. Please provide a file.");
}

fn run(source: &str) -> Result<(), String> {
    let mut lexer = lexer::Lexer::new(source);
    let tokens = lexer.scan_tokens()?;

    let mut parser = parser::Parser::new(tokens);
    let statements = parser.parse()?;

    let mut interpreter = interpreter::Interpreter::new();
    interpreter.interpret(statements)?;

    Ok(())
}
