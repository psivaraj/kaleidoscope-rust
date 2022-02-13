mod ast;
mod lexer;
mod parser;

use std::collections::HashMap;

use ast::Token;
use parser::main_loop;
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
    pub named_values: HashMap<String, FloatValue<'ctx>>,
}

impl<'ctx> State<'ctx> {
    pub fn new(context: &'ctx Context) -> State<'ctx> {
        State {
            cur_tok: Token::TokUndef,
            last_char: ' ',
            context,
            builder: context.create_builder(),
            module: context.create_module("kaleidoscope"),
            named_values: HashMap::new(),
        }
    }
}

fn main() {
    // Statements here are executed when the compiled binary is called
    let context = Context::create();
    let mut state = State::new(&context);
    println!("ready> ");

    // Prime the first token
    lexer::get_next_token(&mut state);

      // Run the main "interpreter loop" now.
    main_loop(&mut state);

    println!("{}", state.module.print_to_string());
}

// TODO: You are currently about to test the CallExprAST, for which you just finished implementing the
// the codegen, but have yet to test.
