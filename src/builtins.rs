use crate::env::Environment;
use crate::error::{Result, SchemeError};
use crate::eval::evaluate; // Needed for `eval` builtin
use crate::value::{Value, BuiltinFn};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// Macro to simplify arity checks
macro_rules! check_arity {
    ($args:expr, $expected:expr, $name:expr) => {
        if $args.len() != $expected {
            return Err(SchemeError::Arity { expected: format!("{}", $expected), got: $args.len() });
        }
    };
     ($args:expr, $min:expr, $max:expr, $name:expr) => {
         if $args.len() < $min || $args.len() > $max {
            return Err(SchemeError::Arity { expected: format!("between {} and {}", $min, $max), got: $args.len() });
         }
    };
    ($args:expr, >= $min:expr, $name:expr) => {
        if $args.len() < $min {
            return Err(SchemeError::Arity { expected: format!("at least {}", $min), got: $args.len() });
        }
    };
}

// Macro to extract integer arguments
macro_rules! extract_int {
    ($val:expr, $name:expr) => {
        match $val {
            Value::Integer(i) => *i,
            _ => return Err(Value::type_error("integer", $val)),
        }
    };
}


// --- Arithmetic ---
fn add(args: &[Value], _env: Rc<RefCell<Environment>>) -> Result<Value> {
    let mut sum: i64 = 0;
    for val in args {
        sum += extract_int!(val, "+");
    }
    Ok(Value::Integer(sum))
}

fn subtract(args: &[Value], _env: Rc<RefCell<Environment>>) -> Result<Value> {
    check_arity!(args, >= 1, "-");
    let first = extract_int!(&args[0], "-");
    if args.len() == 1 {
        Ok(Value::Integer(-first))
    } else {
        let mut result = first;
        for val in &args[1..] {
            result -= extract_int!(val, "-");
        }
        Ok(Value::Integer(result))
    }
}

fn multiply(args: &[Value], _env: Rc<RefCell<Environment>>) -> Result<Value> {
     let mut prod: i64 = 1;
    for val in args {
        prod *= extract_int!(val, "*");
    }
    Ok(Value::Integer(prod))
}

fn divide(args: &[Value], _env: Rc<RefCell<Environment>>) -> Result<Value> {
     check_arity!(args, >= 1, "/");
    let first = extract_int!(&args[0], "/");
     if args.len() == 1 {
         if first == 0 {
              return Err(SchemeError::Runtime("Division by zero".to_string()));
         }
         // Scheme often defines (/ x) as 1/x. Requires floats.
         // For integers, maybe error or return 0? Let's error.
         return Err(SchemeError::Arity { expected: "at least 2 for integer division".to_string(), got: 1 });
     } else {
        let mut result = first;
        for val in &args[1..] {
            let divisor = extract_int!(val, "/");
            if divisor == 0 {
                return Err(SchemeError::Runtime("Division by zero".to_string()));
            }
            result /= divisor; // Integer division
        }
        Ok(Value::Integer(result))
    }
}

// --- Comparison ---
fn equals(args: &[Value], _env: Rc<RefCell<Environment>>) -> Result<Value> {
    check_arity!(args, >= 2, "=");
     let first_val = extract_int!(&args[0], "=");
    for val in &args[1..] {
         if first_val != extract_int!(val, "=") {
             return Ok(Value::Bool(false));
         }
    }
    Ok(Value::Bool(true))
}
fn less_than(args: &[Value], _env: Rc<RefCell<Environment>>) -> Result<Value> {
     check_arity!(args, >= 2, "<");
     let mut prev = extract_int!(&args[0], "<");
     for val in &args[1..] {
         let current = extract_int!(val, "<");
         if !(prev < current) {
             return Ok(Value::Bool(false));
         }
         prev = current;
     }
    Ok(Value::Bool(true))
}
fn greater_than(args: &[Value], _env: Rc<RefCell<Environment>>) -> Result<Value> {
    check_arity!(args, >= 2, ">");
    let mut prev = extract_int!(&args[0], ">");
    for val in &args[1..] {
        let current = extract_int!(val, ">");
        if !(prev > current) {
            return Ok(Value::Bool(false));
        }
        prev = current;
    }
   Ok(Value::Bool(true))
}

