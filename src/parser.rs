use crate::ast::{ExprAST, Token};
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
    return ExprAST::NumberExprAST { val: state.num_val };
}

// parenexpr ::= '(' expression ')'
pub fn parse_paren_expr(state: &mut State) -> ExprAST {
    get_next_token(state); // eat (.

    // TODO
    // let v = parse_expression();

    // If we don't get a ")" then we should panic
    match state.cur_tok {
        Token::TokChar(')') => (),
        _ => panic!("Expected )"),
    }

    get_next_token(state); // eat ).

    // TODO:
    // return v;
    return ExprAST::Null;
}

// identifierexpr
//   ::= identifier
//   ::= identifier '(' expression* ')'
pub fn parse_identifier_expr(state: &mut State) -> ExprAST {
    let id_name = state.identifier_str.clone();

    get_next_token(state); // eat the identifier

    // Handle simple variable reference
    match state.cur_tok {
        Token::TokChar('(') => (),
        _ => return ExprAST::VariableExprAST { name: id_name },
    }

    // Call.
    get_next_token(state); // eat '('

    let cur_tok = match state.cur_tok {
        Token::TokChar(a) => a,
        _ => ' ',
    };

    let mut args: Vec<Box<ExprAST>> = Vec::new();
    if cur_tok != ')' {
        loop {
            // let arg = parse_expression(state);
            // match arg {
                // ExprAST::Null -> return ExprAST::Null
                // _ -> args.push(Box::new(arg))
            // }

            if let Token::TokChar(')') = state.cur_tok {
                break;
            }

            // Handle simple variable reference
            match state.cur_tok {
                Token::TokChar(',') => (),
                _ => panic!("Expected ')' or ',' in argument list"),
            }

            get_next_token(state);
        }
    }

    // Eat the ')'.
    get_next_token(state);

    // TODO: Fix
    // return ExprAST::CallExprAST{ callee: id_name, args: args};
    return ExprAST::Null;
}

// primary
//   ::= identifierexpr
//   ::= numberexpr
//   ::= parenexpr
fn parse_primary(state: &mut State) -> ExprAST {

    match state.cur_tok {
        Token::TokIdentifier => return parse_identifier_expr(state),
        Token::TokNumber => return parse_identifier_expr(state),
        Token::TokChar('(') => return parse_paren_expr(state),
        _ => panic!("unknown token when expecting an expression")
    }

}

fn parse_bin_op_rhs(state: &mut State, expr_prec: i32, expr_ast: ExprAST) -> ExprAST {
    loop {
        let tok_prec = get_tok_precedence(&state.cur_tok);
    }
}
