#![allow(unused_variables)]
use std::env;
use std::fs;

use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::parser::stmt::Stmt;

mod interpreter;
mod lexer;
mod parser;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} tokenize <filename>", args[0]);
        return;
    }

    let command = &args[1];
    let filename = &args[2];

    match command.as_str() {
        "tokenize" => {
            // You can use print statements as follows for debugging, they'll be visible when running tests.
            eprintln!("Logs from your program will appear here!");

            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                eprintln!("Failed to read file {}", filename);
                String::new()
            });

            let l = Lexer::new(&file_contents);
            let mut has_error = false;

            for result in l {
                match result {
                    Ok(token) => println!("{token}"),
                    Err(e) => {
                        has_error = true;
                        eprintln!("{e}")
                    }
                }
            }

            println!("EOF  null");

            if has_error {
                std::process::exit(65);
            }
        }
        "parse" => {
            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                eprintln!("Failed to read file {}", filename);
                String::new()
            });

            let l = Lexer::new(&file_contents);
            let mut has_error = false;

            let mut tokens = Vec::new();

            for result in l {
                match result {
                    Ok(token) => {
                        tokens.push(token);
                    }
                    Err(e) => {
                        has_error = true;
                        eprintln!("{e}")
                    }
                }
            }

            if has_error {
                std::process::exit(65);
            }

            let mut parser = Parser::new(tokens);

            match parser.parse_expression() {
                Ok(result) => {
                    println!("{}", result.pretty_print());
                }
                Err(e) => {
                    eprintln!("Parse Error: {e}");
                    std::process::exit(65);
                }
            }
        }
        "evaluate" => {
            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                eprintln!("Failed to read file {}", filename);
                String::new()
            });

            let l = Lexer::new(&file_contents);
            let mut has_error = false;

            let mut tokens = Vec::new();

            for result in l {
                match result {
                    Ok(token) => {
                        tokens.push(token);
                    }
                    Err(e) => {
                        has_error = true;
                        eprintln!("{e}")
                    }
                }
            }

            if has_error {
                std::process::exit(65);
            }

            let mut parser = Parser::new(tokens);

            match parser.parse_expression() {
                Ok(result) => {
                    let interpretter = Interpreter::new();
                    match interpretter.evaluate(&result) {
                        Ok(value) => {
                            println!("{}", value);
                        }
                        Err(e) => {
                            eprintln!("{e}");
                            std::process::exit(70);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{e}");
                    std::process::exit(70);
                }
            }
        }
        "run" => {
            let file_contents = fs::read_to_string(filename).unwrap_or_else(|_| {
                eprintln!("Failed to read file {}", filename);
                String::new()
            });

            let l = Lexer::new(&file_contents);
            let mut has_error = false;

            let mut tokens = Vec::new();

            for result in l {
                match result {
                    Ok(token) => {
                        tokens.push(token);
                    }
                    Err(e) => {
                        has_error = true;
                        eprintln!("{e}")
                    }
                }
            }

            if has_error {
                std::process::exit(65);
            }

            let mut parser = Parser::new(tokens);

            let statements = parser.parse();

            let interpretter = Interpreter::new();

            for statement in statements {
                match statement {
                    Ok(result) => match interpretter.execute(&result) {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("{e}");
                            std::process::exit(70);
                        }
                    },
                    Err(e) => {
                        eprintln!("Parse Error: {e}");
                        std::process::exit(65);
                    }
                }
            }
        }
        _ => {
            eprintln!("Unknown command: {}", command);
        }
    }
}