// Implement >, <=, >= similarly...

// --- List Operations ---
fn cons(args: &[Value], _env: Rc<RefCell<Environment>>) -> Result<Value> {
    check_arity!(args, 2, "cons");
    let car = args[0].clone();
    let cdr = args[1].clone(); // cdr can be any value for improper lists, but usually a list or Nil

    // Ensure cdr is list-like if we only want proper lists easily representable
    match cdr {
        Value::List(mut list) => {
             list.insert(0, car);
             Ok(Value::List(list))
        }
         Value::Nil => {
             Ok(Value::List(vec![car]))
         }
        // Allow improper lists if needed: return a special Pair type or handle in List representation
         _ => Err(SchemeError::Type { expected:"list or nil".to_string(), found: cdr.type_name()}) // Or allow improper lists
    }
}

fn car(args: &[Value], _env: Rc<RefCell<Environment>>) -> Result<Value> {
    check_arity!(args, 1, "car");
    match &args[0] {
        Value::List(list) if !list.is_empty() => Ok(list[0].clone()),
        // Handle improper lists/pairs if implemented
        _ => Err(SchemeError::Type{ expected: "non-empty list".to_string(), found: args[0].type_name()}),
    }
}

fn cdr(args: &[Value], _env: Rc<RefCell<Environment>>) -> Result<Value> {
    check_arity!(args, 1, "cdr");
    match &args[0] {
        Value::List(list) if !list.is_empty() => {
            if list.len() == 1 {
                Ok(Value::Nil)
            } else {
                Ok(Value::List(list[1..].to_vec()))
            }
        }
         // Handle improper lists/pairs if implemented
        _ => Err(SchemeError::Type{ expected: "non-empty list".to_string(), found: args[0].type_name()}),
    }
}

fn list(args: &[Value], _env: Rc<RefCell<Environment>>) -> Result<Value> {
    Ok(Value::List(args.to_vec()))
}

// --- Type Predicates ---
fn is_null(args: &[Value], _env: Rc<RefCell<Environment>>) -> Result<Value> {
    check_arity!(args, 1, "null?");
    Ok(Value::Bool(matches!(args[0], Value::Nil)))
}
fn is_boolean(args: &[Value], _env: Rc<RefCell<Environment>>) -> Result<Value> {
    check_arity!(args, 1, "boolean?");
    Ok(Value::Bool(matches!(args[0], Value::Bool(_))))
}
fn is_symbol(args: &[Value], _env: Rc<RefCell<Environment>>) -> Result<Value> {
    check_arity!(args, 1, "symbol?");
    Ok(Value::Bool(matches!(args[0], Value::Symbol(_))))
}
fn is_integer(args: &[Value], _env: Rc<RefCell<Environment>>) -> Result<Value> {
     check_arity!(args, 1, "integer?"); // Or number? if floats added
    Ok(Value::Bool(matches!(args[0], Value::Integer(_))))
}
fn is_string(args: &[Value], _env: Rc<RefCell<Environment>>) -> Result<Value> {
    check_arity!(args, 1, "string?");
    Ok(Value::Bool(matches!(args[0], Value::String(_))))
}
fn is_list(args: &[Value], _env: Rc<RefCell<Environment>>) -> Result<Value> {
     check_arity!(args, 1, "list?"); // Or pair? depending on definition
    Ok(Value::Bool(matches!(args[0], Value::List(_))))
}
fn is_procedure(args: &[Value], _env: Rc<RefCell<Environment>>) -> Result<Value> {
    check_arity!(args, 1, "procedure?");
    Ok(Value::Bool(matches!(args[0], Value::Lambda{..} | Value::Builtin(..))))
}
fn is_array(args: &[Value], _env: Rc<RefCell<Environment>>) -> Result<Value> {
    check_arity!(args, 1, "array?");
    Ok(Value::Bool(matches!(args[0], Value::Array(_))))
}
fn is_map(args: &[Value], _env: Rc<RefCell<Environment>>) -> Result<Value> {
    check_arity!(args, 1, "map?");
    Ok(Value::Bool(matches!(args[0], Value::Map(_))))
}


