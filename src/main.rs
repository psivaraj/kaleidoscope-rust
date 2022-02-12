mod ast;
mod lexer;
mod parser;

use std::collections::HashMap;

use ast::Token;
use parser::main_loop;
use ast::NumberExprAST;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::FloatValue;

pub struct State<'ctx> {
    pub cur_tok: Token,
    pub last_char: char,
    pub context: &'ctx Context,
    pub builder: Builder<'ctx>,
    pub module: Module<'ctx>,
    pub named_values: HashMap<String, FloatValue<'ctx>>
}

impl<'ctx> State<'ctx> {
    pub fn new(context: &'ctx Context) -> State<'ctx> {
        State {
            cur_tok: Token::TokUndef,
            last_char: ' ',
            context,
            builder: context.create_builder(),
            module: context.create_module("kaleidoscope"),
            named_values: HashMap::new()
        }
    }

    pub fn insert(&'ctx mut self, key: String, value: FloatValue<'ctx>) {
        self.named_values.insert(key, value);
    }
}

fn main() {
    // Statements here are executed when the compiled binary is called
    let context = Context::create();
    let mut state = State::new(&context);
    println!("ready> ");

    // DELETE: Just testing code
    let node = NumberExprAST::new(4.133);
    let mut fp_val = node.codegen(&mut state);
    state.named_values.insert(String::from("Hello"), fp_val);


    // Prime the first token
    lexer::get_next_token(&mut state);
    main_loop(&mut state);
}
