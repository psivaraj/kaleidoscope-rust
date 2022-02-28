use crate::State;
use inkwell::values::{AnyValueEnum, BasicValue, FunctionValue, PointerValue};
use inkwell::FloatPredicate::{ONE, ULT};

#[derive(Debug)]
pub enum AST {
    Null,
    Number(NumberExprAST),
    Variable(VariableExprAST),
    Binary(BinaryExprAST),
    Call(CallExprAST),
    If(IfExprAST),
    For(ForExprAST),
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

    pub fn codegen<'ctx>(&self, state: &State<'ctx>) -> AnyValueEnum<'ctx> {
        state.context.f64_type().const_float(self.val).into()
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
    pub fn codegen<'ctx>(&self, state: &State<'ctx>) -> AnyValueEnum<'ctx> {
        let val = state.named_values.get(&self.name);
        match val {
            Some(ptr_val) => state.builder.build_load(*ptr_val, &self.name).into(),
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
    pub fn codegen<'ctx>(&self, state: &mut State<'ctx>) -> AnyValueEnum<'ctx> {
        // Special case '=' because we don't want to emit the LHS as an expression.
        if let '=' = self.op {
            // Assignment requires the LHS to be an identifier.
            let lhse = match self.lhs.as_ref() {
                AST::Variable(val) => val,
                _ => panic!("destination of '=' must be a variable"),
            };

            // Codegen the RHS.
            let val = codegen(state, self.rhs.as_ref()).into_float_value();

            let var = state.named_values.get(&(lhse.name)).unwrap();

            state.builder.build_store(*var, val);
            return val.into();
        }

        let lhs = codegen(state, self.lhs.as_ref()).into_float_value();
        let rhs = codegen(state, self.rhs.as_ref()).into_float_value();

        match self.op {
            '+' => state.builder.build_float_add(lhs, rhs, "addtmp").into(),
            '-' => state.builder.build_float_sub(lhs, rhs, "subtmp").into(),
            '*' => state.builder.build_float_mul(lhs, rhs, "multmp").into(),
            '<' => {
                let l = state.builder.build_float_compare(ULT, lhs, rhs, "cmptmp");
                state
                    .builder
                    .build_unsigned_int_to_float(l, state.context.f64_type(), "booltmp")
                    .into()
            }
            _ => panic!(
                "BinaryExprAST code generation failure. The operation {} is not supported",
                self.op
            ),
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
    pub fn codegen<'ctx>(&self, state: &mut State<'ctx>) -> AnyValueEnum<'ctx> {
        let func_val = get_function(state, self.callee.as_str());
        if func_val.count_params() != self.args.len().try_into().unwrap() {
            panic!("CallExprAST code generation failure. Incorrect # of arguments passed.");
        }

        let mut args_v = Vec::new();
        for arg in &self.args {
            args_v.push(codegen(state, arg).into_float_value().into())
        }

        let call_site_val = state
            .builder
            .build_call(func_val, args_v.as_slice(), "calltmp");
        call_site_val
            .try_as_basic_value()
            .unwrap_left()
            .into_float_value()
            .into()
    }
}

// IfExprAST - Expression class for if/then/else.
#[derive(Debug)]
pub struct IfExprAST {
    cond: Box<AST>,
    then: Box<AST>,
    els: Box<AST>,
}

impl IfExprAST {
    pub fn new(cond: AST, then: AST, els: AST) -> Self {
        return IfExprAST {
            cond: Box::new(cond),
            then: Box::new(then),
            els: Box::new(els),
        };
    }

