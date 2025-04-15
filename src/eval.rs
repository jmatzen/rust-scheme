use crate::env::Environment;
use crate::error::{Result, SchemeError};
use crate::value::{Value, BuiltinFn};
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;

// Represents the result of one evaluation step, facilitating TCO.
enum EvalResult {
    Value(Value),
    TailCall {
        proc: Value, // The procedure to call (Lambda or Builtin)
        args: Vec<Value>, // Evaluated arguments
        env: Rc<RefCell<Environment>>, // Environment for the call
    },
}

// The main evaluation function, potentially returning a TailCall signal.
// Takes &Value as input, matching evaluate's initial call signature
fn eval_step(expr: &Value, env: Rc<RefCell<Environment>>) -> Result<EvalResult> {
    // --- No changes needed inside eval_step itself ---
    // --- It still operates on a reference &Value ---
     match expr {
        // Self-evaluating types
        Value::Integer(_) | Value::Bool(_) | Value::String(_) | Value::Nil |
        Value::Array(_) | Value::Map(_) | Value::Lambda { .. } | Value::Builtin(_, _) => Ok(EvalResult::Value(expr.clone())),

        // Symbol lookup
        Value::Symbol(s) => {
            if s.is_empty() { // Handle the empty symbol from parser for empty input
                 Ok(EvalResult::Value(Value::Nil)) // Or some other inert value
            } else {
                env.borrow()
                   .lookup(s)
                   .map(EvalResult::Value)
                   .ok_or_else(|| SchemeError::UndefinedVariable(s.clone()))
            }
        }

        // List evaluation (special forms and procedure calls)
        Value::List(list) => {
            if list.is_empty() {
                return Ok(EvalResult::Value(Value::Nil));
            }

            let (op_expr, args_expr) = list.split_first().unwrap();

            // Handle Special Forms
            if let Value::Symbol(op_sym) = op_expr {
                match op_sym.as_str() {
                    "quote" => {
                        if args_expr.len() != 1 {
                            return Err(SchemeError::Arity { expected: "1".to_string(), got: args_expr.len() });
                        }
                        return Ok(EvalResult::Value(args_expr[0].clone()));
                    }
                    "if" => {
                        if !(args_expr.len() == 2 || args_expr.len() == 3) {
                             return Err(SchemeError::Arity { expected: "2 or 3".to_string(), got: args_expr.len() });
                        }
                        let cond_expr = &args_expr[0];
                        // Evaluate condition first using the main evaluate function
                        let cond_val = evaluate_trampolined(Rc::new(cond_expr.clone()), Rc::clone(&env))?; // Clone expr into Rc for evaluate
                        let branch_expr = match cond_val {
                            Value::Bool(false) => { // Only #f is false
                                if args_expr.len() == 3 {
                                    &args_expr[2] // Else branch
                                } else {
                                    return Ok(EvalResult::Value(Value::Nil)); // Undefined/unspecified in R5RS, Nil is common
                                }
                            }
                            _ => &args_expr[1], // Then branch (anything else is true)
                        };
                        // Tail call: evaluate the chosen branch in the same context
                        // Pass the branch reference directly to eval_step for the *next* step
                        return eval_step(branch_expr, env);
                    }
                    "define" => {
                         if args_expr.len() != 2 {
                             return Err(SchemeError::Arity { expected: "2".to_string(), got: args_expr.len() });
                        }
                        let var_expr = &args_expr[0];
                        let val_expr = &args_expr[1];

                        let name = match var_expr {
                             Value::Symbol(s) => s.clone(),
                             _ => return Err(SchemeError::Type{ expected: "symbol".to_string(), found: var_expr.type_name()}),
                        };

                        // Evaluate the value using the main evaluate function
                        let value = evaluate_trampolined(Rc::new(val_expr.clone()), Rc::clone(&env))?; // Clone expr into Rc for evaluate
                        // Define in the *current* environment
                        env.borrow_mut().define(name, value);
                        return Ok(EvalResult::Value(Value::Nil));
                    }
                     "set!" => {
                        if args_expr.len() != 2 {
                            return Err(SchemeError::Arity { expected: "2".to_string(), got: args_expr.len() });
                        }
                        let var_expr = &args_expr[0];
                        let val_expr = &args_expr[1];

                        let name = match var_expr {
                            Value::Symbol(s) => s.clone(),
                            _ => return Err(SchemeError::Type{ expected: "symbol".to_string(), found: var_expr.type_name()}),
                        };

                        // Evaluate the value using the main evaluate function
                        let value = evaluate_trampolined(Rc::new(val_expr.clone()), Rc::clone(&env))?; // Clone expr into Rc for evaluate
                        // Set in the environment chain
                        env.borrow_mut().set(&name, value)?;
                        return Ok(EvalResult::Value(Value::Nil));
                    }
                    "lambda" => {
                         if args_expr.len() < 1 {
                            return Err(SchemeError::Eval("Invalid lambda syntax: requires parameters and body".to_string()));
                        }
                        let params_expr = &args_expr[0];
                        let body_exprs = &args_expr[1..];

                        let params: Rc<Vec<String>> = match params_expr {
                            Value::List(p_list) => {
                                let mut names = Vec::new();
                                for p in p_list {
                                    if let Value::Symbol(s) = p {
                                        names.push(s.clone());
                                    } else {
                                        return Err(SchemeError::Eval("Lambda parameters must be symbols".to_string()));
                                    }
                                }
                                Rc::new(names)
                            }
                            _ => return Err(SchemeError::Eval("Lambda parameters must be a list of symbols".to_string())),
                        };

                        let body = if body_exprs.len() == 1 {
                             Rc::new(body_exprs[0].clone()) // body is Rc<Value>
                        } else {
                            let mut begin_list = vec![Value::Symbol("begin".to_string())];
                            begin_list.extend(body_exprs.iter().cloned());
                             Rc::new(Value::List(begin_list)) // body is Rc<Value>
                        };

                        let lambda = Value::Lambda {
                            params,
                            body,
                            env: Rc::clone(&env), // Capture current environment
                        };
                        return Ok(EvalResult::Value(lambda));
                    }
                     "begin" => {
                        if args_expr.is_empty() {
                            return Ok(EvalResult::Value(Value::Nil));
                        }
                        // Evaluate all but the last sequentially for side effects
                        for expr in &args_expr[..args_expr.len() - 1] {
                            // Use the main evaluate function here
                             evaluate_trampolined(Rc::new(expr.clone()), Rc::clone(&env))?; // Clone expr into Rc for evaluate
                        }
                        // Tail call: evaluate the last expression by passing it to next eval_step
                        return eval_step(&args_expr[args_expr.len() - 1], env);
                    }
                    _ => {} // Not a special form, proceed to procedure call
                }
            }
         // --- Procedure Call ---
            // 1. Evaluate the operator using the main evaluate function
            // proc_val is the evaluated procedure (Value::Lambda or Value::Builtin)
            let proc_val = evaluate_trampolined(Rc::new(op_expr.clone()), Rc::clone(&env))?;

            // 2. Evaluate arguments using the main evaluate function
            let mut args_val = Vec::with_capacity(args_expr.len());
            for arg_expr in args_expr {
                args_val.push(evaluate_trampolined(Rc::new(arg_expr.clone()), Rc::clone(&env))?);
            }

            // 3. Prepare for tail call (return TailCall signal)
            // --- FIX: Match on a reference to proc_val ---
            match &proc_val {
                Value::Lambda { env: lambda_env, params: _, body: _ } => { // Use _ for fields not needed here
                    // lambda_env is now &Rc<RefCell<Environment>> (a reference to the Rc)
                    // proc_val is still fully valid because we only borrowed it.
                    Ok(EvalResult::TailCall {
                        // Clone the whole procedure Value (Lambda variant)
                        proc: proc_val.clone(),
                        args: args_val,
                        // Clone the Rc pointer for the captured environment
                        env: Rc::clone(lambda_env),
                    })
                }
                Value::Builtin { .. } => {
                    // proc_val is still fully valid.
                    Ok(EvalResult::TailCall {
                        // Clone the whole procedure Value (Builtin variant)
                        proc: proc_val.clone(),
                        args: args_val,
                        // For builtins, the 'next' environment is just the *current*
                        // environment where the call is happening. Clone its Rc.
                        env: Rc::clone(&env),
                    })
                }
                _ => Err(SchemeError::NotProcedure(format!("{:?}", proc_val))), // Can still use proc_val here safely
            }
        }
    }
}

