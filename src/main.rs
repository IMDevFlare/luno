// --- main.rs — Luno CLI Entry Point ---
// Usage:
//   luno run <file.luno>   — Execute a Luno source file
//   luno repl               — Start interactive REPL
//   luno (no args)          — Start interactive REPL

mod interpreter;
mod lexer;
mod parser;
mod repl;

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => {
            // No arguments — start REPL
            repl::run_repl();
        }
        2 => {
            let cmd = &args[1];
            if cmd == "repl" {
                repl::run_repl();
            } else if cmd.ends_with(".luno") || cmd.ends_with(".ln") {
                // Direct file: luno file.luno
                run_file(cmd);
            } else {
                print_usage();
            }
        }
        3 => {
            let cmd = &args[1];
            let file = &args[2];
            if cmd == "run" {
                run_file(file);
            } else {
                print_usage();
            }
        }
        _ => print_usage(),
    }
}

fn run_file(path: &str) {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: Could not read file '{}': {}", path, e);
            std::process::exit(1);
        }
    };

    let mut lexer = lexer::Lexer::new(&source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("SyntaxError: {}", e);
            std::process::exit(1);
        }
    };

    let mut parser = parser::Parser::new(tokens);
    let stmts = match parser.parse() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ParseError: {}", e);
            std::process::exit(1);
        }
    };

    let mut interp = interpreter::Interpreter::new();
    if let Err(e) = interp.run(&stmts) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn print_usage() {
    println!("🌙 Luno v0.1.0 — A Python-inspired scripting language");
    println!();
    println!("Usage:");
    println!("  luno                   Start the interactive REPL");
    println!("  luno repl              Start the interactive REPL");
    println!("  luno run <file.luno>   Execute a Luno source file");
    println!("  luno <file.luno>       Execute a Luno source file");
    println!();
    println!("File extensions: .luno, .ln");
}
