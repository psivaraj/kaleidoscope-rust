use crate::ast::{ExprAST, FunctionAST, PrototypeAST, Token, AST};
use crate::lexer::get_next_token;
use crate::State;

pub fn get_tok_precedence(token: &Token) -> i32 {
    match token {
        Token::TokChar('<') => return 10,
        Token::TokChar('+') => return 20,
        Token::TokChar('-') => return 20,
        Token::TokChar('*') => return 40,
        _ => return -1,
    }
}

// numberexpr ::= number
pub fn parse_number_expr(state: &mut State) -> AST {
    let result = match state.cur_tok {
        Token::TokNumber(num) => AST::Expr(ExprAST::NumberExprAST { val: num }),
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
        return AST::Expr(ExprAST::VariableExprAST { name: id_name });
    }

    // Call.
    get_next_token(state); // eat '('
    let mut args: Vec<Box<ExprAST>> = Vec::new();
    if !matches!(state.cur_tok, Token::TokChar(')')) {
        loop {
            let arg = parse_expression(state);
            match arg {
                AST::Expr(arg) => args.push(Box::new(arg)),
                _ => return AST::Null,
            }

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

    return AST::Expr(ExprAST::CallExprAST {
        callee: id_name,
        args: args,
    });
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
        _ => panic!("Unknown token when expecting an expression."),
    }
}

fn parse_bin_op_rhs(state: &mut State, expr_prec: i32, lhs: AST) -> AST {
    loop {
        let tok_prec = get_tok_precedence(&state.cur_tok);

        // If this is a binop that binds at least as tightly as the current binop,
        // consume it, otherwise we are done.
        if tok_prec < expr_prec {
            return lhs;
        }

        // Okay, we know this is a binop.
        let binop = match state.cur_tok {
            Token::TokChar(a) => a,
            _ => return AST::Null,
        };

        get_next_token(state); // eat binop

        // Parse the primary expression after the binary operator.
        let mut rhs = parse_primary(state);

        if matches!(rhs, AST::Null) {
            return rhs;
        }

        // If BinOp binds less tightly with RHS than the operator after RHS, let
        // the pending operator take RHS as its LHS.
        let next_prec = get_tok_precedence(&state.cur_tok);
        if tok_prec < next_prec {
            rhs = parse_bin_op_rhs(state, tok_prec + 1, rhs);
        }

        match (lhs, rhs) {
            (AST::Expr(lhs_arg), AST::Expr(rhs_arg)) => {
                return AST::Expr(ExprAST::BinaryExprAST {
                    op: binop,
                    lhs: Box::new(lhs_arg),
                    rhs: Box::new(rhs_arg),
                })
            }
            _ => return AST::Null,
        }
    }
}

fn parse_expression(state: &mut State) -> AST {
    let lhs = parse_primary(state);
    if matches!(lhs, AST::Null) {
        return lhs;
    } else {
        return parse_bin_op_rhs(state, 0, lhs);
    }
}

// prototype
//   ::= id '(' id* ')'
fn parse_prototype(state: &mut State) -> AST {
    let fn_name = match state.cur_tok.clone() {
        Token::TokIdentifier(a) => a,
        _ => panic!("Expected function name in prototype."),
    };

    get_next_token(state);

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

    return AST::Prototype(PrototypeAST::new(fn_name, arg_names));
}

// definition ::= 'def' prototype expression
fn parse_definition(state: &mut State) -> AST {
    get_next_token(state); // eat def.
    let proto = parse_prototype(state);
    let body = parse_expression(state);

    match (proto, body) {
        (AST::Prototype(proto_arg), AST::Expr(body_arg)) => {
            return AST::Function(FunctionAST::new(proto_arg, body_arg))
        }
        _ => return AST::Null,
    };
}

// toplevelexpr ::= expression
fn parse_top_level_expr(state: &mut State) -> AST {
    let proto = AST::Prototype(PrototypeAST::new(String::from(""), vec![]));
    let body = parse_expression(state);

    match (proto, body) {
        (AST::Prototype(proto_arg), AST::Expr(body_arg)) => {
            return AST::Function(FunctionAST::new(proto_arg, body_arg))
        }
        _ => return AST::Null,
    };
}

// external ::= 'extern' prototype
fn parse_extern(state: &mut State) -> AST {
    get_next_token(state);
    return parse_prototype(state);
}

// TODO: You are at the point of understanding and building static void MainLoop() {
fn handle_defintion(state: &mut State) {
    let node = parse_definition(state);

    if matches!(node, AST::Null) {
        // Skip the token for error recovery
        get_next_token(state);
    } else {
        println!("Parsed a function definition {}.", node);
        println!("Current token is {:?}.", state.cur_tok);
        println!("Last char is {:?}.", state.last_char);
    }
}

fn handle_extern(state: &mut State) {
    let node = parse_extern(state);

    if matches!(node, AST::Null) {
        // Skip the token for error recovery
        get_next_token(state);
    } else {
        println!("Parsed an extern {}.", node);
        println!("Current token is {:?}.", state.cur_tok);
        println!("Last char is {:?}.", state.last_char);
    }
}

fn handle_top_level_expression(state: &mut State) {
    let node = parse_top_level_expr(state);

    if matches!(node, AST::Null) {
        // Skip the token for error recovery
        get_next_token(state);
    } else {
        println!("Parsed a top-level expression {}.", node);
        println!("Current token is {:?}.", state.cur_tok);
        println!("Last char is {:?}.", state.last_char);
    }
}

pub fn main_loop(state: &mut State) {
    loop {
        println!("ready >");
        match state.cur_tok {
            Token::TokEOF => break,
            Token::TokChar(';') => get_next_token(state),
            Token::TokDef => handle_defintion(state),
            Token::TokExtern => handle_extern(state),
            _ => handle_top_level_expression(state),
        }
    }
}
