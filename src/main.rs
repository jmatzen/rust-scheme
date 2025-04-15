mod value;
mod error;
mod parser;
mod env;
mod eval;
mod builtins;

use rustyline::error::ReadlineError;
use rustyline::history::MemHistory;
use rustyline::{DefaultEditor, Editor};
use std::cell::RefCell;
use std::rc::Rc;

use env::Environment;
use error::Result; // Use our custom Result

fn main() -> Result<()> { // Make main return our Result
    println!("Rusty Scheme Interpreter");
    println!("Press Ctrl+C or Ctrl+D to exit");

    // Create top-level environment
    let mut root_env_core = Environment::new();
    builtins::populate_environment(&mut root_env_core);
    let root_env = Rc::new(RefCell::new(root_env_core));

    let mut rl = DefaultEditor::new().expect("nope");
    // You can load history here if you want:
    // if rl.load_history("history.txt").is_err() {
    //     println!("No previous history.");
    // }

    loop {
        let readline = rl.readline("λ> "); // Or use "> "
        match readline {
            Ok(line) => {
                if line.trim().is_empty() {
                    continue; // Skip empty lines
                }
                rl.add_history_entry(line.as_str());

                match parser::parse(&line) {
                    Ok(parsed_expr) => {
                         // Handle the dummy empty symbol from parser
                         if let value::Value::Symbol(s) = &parsed_expr {
                             if s.is_empty() { continue; }
                         }

                        // Evaluate the parsed expression
                        match eval::evaluate(&parsed_expr, Rc::clone(&root_env)) {
                            Ok(result) => println!("{:?}", result), // Use Debug format from value.rs
                            Err(e) => eprintln!("Error: {}", e),
                        }
                    }
                    Err(e) => eprintln!("Parse Error: {}", e),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Interrupted (Ctrl+C)");
                // Optionally break or continue based on preference
                 break; // Exit on Ctrl+C
            }
            Err(ReadlineError::Eof) => {
                println!("Exiting (Ctrl+D)");
                break; // Exit on Ctrl+D
            }
            Err(err) => {
                eprintln!("Readline Error: {:?}", err);
                break;
            }
        }
    }

    // Save history on exit
    // rl.save_history("history.txt").unwrap();

    Ok(())
}