mod ast;
mod lexer;
mod parser;

use ast::Token;

pub struct State {
    pub cur_tok: Token,
}

impl State {
    pub fn new() -> State {
        State {
            cur_tok: Token::TokUndef,
        }
    }
}

fn main() {
    // Statements here are executed when the compiled binary is called

    // Print text to the console
    println!("Hello World!");
    lexer::getchar();
}
