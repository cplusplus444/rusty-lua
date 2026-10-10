Here is a comprehensive, professional **`README.md`** file tailored for your mini-Lua Rust project. You can drop this directly into your repository root alongside `Cargo.toml` and `src/main.rs`.

---

# Mini-Lua Interpreter in Rust 🦀

A lightweight, educational, single-file Lua-style expression and statement interpreter written in pure, dependency-free Rust.

This project implements a complete compiler pipeline—from source text to abstract syntax tree (AST) execution—supporting a clean subset of Lua syntax entirely in a single Rust source file.

---

## Features

* **Lexical Analysis (Lexer)**: Tokenizes source code, handling identifiers, keywords, numbers, strings (with single or double quotes), and Lua-style comments (`--`).
* **Recursive Descent Parser (Parser)**: Parses expressions with correct operator precedence (equality, comparisons, addition/subtraction, multiplication/division/modulo, and unary negation).
* **Tree-Walk Evaluator (Interpreter)**: Implements lexical scoping via a stack of symbol tables (`HashMap`), supporting variable shadowing, block isolation, and recursive environment lookups.
* **Language Constructs**:
* Variable declarations (`local x = 10`) and mutations (`x = x + 1`)
* Conditional branching (`if ... then ... elseif ... else ... end`)
* `while` loops (`while condition do ... end`)
* The built-in `print(...)` function (supporting multiple arguments separated by tabs)



---

## Project Structure

```text
mini_lua_rs/
├── Cargo.toml
└── src/
    └── main.rs    # Complete lexer, parser, AST, and interpreter in one file

```

---

## Quick Start

### 1. Create a New Rust Project

Initialize a new binary Rust package using Cargo:

```bash
cargo new mini_lua_rs
cd mini_lua_rs

```

### 2. Add the Code

Replace the contents of `src/main.rs` with the provided single-file interpreter source code.

### 3. Run the Interpreter

Execute the project:

```bash
cargo run

```

---

## Supported Language Syntax

Below is an example of a script natively executed by the mini-Lua interpreter:

```lua
-- Define a local string variable and print it
local greeting = "Hello from Rust-powered Mini-Lua!"
print(greeting)

-- Arithmetic operations
local x = 10
local y = 20
print("Sum of x and y is:", x + y)

-- Conditional branching (if-elseif-else)
if x > 15 then
    print("x is greater than 15")
elseif x == 10 then
    print("x is exactly 10!")
else
    print("x is small")
end

-- While loop
print("Starting countdown loop:")
local count = 3
while count > 0 do
    print("Countdown:", count)
    count = count - 1
end

print("Loop finished successfully!")

```

### Expected Output

When you run `cargo run`, you will see:

```text
--------------------------------------------------
EXECUTING EMBEDDED LUA SCRIPT:
--------------------------------------------------
Hello from Rust-powered Mini-Lua!
Sum of x and y is:      30
x is exactly 10!
Starting countdown loop:
Countdown:      3
Countdown:      2
Countdown:      1
Loop finished successfully!
--------------------------------------------------
INTERPRETER FINISHED EXECUTION.
--------------------------------------------------

```

---

## Architectural Breakdown

1. **Lexer (`Lexer`)**: Iterates over the character stream lazily using Rust's `Peekable<Chars>`, grouping characters into meaningful tokens.
2. **Parser (`Parser`)**: Converts tokens into an **Abstract Syntax Tree (AST)** composed of `Expr` (expressions) and `Stmt` (statements) using a clean recursive descent method.
3. **Interpreter (`Interpreter`)**: Evaluates the AST nodes dynamically, maintaining state across a nested scope stack mimicking lexical blocks.

---

## License

This project is open-source and free to use for educational and personal projects.
