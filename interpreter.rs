//! ============================================================================
//! MINI-LUA INTERPRETER IN RUST
//! ============================================================================
//! A clean, educational, single-file interpreter implementing a subset of Lua 
//! features: dynamic variables, basic arithmetic, comparison operators, 
//! `if/elseif/else` conditions, `while` loops, and the `print` function.
//! ============================================================================

use std::collections::HashMap;
use std::io::{self, Write};
use std::iter::Peekable;
use std::str::Chars;

// ============================================================================
// 1. DATA STRUCTURES & AST (Abstract Syntax Tree)
// ============================================================================

/// Represents runtime values supported by our mini-Lua language.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Number(n) => {
                // Print integers cleanly without trailing .0 if they are whole numbers
                if n.fract() == 0.0 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::String(s) => write!(f, "{}", s),
        }
    }
}

/// Binary mathematical and logical operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

/// Expressions evaluate down to a Value.
#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Value),
    Variable(String),
    Binary(Box<Expr>, BinOp, Box<Expr>),
    Assign(String, Box<Expr>),
}

/// Statements represent actions that control execution flow or mutate state.
#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(Expr),
    Print(Vec<Expr>),
    Assignment(String, Expr),
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        elseif_branches: Vec<(Expr, Vec<Stmt>)>,
        else_branch: Option<Vec<Stmt>>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
}