    pub fn codegen<'ctx>(&self, state: &mut State<'ctx>) -> AnyValueEnum<'ctx> {
        let condv = codegen(state, self.cond.as_ref());

        let condv_out = state.builder.build_float_compare(
            ONE,
            condv.into_float_value(),
            state.context.f64_type().const_float(0.0),
            "ifcond",
        );

        // Needed because in the LLVM context, we are within a function, so let's grab that
        // function object.
        let orig_block = state.builder.get_insert_block().unwrap();
        let func_value = orig_block.get_parent().unwrap();

        // Create blocks for the then and else cases.  Insert the 'then' block at the
        // end of the function.
        let mut then_bb = state.context.append_basic_block(func_value, "then");
        let mut else_bb = state.context.append_basic_block(func_value, "else");
        let merge_bb = state.context.append_basic_block(func_value, "ifcont");

        state
            .builder
            .build_conditional_branch(condv_out, then_bb, else_bb);

        // Emit then block
        state.builder.position_at_end(then_bb);
        let thenv = codegen(state, self.then.as_ref());
        state.builder.build_unconditional_branch(merge_bb);
        // codegen of 'Then' can change the current block, update ThenBB for the PHI.
        then_bb = state.builder.get_insert_block().unwrap();

        // Emit else block
        else_bb.move_after(then_bb).unwrap();
        state.builder.position_at_end(else_bb);
        let elsev = codegen(state, self.els.as_ref());
        state.builder.build_unconditional_branch(merge_bb);
        // codegen of 'Else' can change the current block, update ElseBB for the PHI.
        else_bb = state.builder.get_insert_block().unwrap();

        // Emit merge block
        merge_bb.move_after(else_bb).unwrap();
        state.builder.position_at_end(merge_bb);
        let phi_node = state.builder.build_phi(state.context.f64_type(), "iftmp");
        phi_node.add_incoming(&[
            (&thenv.into_float_value(), then_bb),
            (&elsev.into_float_value(), else_bb),
        ]);

        return phi_node.as_basic_value().into();
    }
}

// ForExprAST - Expression class for for/in.
#[derive(Debug)]
pub struct ForExprAST {
    name: String,
    start: Box<AST>,
    end: Box<AST>,
    step: Box<AST>,
    body: Box<AST>,
}

impl ForExprAST {
    pub fn new(name: String, start: AST, end: AST, step: AST, body: AST) -> Self {
        return ForExprAST {
            name,
            start: Box::new(start),
            end: Box::new(end),
            step: Box::new(step),
            body: Box::new(body),
        };
    }

    pub fn codegen<'ctx>(&self, state: &mut State<'ctx>) -> AnyValueEnum<'ctx> {
        let preheader_bb = state.builder.get_insert_block().unwrap();
        let func_value = preheader_bb.get_parent().unwrap();

        let alloca = create_entry_block_alloca(state, func_value, &self.name);

        // Emit the start code first, without 'variable' in scope.
        let start_val = codegen(state, self.start.as_ref());

        // Store the value into alloca
        state
            .builder
            .build_store(alloca, start_val.into_float_value());

        // Make the new basic block for the loop header, inserting after current
        let loop_bb = state.context.append_basic_block(func_value, "loop");

        // Insert an explicit fall through from the current block to the loop_bb.
        state.builder.build_unconditional_branch(loop_bb);

        // start insertion in loop_bb
        state.builder.position_at_end(loop_bb);

        // Within the loop, the variable is defined equal to the PHI node.  If it
        // shadows an existing variable, we have to restore it, so save it now.
        let old_val = state.named_values.insert(self.name.clone(), alloca);

        // Emit the body of the loop.  This, like any other expr, can change the
        // current BB.  Note that we ignore the value computed by the body.
        codegen(state, self.body.as_ref());

        // Emit the step value
        let step_val: AnyValueEnum;
        if !matches!(self.step.as_ref(), AST::Null) {
            step_val = codegen(state, self.step.as_ref());
        } else {
            // If not specified, use 1.0.
            step_val = state.context.f64_type().const_float(0.0).into();
        };

        // Compute the end condition.
        let end_cond = codegen(state, self.end.as_ref());

        // Reload, increment, and restore the alloca.  This handles the case where
        // the body of the loop mutates the variable.
        let cur_var = state.builder.build_load(alloca, &self.name);
        let next_var = state.builder.build_float_add(
            cur_var.into_float_value(),
            step_val.into_float_value(),
            "nextvar",
        );
        state.builder.build_store(alloca, next_var);

        // Convert condition to a bool by comparing non-equal to 0.0.
        let end_cond_val = state.builder.build_float_compare(
            ONE,
            end_cond.into_float_value(),
            state.context.f64_type().const_float(0.0).into(),
            "loopcond",
        );

        // Create the "after loop" block and insert it.
        let after_bb = state.context.append_basic_block(func_value, "afterloop");

        // Insert the conditional branch into the end of LoopEndBB.
        state
            .builder
            .build_conditional_branch(end_cond_val, loop_bb, after_bb);

        // Any new code will be inserted in AfterBB.
        state.builder.position_at_end(after_bb);

        // Restore the unshadowed variable
        if let Some(val) = old_val {
            state.named_values.insert(self.name.clone(), val);
        } else {
            state.named_values.remove(&self.name);
        };

        // for expr always returns 0.0.
        return state.context.f64_type().const_float(0.0).into();
    }
}

