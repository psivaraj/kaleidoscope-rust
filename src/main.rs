mod ast;
mod lexer;
mod parser;

use std::collections::HashMap;

use ast::{PrototypeAST, Token};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::passes::PassManager;
use inkwell::values::{FloatValue, FunctionValue};
use parser::main_loop;

pub struct State<'ctx> {
    pub cur_tok: Token,
    pub last_char: char,
    pub context: &'ctx Context,
    pub builder: Builder<'ctx>,
    pub module: Module<'ctx>,
    pub fpm: PassManager<FunctionValue<'ctx>>,
    pub named_values: HashMap<String, FloatValue<'ctx>>,
    pub function_protos: HashMap<String, PrototypeAST>,
}

impl<'ctx> State<'ctx> {
    pub fn new(context: &'ctx Context) -> State<'ctx> {
        let module = context.create_module("kaleidoscope");
        let fpm: PassManager<FunctionValue> = PassManager::create(&module);
        // Do simple "peephole" optimizations and bit-twiddling optzns.
        fpm.add_instruction_combining_pass();
        // Reassociate expressions.
        fpm.add_reassociate_pass();
        // Eliminate Common SubExpressions.
        fpm.add_gvn_pass();
        // Simplify the control flow graph (deleting unreachable blocks, etc).
        fpm.add_cfg_simplification_pass();
        fpm.initialize();

        State {
            cur_tok: Token::TokUndef,
            last_char: ' ',
            context,
            builder: context.create_builder(),
            module,
            fpm,
            named_values: HashMap::new(),
            function_protos: HashMap::new(),
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

// TODO: You are ready to start Chapter 4.
