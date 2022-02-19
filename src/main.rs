mod ast;
mod lexer;
mod parser;

use std::collections::HashMap;

use ast::Token;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::ExecutionEngine;
use inkwell::module::Module;
use inkwell::passes::PassManager;
use inkwell::values::{FloatValue, FunctionValue};
use inkwell::OptimizationLevel;
use parser::main_loop;

pub struct State<'ctx> {
    pub cur_tok: Token,
    pub last_char: char,
    pub context: &'ctx Context,
    pub builder: Builder<'ctx>,
    pub module: Module<'ctx>,
    pub execution_engine: ExecutionEngine<'ctx>,
    pub fpm: PassManager<FunctionValue<'ctx>>,
    pub named_values: HashMap<String, FloatValue<'ctx>>,
}

impl<'ctx> State<'ctx> {
    pub fn new(context: &'ctx Context, module: Module<'ctx>) -> State<'ctx> {
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

        let execution_engine = module
            .create_jit_execution_engine(OptimizationLevel::None)
            .unwrap();

        State {
            cur_tok: Token::TokUndef,
            last_char: ' ',
            context,
            builder: context.create_builder(),
            module,
            execution_engine,
            fpm,
            named_values: HashMap::new(),
        }
    }
    pub fn reinit(&mut self) {
        // TODO: Should be able to use add/remove module here instead of re-generating the execution-engine
        self.module = self.context.create_module("kaleidoscope");
        self.execution_engine = self
            .module
            .create_jit_execution_engine(OptimizationLevel::None)
            .unwrap();
    }
}

fn main() {
    // Statements here are executed when the compiled binary is called
    let context = Context::create();
    let module = context.create_module("kaleidoscope");
    let mut state = State::new(&context, module);
    println!("ready> ");

    // Prime the first token
    lexer::get_next_token(&mut state);

    // Run the main "interpreter loop" now.
    main_loop(&mut state);

    println!("{}", state.module.print_to_string());
}

// TODO: You are ready to start Chapter 4.
