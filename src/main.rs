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
    let mut state = State::new();
    lexer::get_next_token(&mut state);
    println!("Next token is {:?}!", state.cur_tok);
    lexer::get_next_token(&mut state);
    println!("Next token is {:?}!", state.cur_tok);
}
