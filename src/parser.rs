use core::panic;
use std::collections::HashMap;
use std::io::Write;

use crate::ast::{
    codegen, BinaryExprAST, CallExprAST, ForExprAST, FunctionAST, IfExprAST, NumberExprAST,
    PrototypeAST, UnaryExprAST, VarExprAST, VariableExprAST, AST,
};
use crate::lexer::{get_next_token, Token};
use crate::State;
use inkwell::OptimizationLevel;

pub fn get_tok_precedence(state: &State) -> i32 {
    // get the char of the token
    let bin_op = match &state.cur_tok {
        Token::TokChar(this_char) => this_char,
        _ => return -1,
    };
    let precedence = state.bin_op_precedence.get(&bin_op.to_string());
    match precedence {
        Some(val) => *val,
        None => -1,
    }
}

// numberexpr ::= number
pub fn parse_number_expr(state: &mut State) -> AST {
    let result = match state.cur_tok {
        Token::TokNumber(num) => AST::Number(NumberExprAST::new(num)),
        _ => AST::Null,
    };
    get_next_token(state); // consume the Number
    return result;
}

// parenexpr ::= '(' expression ')'
pub fn parse_paren_expr(state: &mut State) -> AST {
    get_next_token(state); // eat (.

    let v = parse_expression(state);

    if matches!(v, AST::Null) {
        return v;
    }

    // If we don't get a ")" then we should panic
    if !matches!(state.cur_tok, Token::TokChar(')')) {
        panic!("Expected ')'");
    }

    get_next_token(state); // eat ).

    return v;
}

// identifierexpr
//   ::= identifier
//   ::= identifier '(' expression* ')'
pub fn parse_identifier_expr(state: &mut State) -> AST {
    let id_name = match state.cur_tok.clone() {
        Token::TokIdentifier(a) => a,
        _ => return AST::Null,
    };

    get_next_token(state); // eat the identifier

    // Handle simple variable reference
    if !matches!(state.cur_tok, Token::TokChar('(')) {
        return AST::Variable(VariableExprAST::new(id_name));
    }

    // Call.
    get_next_token(state); // eat '('
    let mut args: Vec<Box<AST>> = Vec::new();
    if !matches!(state.cur_tok, Token::TokChar(')')) {
        loop {
            let arg = parse_expression(state);
            args.push(Box::new(arg));

            if matches!(state.cur_tok, Token::TokChar(')')) {
                break;
            }

            if !matches!(state.cur_tok, Token::TokChar(',')) {
                panic!("Expected ')' or ',' in argument list")
            }

            get_next_token(state);
        }
    }

    // Eat the ')'.
    get_next_token(state);

    return AST::Call(CallExprAST::new(id_name, args));
}

// primary
//   ::= identifierexpr
//   ::= numberexpr
//   ::= parenexpr
fn parse_primary(state: &mut State) -> AST {
    match state.cur_tok {
        Token::TokChar('(') => return parse_paren_expr(state),
        Token::TokIdentifier(_) => return parse_identifier_expr(state),
        Token::TokNumber(_) => return parse_number_expr(state),
        Token::TokIf => return parse_if_expr(state),
        Token::TokFor => return parse_for_expr(state),
        Token::TokVar => return parse_var_expr(state),
        _ => panic!(
            "Unknown token `{:?}` when expecting an expression.",
            state.cur_tok
        ),
    }
}

fn parse_bin_op_rhs(state: &mut State, expr_prec: i32, lhs: AST) -> AST {
    let mut lhs_loop = lhs;
    loop {
        let tok_prec = get_tok_precedence(&state);

        // If this is a binop that binds at least as tightly as the current binop,
        // consume it, otherwise we are done.
        if tok_prec < expr_prec {
            return lhs_loop;
        }

        // Okay, we know this is a binop.
        let binop = match state.cur_tok {
            Token::TokChar(a) => a,
            _ => return AST::Null,
        };

        get_next_token(state); // eat binop

        // Parse the primary expression after the binary operator.
        let mut rhs = parse_unary(state);

        if matches!(rhs, AST::Null) {
            return rhs;
        }

        // If BinOp binds less tightly with RHS than the operator after RHS, let
        // the pending operator take RHS as its LHS.
        let next_prec = get_tok_precedence(&state);
        if tok_prec < next_prec {
            rhs = parse_bin_op_rhs(state, tok_prec + 1, rhs);
        }

        lhs_loop = AST::Binary(BinaryExprAST::new(binop, lhs_loop, rhs));
    }
}

fn parse_expression(state: &mut State) -> AST {
    let lhs = parse_unary(state);
    if matches!(lhs, AST::Null) {
        return lhs;
    } else {
        return parse_bin_op_rhs(state, 0, lhs);
    }
}