// --- Array Functions ---
fn make_array(args: &[Value], _env: Rc<RefCell<Environment>>) -> Result<Value> {
     // (make-array k) or (make-array k fill)
    check_arity!(args, 1, 2, "make-array");
     let k = extract_int!(&args[0], "make-array") as usize;
     let fill = if args.len() == 2 { args[1].clone() } else { Value::Nil }; // Default fill
     let vec = vec![fill; k];
     Ok(Value::Array(Rc::new(RefCell::new(vec))))
}

fn array_ref(args: &[Value], _env: Rc<RefCell<Environment>>) -> Result<Value> {
     check_arity!(args, 2, "array-ref");
     let index = extract_int!(&args[1], "array-ref") as usize;
     match &args[0] {
        Value::Array(arr_rc) => {
             let arr = arr_rc.borrow();
            arr.get(index)
               .cloned()
               .ok_or_else(|| SchemeError::Runtime(format!("Array index out of bounds: {}", index)))
        }
        _ => Err(Value::type_error("array", &args[0]))
     }
}

fn array_set(args: &[Value], _env: Rc<RefCell<Environment>>) -> Result<Value> {
    check_arity!(args, 3, "array-set!");
    let index = extract_int!(&args[1], "array-set!") as usize;
    let value = args[2].clone();
     match &args[0] {
        Value::Array(arr_rc) => {
            let mut arr = arr_rc.borrow_mut(); // Mutable borrow
             if index >= arr.len() {
                return Err(SchemeError::Runtime(format!("Array index out of bounds: {}", index)));
             }
            arr[index] = value;
             Ok(Value::Nil) // Side-effecting functions often return Nil or unspecified
        }
        _ => Err(Value::type_error("array", &args[0]))
     }
}

fn array_length(args: &[Value], _env: Rc<RefCell<Environment>>) -> Result<Value> {
    check_arity!(args, 1, "array-length");
    match &args[0] {
        Value::Array(arr_rc) => {
            let len = arr_rc.borrow().len();
            Ok(Value::Integer(len as i64))
        }
         _ => Err(Value::type_error("array", &args[0]))
    }
}

// --- Map Functions ---
fn make_map(_args: &[Value], _env: Rc<RefCell<Environment>>) -> Result<Value> {
    // Could add initialization later e.g. (make-map '( (k1 v1) (k2 v2) ))
    //check_arity!(args, 0, "make-map"); // For now, just creates empty
    Ok(Value::Map(Rc::new(RefCell::new(HashMap::new()))))
}

fn map_ref(args: &[Value], _env: Rc<RefCell<Environment>>) -> Result<Value> {
    check_arity!(args, 2, "map-ref");
    let key = match &args[1] {
        Value::Symbol(s) => s.clone(),
        Value::String(s) => s.clone(), // Allow string keys too?
        _ => return Err(Value::type_error("symbol or string", &args[1])),
    };
    match &args[0] {
        Value::Map(map_rc) => {
            let map = map_rc.borrow();
            Ok(map.get(&key)
               .cloned()
               .unwrap_or(Value::Nil)) // Return Nil if key not found? Or error? Nil is safer.
                                      // Could add a third argument for default value.
        }
         _ => Err(Value::type_error("map", &args[0]))
    }
}

