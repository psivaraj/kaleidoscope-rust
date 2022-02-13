use core::panic;

use crate::State;
use inkwell::types::FloatMathType;
use inkwell::values::FloatValue;
use inkwell::FloatPredicate::OLT;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    // default null
    TokUndef,

    // end of file
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
        return NumberExprAST { val };
    }

    pub fn codegen<'ctx>(&self, state: &mut State<'ctx>) -> FloatValue<'ctx> {
        state.context.f64_type().const_float(self.val)
    }
}

// VariableExprAST - Expression class for referencing a variable, like "a".
#[derive(Debug)]
pub struct VariableExprAST {
    name: String,
}

impl VariableExprAST {
    pub fn new(name: String) -> Self {
        return VariableExprAST { name };
    }
    pub fn codegen<'ctx>(&self, state: &mut State<'ctx>) -> FloatValue<'ctx> {
        let val = state.named_values.get(&self.name);
        match val {
            Some(float_val) => *float_val,
            None => panic!(
                "VariableExprAST code generation failure. Could not find key `{}`",
                self.name
            ),
        }
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
    pub fn codegen<'ctx>(&self, state: &mut State<'ctx>) -> FloatValue<'ctx> {
        let lhs = codegen(state, self.lhs.as_ref());
        let rhs = codegen(state, self.rhs.as_ref());
        match self.op {
            '+' => state.builder.build_float_add(lhs, rhs, "addtmp"),
            '-' => state.builder.build_float_sub(lhs, rhs, "subtmp"),
            '*' => state.builder.build_float_mul(lhs, rhs, "multmp"),
            '<' => {
                let l = state.builder.build_float_compare(OLT, lhs, rhs, "cmptmp");
                state
                    .builder
                    .build_unsigned_int_to_float(l, state.context.f64_type(), "booltmp")
            }
            _ => panic!("BinaryExprAST code generation failure. The operation {} is not supported", self.op)
        }
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
        return CallExprAST { callee, args };
    }
    pub fn codegen<'ctx>(&self, state: &mut State<'ctx>) -> () {
        let val = state.module.get_function(self.callee.as_str());
        let func_val = match val {
            Some(float_val) => float_val,
            None => panic!("CallExprAST code generation failure. Unknown function referenced"),
        };
        if func_val.count_params() != self.args.len().try_into().unwrap() {
            panic!("CallExprAST code generation failure. Incorrect # of arguments passed.");
        }

        let mut args_v = Vec::new();
        for arg in &self.args {
            args_v.push(codegen(state, arg).into())
        }

        state.builder.build_call(func_val, args_v.as_slice(), "calltmp");

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
        PrototypeAST { name, args }
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

// General code generation function
pub fn codegen<'ctx>(state: &mut State<'ctx>, node: &AST) -> FloatValue<'ctx> {
    match node {
        AST::Number(inner_val) => inner_val.codegen(state),
        AST::Variable(inner_val) => inner_val.codegen(state),
        AST::Binary(inner_val) => inner_val.codegen(state),
        // AST::Call(inner_val) => inner_val.codegen(state),
        _ => panic!("BinaryExprAST code generation failure. Could not find key `{:?}`", node)
    }
}
