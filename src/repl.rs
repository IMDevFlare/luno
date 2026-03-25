// --- repl.rs — Luno Interactive REPL ---

use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use std::io::{self, Write};

pub fn run_repl() {
    println!("🌙 Luno v0.1.0 — Interactive REPL");
    println!("Type 'exit' or Ctrl+C to quit.\n");

    let mut interpreter = Interpreter::new();
    let mut buffer = String::new();
    let mut in_block = false;

    loop {
        // Prompt
        if in_block {
            print!("...   ");
        } else {
            print!("luno> ");
        }
        io::stdout().flush().ok();

        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(0) => break, // EOF
            Err(_) => break,
            _ => {}
        }

        let trimmed = line.trim();

        // Exit command
        if trimmed == "exit" || trimmed == "quit" {
            break;
        }

        // Multi-line block handling
        if trimmed.ends_with(':') {
            buffer.push_str(&line);
            in_block = true;
            continue;
        }

        if in_block {
            if trimmed.is_empty() {
                // End of block — execute
                in_block = false;
                let source = buffer.clone();
                buffer.clear();
                execute_source(&source, &mut interpreter);
                continue;
            } else {
                buffer.push_str(&line);
                continue;
            }
        }

        // Single-line statement
        execute_source(trimmed, &mut interpreter);
    }

    println!("\nGoodbye! 🌙");
}

fn execute_source(source: &str, interpreter: &mut Interpreter) {
    // Lex
    let mut lexer = Lexer::new(source);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("SyntaxError: {}", e);
            return;
        }
    };

    // Parse
    let mut parser = Parser::new(tokens);
    let stmts = match parser.parse() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("ParseError: {}", e);
            return;
        }
    };

    // Execute
    if let Err(e) = interpreter.run(&stmts) {
        eprintln!("{}", e);
    }
}
