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
    Expr(ExprAST),
    Prototype(PrototypeAST),
    Function(FunctionAST),
}

#[derive(Debug, Clone)]
pub enum ExprAST {
    // NumberExprAST - Expression class for numeric literals like "1.0".
    NumberExprAST {
        val: f64,
    },

    // VariableExprAST - Expression class for referencing a variable, like "a".
    VariableExprAST {
        name: String,
    },

    // BinaryExprAST - Expression class for a binary operator.
    BinaryExprAST {
        op: char,
        lhs: Box<ExprAST>, // #TODO: Should be an ExprAST
        rhs: Box<ExprAST>,
    },

    // CallExprAST - Expression class for function calls.
    CallExprAST {
        callee: String,
        args: Vec<Box<ExprAST>>, // #TODO: This should be a vector of ExprAST
    },
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
#[derive(Debug)]
pub struct FunctionAST {
    proto: PrototypeAST,
    body: ExprAST,
}

impl FunctionAST {
    pub fn new(proto: PrototypeAST, body: ExprAST) -> Self {
        FunctionAST {
            proto: proto,
            body: body,
        }
    }
}