// Renamed the public function to avoid confusion with eval_step
// Now takes Rc<Value> to manage lifetime in the loop
pub fn evaluate_trampolined(initial_expr: Rc<Value>, initial_env: Rc<RefCell<Environment>>) -> Result<Value> {
    let mut current_expr_rc = initial_expr; // current_expr_rc now holds the Rc
    let mut current_env = initial_env;

    loop {
        // Check for stack depth / infinite loop prevention (optional)

        // Pass a reference to the Value inside the Rc to eval_step
        match eval_step(&*current_expr_rc, Rc::clone(&current_env))? {
            EvalResult::Value(v) => return Ok(v),
            EvalResult::TailCall { proc, args, env: next_env_base } => {
                match proc {
                    Value::Lambda { params, body, env: _lambda_captured_env } => {
                        if params.len() != args.len() {
                             return Err(SchemeError::Arity { expected: format!("{}", params.len()), got: args.len() });
                        }

                        let mut call_env_bindings = Environment::new_child(Rc::clone(&next_env_base));
                        for (param_name, arg_val) in params.iter().zip(args.iter()) {
                            call_env_bindings.define(param_name.clone(), arg_val.clone());
                        }

                        // --- The Fix ---
                        // Assign the Rc<Value> directly. This clones the Rc pointer (cheap)
                        // and ensures the body Value stays alive for the next iteration.
                        current_expr_rc = Rc::clone(&body); // body is already Rc<Value>
                        current_env = Rc::new(RefCell::new(call_env_bindings));
                        // Continue the loop (tail call)
                    }
                     Value::Builtin(func, _name) => {
                         // Builtins don't continue the loop; they return a final value or error.
                        return func(&args, current_env); // Pass the env the builtin runs in
                    }
                     _ => {
                        return Err(SchemeError::NotProcedure(format!("Internal Error: Tail call with non-procedure: {:?}", proc)));
                     }
                }
            }
        }
    }
}

// Keep a version matching the original signature expected by builtins like `eval`
// This function now just wraps the call to the trampolined version.
pub fn evaluate(expr: &Value, env: Rc<RefCell<Environment>>) -> Result<Value> {
    // Clone the input expression into an Rc to pass to the TCO loop
    let expr_rc = Rc::new(expr.clone());
    evaluate_trampolined(expr_rc, env)
}