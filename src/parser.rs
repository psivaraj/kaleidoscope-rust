use crate::ast::{ExprAST, FunctionAST, PrototypeAST, Token};
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
pub fn parse_number_expr(state: &mut State) -> ExprAST {
    match state.cur_tok {
        Token::TokNumber(num) => ExprAST::NumberExprAST { val: num },
        _ => ExprAST::Null,
    }
}

// parenexpr ::= '(' expression ')'
pub fn parse_paren_expr(state: &mut State) -> ExprAST {
    get_next_token(state); // eat (.

    let v = parse_expression(state);

    if matches!(v, ExprAST::Null) {
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
pub fn parse_identifier_expr(state: &mut State) -> ExprAST {
    // TODO: Unclear why I have to clone here
    let id_name = match state.cur_tok.clone() {
        Token::TokIdentifier(a) => a,
        _ => String::new(),
    };

    get_next_token(state); // eat the identifier

    // Handle simple variable reference
    if !matches!(state.cur_tok, Token::TokChar('(')) {
        return ExprAST::VariableExprAST { name: id_name };
    }

    // Call.
    get_next_token(state); // eat '('
    let mut args: Vec<Box<ExprAST>> = Vec::new();
    if !matches!(state.cur_tok, Token::TokChar(')')) {
        loop {
            let arg = parse_expression(state);
            match arg {
                ExprAST::Null => return ExprAST::Null,
                _ => args.push(Box::new(arg)),
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

    return ExprAST::CallExprAST {
        callee: id_name,
        args: args,
    };
}

// primary
//   ::= identifierexpr
//   ::= numberexpr
//   ::= parenexpr
fn parse_primary(state: &mut State) -> ExprAST {
    match state.cur_tok {
        Token::TokIdentifier(_) => return parse_identifier_expr(state),
        Token::TokNumber(_) => return parse_number_expr(state),
        Token::TokChar('(') => return parse_paren_expr(state),
        _ => ExprAST::Null,
    }
}

fn parse_bin_op_rhs(state: &mut State, expr_prec: i32, lhs: ExprAST) -> ExprAST {
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
            _ => panic!("Expecting a TokChar containing a binary operator e.g. `+`"),
        };

        get_next_token(state); // eat binop

        // Parse the primary expression after the binary operator.
        let mut rhs = parse_primary(state);

        if matches!(rhs, ExprAST::Null) {
            return rhs;
        }

        // If BinOp binds less tightly with RHS than the operator after RHS, let
        // the pending operator take RHS as its LHS.
        let next_prec = get_tok_precedence(&state.cur_tok);
        if tok_prec < next_prec {
            rhs = parse_bin_op_rhs(state, tok_prec + 1, rhs);
        }

        return ExprAST::BinaryExprAST {
            op: binop,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        };
    }
}

fn parse_expression(state: &mut State) -> ExprAST {
    let lhs = parse_primary(state);
    if matches!(lhs, ExprAST::Null) {
        return lhs;
    } else {
        return parse_bin_op_rhs(state, 0, lhs);
    }
}

// prototype
//   ::= id '(' id* ')'
fn parse_prototype(state: &mut State) -> PrototypeAST {
    let fn_name = match state.cur_tok.clone() {
        Token::TokIdentifier(a) => a,
        _ => panic!("Expected function name in prototype"),
    };

    get_next_token(state);

    // Handle simple variable reference
    // If we don't get a ")" then we should panic
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

    return PrototypeAST::new(fn_name, arg_names);
}