// PrototypeAST - This class represents the "prototype" for a function,
// which captures its name, and its argument names (thus implicitly the number
// of arguments the function takes).
#[derive(Debug, Clone)]
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

    pub fn codegen<'ctx>(&self, state: &State<'ctx>) -> AnyValueEnum<'ctx> {
        let mut param_types = Vec::new();
        for _ in &self.args {
            param_types.push(state.context.f64_type().into())
        }

        let func_type = state
            .context
            .f64_type()
            .fn_type(param_types.as_slice(), false);

        let func = state
            .module
            .add_function(self.name.as_str(), func_type, None);

        for (i, arg) in func.get_param_iter().enumerate() {
            arg.into_float_value().set_name(self.args[i].as_str());
        }

        return func.into();
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
        // body must be an ExprAST type
        assert!(matches!(
            body,
            AST::Number(_)
                | AST::Variable(_)
                | AST::Binary(_)
                | AST::Call(_)
                | AST::If(_)
                | AST::For(_)
        ));
        FunctionAST {
            proto: Box::new(proto),
            body: Box::new(body),
        }
    }

    pub fn codegen<'ctx>(&self, state: &mut State<'ctx>) -> AnyValueEnum<'ctx> {
        // Get the proto body
        let proto = match self.proto.as_ref() {
            AST::Prototype(val) => val,
            _ => panic!(
                "FunctionAST code generation failure, expected a ProtoTypeAST for proto field."
            ),
        };

        // Transfer ownership of the prototype to the FunctionProtos map
        state
            .function_protos
            .insert(proto.get_name().to_string(), proto.clone());

        let func_value = get_function(state, proto.get_name());

        // Create a new basic block to start insertion into.
        let basic_block = state.context.append_basic_block(func_value, "entry");
        state.builder.position_at_end(basic_block);

        // Record the function arguments in the NamedValues map.
        state.named_values.clear();
        for arg in func_value.get_param_iter() {
            // Create an alloca for this variable.
            let arg_float_val = arg.into_float_value();
            let arg_name = arg_float_val.get_name().to_str().unwrap();
            let alloca = create_entry_block_alloca(state, func_value, arg_name);

            // Store the initial value into the alloca.
            state.builder.build_store(alloca, arg);

            // Add arguments to variable symbol table.
            state.named_values.insert(arg_name.to_string(), alloca);
        }

        let retval = codegen(state, &*self.body).into_float_value();
        state.builder.build_return(Some(&retval));

        assert!(
            func_value.verify(false),
            "FunctionAST code generation failure. LLVM could not verify function."
        );

        state.fpm.run_on(&func_value);

        return func_value.into();
    }
}

// General code generation function
// TODO: There's got to be a better way -- presumably with anonymous functions
pub fn codegen<'ctx>(state: &mut State<'ctx>, node: &AST) -> AnyValueEnum<'ctx> {
    match node {
        AST::Number(inner_val) => inner_val.codegen(state),
        AST::Variable(inner_val) => inner_val.codegen(state),
        AST::Binary(inner_val) => inner_val.codegen(state),
        AST::Call(inner_val) => inner_val.codegen(state),
        AST::Function(inner_val) => inner_val.codegen(state),
        AST::Prototype(inner_val) => inner_val.codegen(state),
        AST::If(inner_val) => inner_val.codegen(state),
        AST::For(inner_val) => inner_val.codegen(state),
        _ => panic!(
            "General code generation failure. Could not find key `{:?}`",
            node
        ),
    }
}

// General helper to get function
pub fn get_function<'ctx>(state: &mut State<'ctx>, name: &str) -> FunctionValue<'ctx> {
    let val = state.module.get_function(name);
    if let Some(func_val) = val {
        return func_val;
    };

    let proto_some = state.function_protos.get(&name.to_string());
    match proto_some {
        Some(proto) => return proto.codegen(state).into_function_value(),
        None => panic!("get_function failure. Could not find key `{name}`",),
    }
}

// func_value.get_first_basic_block();
// create a builder and position at basic block
// use the builder to generate an alloca
// state.builder.build_alloca(state.context.f64_type(), name);
pub fn create_entry_block_alloca<'ctx>(
    state: &mut State<'ctx>,
    func_value: FunctionValue<'ctx>,
    name: &str,
) -> PointerValue<'ctx> {
    let entry_bb = func_value.get_first_basic_block().unwrap();
    let builder = state.context.create_builder();
    builder.position_at_end(entry_bb);
    builder.build_alloca(state.context.f64_type(), name)
}