fn parse_unary(state: &mut State) -> AST {
    // If the current token is not an operator, it must be a primary expr.
    if !matches!(state.cur_tok, Token::TokChar(_)) {
        return parse_primary(state);
    };

    // If this is a unary operator, read it.
    match state.cur_tok {
        Token::TokChar(this_char) => {
            if this_char == '(' || this_char == ')' {
                return parse_primary(state);
            }
            get_next_token(state);
            let operand = parse_unary(state);
            return AST::Unary(UnaryExprAST::new(this_char, operand))
        }
        _ => return AST::Null,
    }
}

// prototype
//   ::= id '(' id* ')'
fn parse_prototype(state: &mut State) -> AST {
    let mut fn_name: String;

    let kind: usize; // 0 = identifier, 1 = unary, 2 = binary.
    let mut binary_precedence = 30;

    match state.cur_tok.clone() {
        Token::TokIdentifier(a) => {
            fn_name = a;
            kind = 0;
            get_next_token(state);
        }
        Token::TokBinary => {
            get_next_token(state);
            let this_char = match state.cur_tok {
                Token::TokChar(this_char) => {
                    assert!(this_char.is_ascii(), "Expected binary operator");
                    this_char
                }
                _ => panic!("Expected binary operator"),
            };
            fn_name = String::from("binary");
            fn_name.push_str(&this_char.to_string());
            kind = 2;
            get_next_token(state);

            // Read the precedence if present.
            if let Token::TokNumber(number) = state.cur_tok {
                if number < 1. || number > 100. {
                    panic!("Invalid precedence: must be 1..100");
                }
                binary_precedence = number as i32;
                get_next_token(state);
            }
        }
        Token::TokUnary => {
            get_next_token(state);
            let this_char = match state.cur_tok {
                Token::TokChar(this_char) => {
                    assert!(this_char.is_ascii(), "Expected unary operator");
                    this_char
                }
                _ => panic!("Expected binary operator"),
            };
            fn_name = String::from("unary");
            fn_name.push_str(&this_char.to_string());
            kind = 1;
            get_next_token(state);
        }
        _ => panic!("Expected function name in prototype"),
    };

    if !matches!(state.cur_tok, Token::TokChar('(')) {
        panic!("Expected '(' in prototype");
    }

    let mut arg_names: Vec<String> = Vec::new();
    get_next_token(state);

    while matches!(state.cur_tok, Token::TokIdentifier(_)) {
        if let Token::TokIdentifier(a) = state.cur_tok.clone() {
            arg_names.push(a)
        }
        get_next_token(state);
    }

    if !matches!(state.cur_tok, Token::TokChar(')')) {
        panic!("Expected ')' in prototype");
    }

    // success.
    get_next_token(state); // eat ')'.

    // Verify right number of names for operator.
    if kind != 0 && arg_names.len() != kind {
        panic!("Invalid number of operands for operator")
    }

    return AST::Prototype(PrototypeAST::new(
        fn_name,
        arg_names,
        kind != 0,
        binary_precedence,
    ));
}

// definition ::= 'def' prototype expression
fn parse_definition(state: &mut State) -> AST {
    get_next_token(state); // eat def.
    let proto = parse_prototype(state);
    let body = parse_expression(state);

    return AST::Function(FunctionAST::new(proto, body));
}

// toplevelexpr ::= expression
fn parse_top_level_expr(state: &mut State) -> AST {
    let proto = AST::Prototype(PrototypeAST::new(String::from("anon"), vec![], false, 0));
    let body = parse_expression(state);

    return AST::Function(FunctionAST::new(proto, body));
}

// external ::= 'extern' prototype
fn parse_extern(state: &mut State) -> AST {
    get_next_token(state);
    let proto = parse_prototype(state);
    return proto;
}

// ifexpr ::= 'if' expression 'then' expression 'else' expression
fn parse_if_expr(state: &mut State) -> AST {
    get_next_token(state); // eat the `if`

    // condition.
    let cond = parse_expression(state);

    if !matches!(state.cur_tok, Token::TokThen) {
        panic!("Expected 'then' in if expression");
    };

    get_next_token(state); // eat the `then`

    let then = parse_expression(state);

    if !matches!(state.cur_tok, Token::TokElse) {
        panic!("Expected 'else' in if expression");
    };

    get_next_token(state); // eat the `else`

    let els = parse_expression(state);

    return AST::If(IfExprAST::new(cond, then, els));
}

