mod ast;
mod lexer;
mod parser;

use std::collections::HashMap;

use ast::{Token, AST};
use ast::{NumberExprAST, VariableExprAST};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::FloatValue;

use crate::ast::BinaryExprAST;

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

    // DELETE: Just testing code
    let node_var = VariableExprAST::new(String::from("Hello"));
    let node_num1 = NumberExprAST::new(4.);
    let node_num2 = NumberExprAST::new(2.);
    let mut fp_val1 = node_num1.codegen(&mut state);
    state.named_values.insert(String::from("Hello"), fp_val1);
    let mut fp_val2 = node_var.codegen(&mut state);
    println!("{:?}", fp_val2);

    let node_bin_expr = BinaryExprAST::new('+', AST::Number(node_num1), AST::Number(node_num2));
    let func_val = node_bin_expr.codegen(&mut state);
    println!("{:?}", func_val);

    // Prime the first token
    //lexer::get_next_token(&mut state);
    //main_loop(&mut state);
}
