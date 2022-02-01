mod ast;
mod lexer;
mod parser;

use ast::Token;

pub struct State {
    pub identifier_str: String,
    pub num_val: f64,
    pub cur_tok: Token,
    pub last_char: char,
}

impl State {
    pub fn new() -> State {
        State {
            identifier_str: String::from(""),
            num_val: 0.0,
            cur_tok: Token::TokUndef,
            last_char: ' ',
        }
    }
}

fn main() {
    // Statements here are executed when the compiled binary is called

    // Print text to the console
    println!("Hello World!");
    lexer::getchar();
}