// forexpr ::= 'for' identifier '=' expr ',' expr (',' expr)? 'in' expression
fn parse_for_expr(state: &mut State) -> AST {
    get_next_token(state); // eat the `for`

    let id_name = match state.cur_tok.clone() {
        Token::TokIdentifier(a) => a,
        _ => return AST::Null,
    };
    get_next_token(state); // eat the identifier

    if !matches!(state.cur_tok, Token::TokChar('=')) {
        panic!("Expected '=' after for");
    };
    get_next_token(state); // eat '='.

    let start = parse_expression(state);
    if !matches!(state.cur_tok, Token::TokChar(',')) {
        panic!("Expected ',' after for start value");
    };
    get_next_token(state); // eat the ','

    let end = parse_expression(state);

    // Step value is optional
    let mut step = AST::Null;
    if matches!(state.cur_tok, Token::TokChar(',')) {
        get_next_token(state); // eat the ','
        step = parse_expression(state);
    };

    if !matches!(state.cur_tok, Token::TokIn) {
        panic!("Expected 'in' after for");
    };
    get_next_token(state); // eat the `in`

    let body = parse_expression(state);

    return AST::For(ForExprAST::new(id_name, start, end, step, body));
}

// varexpr ::= 'var' identifier ('=' expression)?
//                    (',' identifier ('=' expression)?)* 'in' expression
fn parse_var_expr(state: &mut State) -> AST {
    get_next_token(state); // eat the `var`

    let mut names: HashMap<String, AST> = HashMap::new();

    // At least one variable name is required.
    assert!(
        matches!(state.cur_tok, Token::TokIdentifier(_)),
        "expected identifier after var"
    );

    loop {
        let id_name = match state.cur_tok.clone() {
            Token::TokIdentifier(a) => a,
            _ => return AST::Null,
        };
        get_next_token(state); // eat the `identifier`

        // Step value is optional
        let mut init = AST::Null;
        if matches!(state.cur_tok, Token::TokChar('=')) {
            get_next_token(state); // eat the '='
            init = parse_expression(state);
        };

        names.insert(id_name.to_string(), init);

        // End of var list, exit loop.
        if !matches!(state.cur_tok, Token::TokChar(',')) {
            break;
        }

        get_next_token(state); // eat the ','.

        assert!(
            matches!(state.cur_tok, Token::TokIdentifier(_)),
            "expected identifier list after var"
        );
    }

    if !matches!(state.cur_tok, Token::TokIn) {
        panic!("expected 'in' keyword after 'var'");
    };

    get_next_token(state); // eat the 'in'.

    let body = parse_expression(state);

    return AST::Var(VarExprAST::new(names, body));
}

fn handle_definition(state: &mut State) {
    // TODO: Can't redefine files yet.
    let node = parse_definition(state);

    if matches!(node, AST::Null) {
        // Skip the token for error recovery
        get_next_token(state);
    } else {
        codegen(state, &node);
    }
}

fn handle_extern(state: &mut State) {
    let node = parse_extern(state);

    if matches!(node, AST::Null) {
        // Skip the token for error recovery
        get_next_token(state);
    } else {
        codegen(state, &node);

        let proto = match node {
            AST::Prototype(val) => val,
            _ => panic!(
                "FunctionAST code generation failure, expected a ProtoTypeAST for proto field."
            ),
        };
        state
            .function_protos
            .insert(proto.get_name().to_string(), proto);
    }
}

fn handle_top_level_expression(state: &mut State) {
    let temp_module = state.module.clone();
    let node = parse_top_level_expr(state);

    if matches!(node, AST::Null) {
        // Skip the token for error recovery
        get_next_token(state);
    } else {
        codegen(state, &node);
        unsafe {
            let ee = state
                .module
                .create_jit_execution_engine(OptimizationLevel::None)
                .unwrap();
            let test_fn = ee
                .get_function::<unsafe extern "C" fn() -> f64>("anon")
                .unwrap();
            let return_value = test_fn.call();
            println!("Out[#]: {return_value}\n");
        };
    }
    state.module = temp_module;
}

pub fn main_loop(state: &mut State) {
    print!("In [#]: ");
    std::io::stdout().flush().unwrap();
    // Prime the first token
    get_next_token(state);
    match state.cur_tok {
        Token::TokChar(';') => get_next_token(state),
        Token::TokDef => handle_definition(state),
        Token::TokExtern => handle_extern(state),
        _ => handle_top_level_expression(state),
    };
    loop {
        print!("In [#]: ");
        std::io::stdout().flush().unwrap();
        get_next_token(state);
        match state.cur_tok {
            Token::TokEOF => break,
            Token::TokChar(';') => get_next_token(state),
            Token::TokDef => handle_definition(state),
            Token::TokExtern => handle_extern(state),
            _ => handle_top_level_expression(state),
        }
    }
}
