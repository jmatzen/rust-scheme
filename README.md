# Rusty Scheme

A simple Scheme-like interpreter implemented in Rust, focusing on core Lisp features and demonstrating custom syntax extensions for arrays and maps.

## Overview

This project is an educational implementation of a Lisp dialect resembling Scheme. It's built using Rust and includes:

*   Traditional S-expression parsing.
*   A Read-Eval-Print Loop (REPL) for interactive use.
*   Core Scheme features like `lambda`, closures, and lexical scoping.
*   Tail Call Optimization (TCO) via trampolining.
*   **New Feature:** Array literals using `[element1, element2, ...]` syntax.
*   **New Feature:** Hash map literals using `{key1: value1, key2: value2, ...}` syntax (keys are symbols).
*   A basic set of built-in procedures.

## Features

*   **S-Expressions:** Parses standard Lisp S-expressions.
*   **REPL:** Interactive command line using `rustyline`.
*   **Data Types:** Integers, Booleans, Strings, Symbols, Lists (`()`), `Nil`.
*   **New Data Types:**
    *   **Arrays:** Fixed-size, mutable sequence using `[...]` syntax. Supports `make-array`, `array-ref`, `array-set!`, `array-length`.
    *   **Maps (HashMaps):** Key-value store using `{key: value, ...}` syntax (keys are symbols). Supports `make-map`, `map-ref`, `map-set!`, `map-keys`.
*   **Core Forms:** `quote`, `if`, `define`, `set!`, `lambda`, `begin`.
*   **Closures:** Lambdas capture their lexical environment.
*   **Tail Call Optimization:** Allows deep recursion in tail position without stack overflow.
*   **Basic Built-ins:** Arithmetic (`+`, `-`, `*`, `/`), comparisons (`=`, `<`), list operations (`cons`, `car`, `cdr`, `list`), type predicates (`integer?`, `symbol?`, `list?`, `array?`, `map?`, `procedure?`, etc.), `display`, `newline`, `equal?`, `eval`.
*   **Error Handling:** Reports parse and evaluation errors.

## Requirements

*   **Rust Toolchain:** Latest stable version recommended (developed with Rust 1.6x+). Install via [rustup](https://rustup.rs/).

## Building

Clone the repository and use Cargo to build:

```bash
git clone <your-repository-url>
cd rusty-scheme
cargo build
```

For an optimized build:

```bash
cargo build --release
```

## Running

### REPL

Start the interactive REPL:

```bash
# Debug build
./target/debug/rusty-scheme

# Release build
./target/release/rusty-scheme
```

You'll be greeted with the `λ> ` prompt. Type Scheme expressions and press Enter. Use `Ctrl+C` or `Ctrl+D` to exit.

### Executing Files (Basic)

*(Note: The current implementation primarily focuses on the REPL. A robust file loading mechanism (`load` procedure or enhanced CLI) could be added.)*

You can paste the contents of a `.scm` file into the REPL. Alternatively, a basic command-line runner could be implemented to execute a file directly.

## Language Features / Examples

```scheme
;; Basic Arithmetic and Variables
λ> (+ 10 20 5)
35
λ> (define x 100)
()
λ> (* x 3)
300

;; Conditionals
λ> (if (> x 50) "big" "small")
"big"

;; Lists
λ> (define my-list (list 1 2 3))
()
λ> (car my-list)
1
λ> (cdr my-list)
(2 3)
λ> (cons 0 my-list)
(0 1 2 3)
λ> '(a b (c d)) ; Quoted list (literal)
(a b (c d))

;; Array Literals and Functions
λ> (define my-arr [10, "hello", #t])
()
λ> my-arr
[10, "hello", #t]
λ> (array-ref my-arr 1)
"hello"
λ> (array-set! my-arr 0 99)
()
λ> my-arr
[99, "hello", #t]
λ> (array-length my-arr)
3

;; Map Literals and Functions
λ> (define my-map { name: "Bob", age: 42, active: #f, }) ; Trailing comma ok
()
λ> my-map
{name: "Bob", age: 42, active: #f}
λ> (map-ref my-map 'age)
42
λ> (map-set! my-map 'city "London")
()
λ> (map-keys my-map)
(name city age active) ; Order not guaranteed

;; Lambda (Functions) and Closures
λ> (define add (lambda (a b) (+ a b)))
()
λ> (add 5 7)
12

λ> (define make-adder (lambda (n) (lambda (x) (+ x n))))
()
λ> (define add5 (make-adder 5))
()
λ> (add5 10)
15

;; Tail Call Optimization (Factorial example)
λ> (define factorial
     (lambda (n)
       (define iter ; Inner helper function for TCO
         (lambda (i acc)
           (if (= i 0)
               acc
               (iter (- i 1) (* i acc))))) ; Tail call
       (iter n 1)))
()
λ> (factorial 5) ; Works without stack overflow
120
; λ> (factorial 10000) ; Should also work due to TCO
```

## Code Structure

The project is organized into several modules:

*   `main.rs`: Entry point, REPL loop setup.
*   `value.rs`: Defines the core `Value` enum representing all data types in the language.
*   `error.rs`: Defines the custom `SchemeError` enum and `Result` type alias.
*   `parser.rs`: Handles tokenizing and parsing text input (S-expressions, arrays, maps) into `Value` representations.
*   `env.rs`: Implements the `Environment` struct for managing variable bindings and lexical scope (using parent pointers).
*   `eval.rs`: Contains the core `evaluate` function (with TCO trampoline) and `eval_step` logic for interpreting `Value`s. Handles special forms and procedure application.
*   `builtins.rs`: Implements all the built-in procedures callable from the Scheme code.

## Testing

A test suite written *in* Rusty Scheme itself is provided in `tests.scm`. It uses a simple `assert-equal?` helper defined within the suite.

To run the tests:

1.  Build the interpreter: `cargo build`
2.  Start the REPL: `./target/debug/rusty-scheme`
3.  Copy the *entire* content of the `tests.scm` file and paste it into the REPL.

The tests will execute, printing status messages and a final summary:

```
--- Running Test Suite ---
Testing Literals and Quote...
... (other test sections) ...
Re-Testing Maps with list-length...

--- Test Suite Summary ---
Passed: <N>
Failed: 0
All tests passed!
#t
```

If any tests fail, details will be printed.

## Future Work / TODOs

*   Floating-point numbers.
*   Character data type.
*   Macros (`define-syntax`, `syntax-rules`).
*   Full numeric tower (rational, complex).
*   `load` procedure to execute code from files.
*   More robust error handling and recovery in the REPL.
*   I/O procedures (ports, `read`, `write`).
*   Continuations (`call/cc`).
*   Expand the standard library (more list utilities, string functions, etc.).