// ============================================================================
// 2. LEXER (Tokenizer)
// ============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Local, If, Then, ElseIf, Else, End, While, Do, Print, True, False, Nil,
    // Literals & Identifiers
    Identifier(String),
    Number(f64),
    StringLiteral(String),
    // Operators & Punctuation
    Assign, Plus, Minus, Star, Slash, Percent,
    EqualEqual, NotEqual, Less, LessEqual, Greater, GreaterEqual,
    LParen, RParen, Comma,
    EOF,
}

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { chars: input.chars().peekable() }
    }

    /// Pulls the next token from the character stream.
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();

        let Some(&ch) = self.chars.peek() else {
            return Token::EOF;
        };

        // 1. Identifiers and Keywords
        if ch.is_alphabetic() || ch == '_' {
            let mut ident = String::new();
            while let Some(&c) = self.chars.peek() {
                if c.is_alphanumeric() || c == '_' {
                    ident.push(c);
                    self.chars.next();
                } else {
                    break;
                }
            }
            return match ident.as_str() {
                "local"  => Token::Local,
                "if"     => Token::If,
                "then"   => Token::Then,
                "elseif" => Token::ElseIf,
                "else"   => Token::Else,
                "end"    => Token::End,
                "while"  => Token::While,
                "do"     => Token::Do,
                "print"  => Token::Print,
                "true"   => Token::True,
                "false"  => Token::False,
                "nil"    => Token::Nil,
                _        => Token::Identifier(ident),
            };
        }

        // 2. Numbers (integers or decimals)
        if ch.is_ascii_digit() {
            let mut num_str = String::new();
            while let Some(&c) = self.chars.peek() {
                if c.is_ascii_digit() || c == '.' {
                    num_str.push(c);
                    self.chars.next();
                } else {
                    break;
                }
            }
            let val = num_str.parse::<f64>().unwrap_or(0.0);
            return Token::Number(val);
        }

        // 3. Strings (supports single or double quotes)
        if ch == '"' || ch == '\'' {
            let quote_char = ch;
            self.chars.next(); // Consume opening quote
            let mut s = String::new();
            while let Some(&c) = self.chars.peek() {
                if c == quote_char {
                    self.chars.next(); // Consume closing quote
                    return Token::StringLiteral(s);
                } else {
                    s.push(c);
                    self.chars.next();
                }
            }
            panic!("Unterminated string literal");
        }

        // 4. Multi-character operators and symbols
        self.chars.next(); // Consume current character for matching
        match ch {
            '=' => {
                if self.match_char('=') { Token::EqualEqual } else { Token::Assign }
            }
            '~' => {
                if self.match_char('=') { Token::NotEqual } else { panic!("Unexpected character '~'") }
            }
            '<' => {
                if self.match_char('=') { Token::LessEqual } else { Token::Less }
            }
            '>' => {
                if self.match_char('=') { Token::GreaterEqual } else { Token::Greater }
            }
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Star,
            '/' => Token::Slash,
            '%' => Token::Percent,
            '(' => Token::LParen,
            ')' => Token::RParen,
            ',' => Token::Comma,
            other => panic!("Unexpected character: '{}'", other),
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if let Some(&c) = self.chars.peek() {
            if c == expected {
                self.chars.next();
                return true;
            }
        }
        false
    }

    fn skip_whitespace_and_comments(&mut self) {
        while let Some(&c) = self.chars.peek() {
            if c.is_whitespace() {
                self.chars.next();
            } else if c == '-' {
                // Check for Lua comment style `--`
                let mut clone = self.chars.clone();
                clone.next();
                if clone.peek() == Some(&'-') {
                    self.chars.next(); // consume first '-'
                    self.chars.next(); // consume second '-'
                    // Skip until newline or EOF
                    while let Some(&nc) = self.chars.peek() {
                        if nc == '\n' {
                            break;
                        }
                        self.chars.next();
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }
}

// ============================================================================
// 3. PARSER (Recursive Descent)
// ============================================================================

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer::new(input);
        let current_token = lexer.next_token();
        Self { lexer, current_token }
    }

    fn eat(&mut self, expected: Token) {
        if std::mem::discriminant(&self.current_token) == std::mem::discriminant(&expected) {
            self.current_token = self.lexer.next_token();
        } else {
            panic!("Syntax error: expected token {:?}, got {:?}", expected, self.current_token);
        }
    }

    /// Parses a complete chunk (list of statements).
    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
        while self.current_token != Token::EOF {
            // Skip dangling semicolons if any
            statements.push(self.parse_stmt());
        }
        statements
    }

    fn parse_stmt(&mut self) -> Stmt {
        match &self.current_token {
            Token::Print => {
                self.eat(Token::Print);
                self.eat(Token::LParen);
                let mut args = Vec::new();
                if self.current_token != Token::RParen {
                    loop {
                        args.push(self.parse_expr());
                        if self.current_token == Token::Comma {
                            self.eat(Token::Comma);
                        } else {
                            break;
                        }
                    }
                }
                self.eat(Token::RParen);
                Stmt::Print(args)
            }
            Token::Local => {
                self.eat(Token::Local);
                let Token::Identifier(name) = self.current_token.clone() else {
                    panic!("Expected variable name after 'local'");
                };
                self.eat(Token::Identifier(name.clone()));
                self.eat(Token::Assign);
                let expr = self.parse_expr();
                Stmt::Assignment(name, expr)
            }
            Token::If => {
                self.eat(Token::If);
                let condition = self.parse_expr();
                self.eat(Token::Then);
                let mut then_branch = Vec::new();
                while !matches!(self.current_token, Token::ElseIf | Token::Else | Token::End | Token::EOF) {
                    then_branch.push(self.parse_stmt());
                }

                let mut elseif_branches = Vec::new();
                while self.current_token == Token::ElseIf {
                    self.eat(Token::ElseIf);
                    let ei_cond = self.parse_expr();
                    self.eat(Token::Then);
                    let mut ei_body = Vec::new();
                    while !matches!(self.current_token, Token::ElseIf | Token::Else | Token::End | Token::EOF) {
                        ei_body.push(self.parse_stmt());
                    }
                    elseif_branches.push((ei_cond, ei_body));
                }

                let mut else_branch = None;
                if self.current_token == Token::Else {
                    self.eat(Token::Else);
                    let mut el_body = Vec::new();
                    while self.current_token != Token::End && self.current_token != Token::EOF {
                        el_body.push(self.parse_stmt());
                    }
                    else_branch = Some(el_body);
                }

                self.eat(Token::End);
                Stmt::If { condition, then_branch, elseif_branches, else_branch }
            }
            Token::While => {
                self.eat(Token::While);
                let condition = self.parse_expr();
                self.eat(Token::Do);
                let mut body = Vec::new();
                while self.current_token != Token::End && self.current_token != Token::EOF {
                    body.push(self.parse_stmt());
                }
                self.eat(Token::End);
                Stmt::While { condition, body }
            }
            Token::Identifier(name) => {
                let name = name.clone();
                // Check if it's an assignment statement: var = expr
                let mut clone_lexer = self.lexer.clone();
                // Look ahead past identifier
                let next = clone_lexer.next_token();
                if next == Token::Assign {
                    self.eat(Token::Identifier(name.clone()));
                    self.eat(Token::Assign);
                    let expr = self.parse_expr();
                    Stmt::Assignment(name, expr)
                } else {
                    let expr = self.parse_expr();
                    Stmt::Expr(expr)
                }
            }
            _ => {
                let expr = self.parse_expr();
                Stmt::Expr(expr)
            }
        }
    }

    fn parse_expr(&mut self) -> Expr {
        self.parse_equality()
    }

    // Equality: ==, ~=
    fn parse_equality(&mut self) -> Expr {
        let mut expr = self.parse_comparison();
        while self.current_token == Token::EqualEqual || self.current_token == Token::NotEqual {
            let op = match self.current_token {
                Token::EqualEqual => BinOp::Equal,
                Token::NotEqual => BinOp::NotEqual,
                _ => unreachable!(),
            };
            self.eat(self.current_token.clone());
            let right = self.parse_comparison();
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }
        expr
    }

    // Comparison: <, <=, >, >=
    fn parse_comparison(&mut self) -> Expr {
        let mut expr = self.parse_term();
        while matches!(self.current_token, Token::Less | Token::LessEqual | Token::Greater | Token::GreaterEqual) {
            let op = match self.current_token {
                Token::Less => BinOp::Less,
                Token::LessEqual => BinOp::LessEqual,
                Token::Greater => BinOp::Greater,
                Token::GreaterEqual => BinOp::GreaterEqual,
                _ => unreachable!(),
            };
            self.eat(self.current_token.clone());
            let right = self.parse_term();
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }
        expr
    }

    // Term: +, -
    fn parse_term(&mut self) -> Expr {
        let mut expr = self.parse_factor();
        while self.current_token == Token::Plus || self.current_token == Token::Minus {
            let op = if self.current_token == Token::Plus { BinOp::Add } else { BinOp::Sub };
            self.eat(self.current_token.clone());
            let right = self.parse_factor();
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }
        expr
    }

    // Factor: *, /, %
    fn parse_factor(&mut self) -> Expr {
        let mut expr = self.parse_unary();
        while matches!(self.current_token, Token::Star | Token::Slash | Token::Percent) {
            let op = match self.current_token {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Mod,
                _ => unreachable!(),
            };
            self.eat(self.current_token.clone());
            let right = self.parse_unary();
            expr = Expr::Binary(Box::new(expr), op, Box::new(right));
        }
        expr
    }

    // Unary placeholders (e.g. negative numbers)
    fn parse_unary(&mut self) -> Expr {
        if self.current_token == Token::Minus {
            self.eat(Token::Minus);
            let right = self.parse_primary();
            // Desugar -expr into (0 - expr)
            Expr::Binary(
                Box::new(Expr::Literal(Value::Number(0.0))),
                BinOp::Sub,
                Box::new(right),
            )
        } else {
            self.parse_primary()
        }
    }

    // Primary literals and parenthesized expressions
    fn parse_primary(&mut self) -> Expr {
        match self.current_token.clone() {
            Token::Number(n) => {
                self.eat(Token::Number(n));
                Expr::Literal(Value::Number(n))
            }
            Token::StringLiteral(s) => {
                self.eat(Token::StringLiteral(s.clone()));
                Expr::Literal(Value::String(s))
            }
            Token::True => {
                self.eat(Token::True);
                Expr::Literal(Value::Boolean(true))
            }
            Token::False => {
                self.eat(Token::False);
                Expr::Literal(Value::Boolean(false))
            }
            Token::Nil => {
                self.eat(Token::Nil);
                Expr::Literal(Value::Nil)
            }
            Token::Identifier(name) => {
                self.eat(Token::Identifier(name.clone()));
                Expr::Variable(name)
            }
            Token::LParen => {
                self.eat(Token::LParen);
                let expr = self.parse_expr();
                self.eat(Token::RParen);
                expr
            }
            other => panic!("Unexpected token in expression: {:?}", other),
        }
    }
}

