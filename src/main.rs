mod ast;
mod lexer;
mod parser;

use ast::Token;
use parser::main_loop;

pub struct State {
    pub cur_tok: Token,
    pub last_char: char,
}

impl State {
    pub fn new() -> State {
        State {
            cur_tok: Token::TokUndef,
            last_char: ' ',
        }
    }
}

fn main() {
    // Statements here are executed when the compiled binary is called

    let mut state = State::new();
    println!("ready> ");

    // Prime the first token
    lexer::get_next_token(&mut state);
    main_loop(&mut state);
}

// TODO: You changed to a flattened AST and :fire: ExprAST, so now you need to check
// all the parsing still works as before. Then you can implement a CodeGen method on
// each struct.