fn map_set(args: &[Value], _env: Rc<RefCell<Environment>>) -> Result<Value> {
     check_arity!(args, 3, "map-set!");
     let key = match &args[1] {
        Value::Symbol(s) => s.clone(),
        Value::String(s) => s.clone(),
        _ => return Err(Value::type_error("symbol or string", &args[1])),
    };
    let value = args[2].clone();
     match &args[0] {
        Value::Map(map_rc) => {
             let mut map = map_rc.borrow_mut();
            map.insert(key, value);
             Ok(Value::Nil)
        }
        _ => Err(Value::type_error("map", &args[0]))
     }
}

fn map_keys(args: &[Value], _env: Rc<RefCell<Environment>>) -> Result<Value> {
     check_arity!(args, 1, "map-keys");
     match &args[0] {
        Value::Map(map_rc) => {
             let map = map_rc.borrow();
             let keys: Vec<Value> = map.keys().map(|k| Value::Symbol(k.clone())).collect(); // Return keys as symbols
            Ok(Value::List(keys))
        }
        _ => Err(Value::type_error("map", &args[0]))
     }
}

// --- Other ---
fn display(args: &[Value], _env: Rc<RefCell<Environment>>) -> Result<Value> {
    // Basic display, prints without quotes for strings
    for (i, arg) in args.iter().enumerate() {
        if i > 0 { print!(" "); }
         match arg {
             Value::String(s) => print!("{}", s),
             _ => print!("{:?}", arg), // Use Debug formatting for others
         }
    }
    println!(); // Add newline
    Ok(Value::Nil)
}

fn newline(args: &[Value], _env: Rc<RefCell<Environment>>) -> Result<Value> {
    check_arity!(args, 0, "newline");
    println!();
    Ok(Value::Nil)
}

// Eval function (use cautiously)
fn builtin_eval(args: &[Value], env: Rc<RefCell<Environment>>) -> Result<Value> {
    check_arity!(args, 1, "eval");
    let expr_to_eval = &args[0];
    // Evaluate the expression in the *current* dynamic environment
    evaluate(expr_to_eval, env)
}

// General Equality Predicate
fn equal_q(args: &[Value], _env: Rc<RefCell<Environment>>) -> Result<Value> {
    check_arity!(args, 2, "equal?");
    // Use the derived PartialEq implementation on Value
    Ok(Value::Bool(args[0] == args[1]))
}

// Function to populate the initial environment
pub fn populate_environment(env: &mut Environment) {
    let builtins: Vec<(&str, BuiltinFn)> = vec![
        // Arithmetic
        ("+", add), ("-", subtract), ("*", multiply), ("/", divide),
        // Comparison (add more)
        ("=", equals), ("<", less_than), (">", greater_than),
        // List Ops
        ("cons", cons), ("car", car), ("cdr", cdr), ("list", list),
        // Type Predicates
        ("null?", is_null), ("boolean?", is_boolean), ("symbol?", is_symbol),
        ("integer?", is_integer), ("string?", is_string), ("list?", is_list),
        ("procedure?", is_procedure), ("array?", is_array), ("map?", is_map),
        ("equal?", equal_q),
        // Array Functions
         ("make-array", make_array), ("array-ref", array_ref), ("array-set!", array_set), ("array-length", array_length),
        // Map Functions
        ("make-map", make_map), ("map-ref", map_ref), ("map-set!", map_set), ("map-keys", map_keys),
        // Other
        ("display", display), ("newline", newline),
        ("eval", builtin_eval),
        // Constants (could be defined directly, but this is cleaner)
        // ("#t", |_args, _env| Ok(Value::Bool(true))), // Define #t/#f as vars? Usually they are literals.
        // ("#f", |_args, _env| Ok(Value::Bool(false))),
        // ("else", |_args, _env| Ok(Value::Bool(true))), // For cond special form later
    ];

    for (name, func) in builtins {
        env.define(name.to_string(), Value::Builtin(func, name.to_string()));
    }

    // Define #t and #f directly if needed, though parser handles them as literals
    // env.define("#t".to_string(), Value::Bool(true));
    // env.define("#f".to_string(), Value::Bool(false));

}