// ============================================================================
// 4. EVALUATOR (Environment & Tree-Walk Interpreter)
// ============================================================================

pub struct Interpreter {
    // Scope hierarchy stack simulating Lua environments
    scopes: Vec<HashMap<String, Value>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()], // Global scope
        }
    }

    pub fn interpret(&mut self, statements: &[Stmt]) {
        for stmt in statements {
            self.eval_stmt(stmt);
        }
    }

    fn get_variable(&self, name: &str) -> Value {
        // Search scopes from innermost (local) out to global
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.get(name) {
                return val.clone();
            }
        }
        panic!("Runtime error: undefined variable '{}'", name);
    }

    fn set_variable(&mut self, name: &str, value: Value) {
        // Assign to existing variable in closest scope if it exists
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return;
            }
        }
        // Otherwise default to updating/creating in the current active scope
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), value);
        }
    }

    fn eval_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expr(expr) => {
                self.eval_expr(expr);
            }
            Stmt::Print(args) => {
                let evaluated: Vec<String> = args.iter().map(|arg| self.eval_expr(arg).to_string()).collect();
                println!("{}", evaluated.join("\t"));
            }
            Stmt::Assignment(name, expr) => {
                let val = self.eval_expr(expr);
                self.set_variable(name, val);
            }
            Stmt::If { condition, then_branch, elseif_branches, else_branch } => {
                let cond_val = self.eval_expr(condition);
                if self.is_truthy(&cond_val) {
                    self.execute_block(then_branch);
                } else {
                    let mut executed = false;
                    for (ei_cond, ei_body) in elseif_branches {
                        let val = self.eval_expr(ei_cond);
                        if self.is_truthy(&val) {
                            self.execute_block(ei_body);
                            executed = true;
                            break;
                        }
                    }
                    if !executed {
                        if let Some(el_body) = else_branch {
                            self.execute_block(el_body);
                        }
                    }
                }
            }
            Stmt::While { condition, body } => {
                while self.is_truthy(&self.eval_expr(condition)) {
                    self.execute_block(body);
                }
            }
        }
    }

    fn execute_block(&mut self, statements: &[Stmt]) {
        // Push a fresh nested scope block
        self.scopes.push(HashMap::new());
        for stmt in statements {
            self.eval_stmt(stmt);
        }
        self.scopes.pop();
    }

    fn is_truthy(&self, val: &Value) -> bool {
        match val {
            Value::Nil => false,
            Value::Boolean(b) => *b,
            // In Lua, all numbers and strings are truthy (unlike some languages where 0 is falsy)
            _ => true,
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> Value {
        match expr {
            Expr::Literal(val) => val.clone(),
            Expr::Variable(name) => self.get_variable(name),
            Expr::Binary(left, op, right) => {
                let l_val = self.eval_expr(left);
                let r_val = self.eval_expr(right);
                self.eval_binary_op(&l_val, *op, &r_val)
            }
            Expr::Assign(name, expr) => {
                let val = self.eval_expr(expr);
                self.set_variable(name, val.clone());
                val
            }
        }
    }

    fn eval_binary_op(&self, left: &Value, op: BinOp, right: &Value) -> Value {
        match (left, op, right) {
            // Numeric operations
            (Value::Number(l), BinOp::Add, Value::Number(r)) => Value::Number(l + r),
            (Value::Number(l), BinOp::Sub, Value::Number(r)) => Value::Number(l - r),
            (Value::Number(l), BinOp::Mul, Value::Number(r)) => Value::Number(l * r),
            (Value::Number(l), BinOp::Div, Value::Number(r)) => Value::Number(l / r),
            (Value::Number(l), BinOp::Mod, Value::Number(r)) => Value::Number(l % r),

            // Comparisons
            (Value::Number(l), BinOp::Less, Value::Number(r)) => Value::Boolean(l < r),
            (Value::Number(l), BinOp::LessEqual, Value::Number(r)) => Value::Boolean(l <= r),
            (Value::Number(l), BinOp::Greater, Value::Number(r)) => Value::Boolean(l > r),
            (Value::Number(l), BinOp::GreaterEqual, Value::Number(r)) => Value::Boolean(l >= r),

            // Universal equality checks
            (_, BinOp::Equal, _) => Value::Boolean(left == right),
            (_, BinOp::NotEqual, _) => Value::Boolean(left != right),

            _ => panic!("Runtime type error: invalid operation between {:?} and {:?}", left, right),
        }
    }
}

// ============================================================================
// 5. MAIN ENTRY POINT & INTEGRATION TEST
// ============================================================================

fn main() {
    // Demonstration Lua script exercising core mini-Lua features
    let script = r#"
        -- Welcome script in Mini-Lua written in Rust!
        local greeting = "Hello from Rust-powered Mini-Lua!"
        print(greeting)

        local x = 10
        local y = 20
        print("Sum of x and y is:", x + y)

        -- Conditional tests
        if x > 15 then
            print("x is greater than 15")
        elseif x == 10 then
            print("x is exactly 10!")
        else
            print("x is small")
        end

        -- While loop test (Counting down)
        print("Starting countdown loop:")
        local count = 3
        while count > 0 do
            print("Countdown:", count)
            count = count - 1
        end
        print("Loop finished successfully!")
    "#;

    println!("--------------------------------------------------");
    println!("EXECUTING EMBEDDED LUA SCRIPT:");
    println!("--------------------------------------------------");

    let mut parser = Parser::new(script);
    let ast = parser.parse();

    let mut interpreter = Interpreter::new();
    interpreter.interpret(&ast);

    println!("--------------------------------------------------");
    println!("INTERPRETER FINISHED EXECUTION.");
    println!("--------------------------------------------------");
}
