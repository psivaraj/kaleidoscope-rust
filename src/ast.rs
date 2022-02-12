use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    TokUndef,
    TokEOF,

    // commands
    TokDef,
    TokExtern,

    // primary
    TokIdentifier(String),
    TokNumber(f64),

    // catch-all
    TokChar(char),
}

#[derive(Debug)]
pub enum AST {
    Null,
    Number(NumberExprAST),
    Variable(VariableExprAST),
    Binary(BinaryExprAST),
    Call(CallExprAST),
    Prototype(PrototypeAST),
    Function(FunctionAST),
}

// NumberExprAST - Expression class for numeric literals like "1.0".
#[derive(Debug)]
pub struct NumberExprAST {
    val: f64,
}

impl NumberExprAST {
    pub fn new(val: f64) -> Self {
        return NumberExprAST { val: val };
    }
}

// VariableExprAST - Expression class for referencing a variable, like "a".
#[derive(Debug)]
pub struct VariableExprAST {
    name: String,
}

impl VariableExprAST {
    pub fn new(name: String) -> Self {
        return VariableExprAST { name: name };
    }
}

// BinaryExprAST - Expression class for a binary operator.
#[derive(Debug)]
pub struct BinaryExprAST {
    op: char,
    lhs: Box<AST>, // #TODO: Should be an ExprAST
    rhs: Box<AST>,
}

// TODO: Limit this to ExprAST types using generics, marker traits, etc..
impl BinaryExprAST {
    pub fn new(op: char, lhs: AST, rhs: AST) -> Self {
        return BinaryExprAST {
            op: op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        };
    }
}

// CallExprAST - Expression class for function calls.
// TODO: Limit args to ExprAST types using generics, marker traits, etc..
#[derive(Debug)]
pub struct CallExprAST {
    callee: String,
    args: Vec<Box<AST>>,
}

impl CallExprAST {
    pub fn new(callee: String, args: Vec<Box<AST>>) -> Self {
        return CallExprAST { callee: callee, args: args };
    }
}

// PrototypeAST - This class represents the "prototype" for a function,
// which captures its name, and its argument names (thus implicitly the number
// of arguments the function takes).
#[derive(Debug)]
pub struct PrototypeAST {
    name: String,
    args: Vec<String>,
}

impl PrototypeAST {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn new(name: String, args: Vec<String>) -> Self {
        PrototypeAST {
            name: name,
            args: args,
        }
    }
}

// FunctionAST - This class represents a function definition itself.
// TODO: Limit proto and body to specific subsets using generics, marker traits, etc.. rather
// than checking at run-time.
#[derive(Debug)]
pub struct FunctionAST {
    proto: Box<AST>,
    body: Box<AST>,
}

impl FunctionAST {
    pub fn new(proto: AST, body: AST) -> Self {
        assert!(matches!(proto, AST::Prototype(_)));
        assert!(matches!(
            body,
            AST::Number(_) | AST::Variable(_) | AST::Binary(_) | AST::Call(_)
        ));
        FunctionAST {
            proto: Box::new(proto),
            body: Box::new(body),
        }
    }
}
