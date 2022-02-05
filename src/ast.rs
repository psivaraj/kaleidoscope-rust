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

#[derive(Clone)]
pub enum ExprAST {
    Null,

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
pub struct FunctionAST {
    proto: PrototypeAST,
    body: ExprAST,
}
