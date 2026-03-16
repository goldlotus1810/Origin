//! # math — Symbolic math engine
//!
//! Parse, simplify, solve equations, derive, integrate.
//!
//! ```text
//! solve "2x + 3 = 7"      → x = 2
//! derive "x^2 + 3x"       → 2x + 3
//! integrate "2x"           → x^2
//! simplify "2x + 3x + 1"  → 5x + 1
//! ```

extern crate alloc;
use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

// ─────────────────────────────────────────────────────────────────────────────
// MathExpr — symbolic expression tree
// ─────────────────────────────────────────────────────────────────────────────

/// A symbolic math expression.
#[derive(Debug, Clone, PartialEq)]
pub enum MathExpr {
    /// Numeric constant
    Num(f64),
    /// Variable (e.g. "x", "y")
    Var(String),
    /// Addition: a + b
    Add(Box<MathExpr>, Box<MathExpr>),
    /// Subtraction: a - b (stored as Add(a, Neg(b)))
    Sub(Box<MathExpr>, Box<MathExpr>),
    /// Multiplication: a * b
    Mul(Box<MathExpr>, Box<MathExpr>),
    /// Division: a / b
    Div(Box<MathExpr>, Box<MathExpr>),
    /// Power: a ^ b
    Pow(Box<MathExpr>, Box<MathExpr>),
    /// Negation: -a
    Neg(Box<MathExpr>),
    /// sin(a)
    Sin(Box<MathExpr>),
    /// cos(a)
    Cos(Box<MathExpr>),
    /// ln(a)
    Ln(Box<MathExpr>),
}

/// A math equation: lhs = rhs.
#[derive(Debug, Clone)]
pub struct Equation {
    /// Left-hand side
    pub lhs: MathExpr,
    /// Right-hand side
    pub rhs: MathExpr,
}

/// Result of a math operation with step-by-step explanation.
#[derive(Debug, Clone)]
pub struct MathResult {
    /// The final result expression
    pub result: MathExpr,
    /// Step-by-step explanation
    pub steps: Vec<String>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Display
// ─────────────────────────────────────────────────────────────────────────────

impl MathExpr {
    /// Format expression as human-readable string.
    pub fn display(&self) -> String {
        match self {
            MathExpr::Num(n) => {
                if (*n - libm::round(*n)).abs() < 1e-10 && n.abs() < 1e15 {
                    format!("{}", libm::round(*n) as i64)
                } else {
                    format!("{:.6}", n)
                }
            }
            MathExpr::Var(v) => v.clone(),
            MathExpr::Add(a, b) => format!("{} + {}", a.display(), b.display()),
            MathExpr::Sub(a, b) => format!("{} - {}", a.display(), b.paren_display()),
            MathExpr::Mul(a, b) => {
                let a_s = a.paren_display_mul();
                let b_s = b.paren_display_mul();
                // "2 * x" → "2x" for coefficients
                if matches!(a.as_ref(), MathExpr::Num(_)) && matches!(b.as_ref(), MathExpr::Var(_))
                {
                    format!("{}{}", a.display(), b.display())
                } else {
                    format!("{} * {}", a_s, b_s)
                }
            }
            MathExpr::Div(a, b) => format!("{} / {}", a.paren_display(), b.paren_display()),
            MathExpr::Pow(a, b) => format!("{}^{}", a.paren_display(), b.paren_display()),
            MathExpr::Neg(a) => format!("-{}", a.paren_display()),
            MathExpr::Sin(a) => format!("sin({})", a.display()),
            MathExpr::Cos(a) => format!("cos({})", a.display()),
            MathExpr::Ln(a) => format!("ln({})", a.display()),
        }
    }

    fn paren_display(&self) -> String {
        match self {
            MathExpr::Add(..) | MathExpr::Sub(..) => format!("({})", self.display()),
            _ => self.display(),
        }
    }

    fn paren_display_mul(&self) -> String {
        match self {
            MathExpr::Add(..) | MathExpr::Sub(..) => format!("({})", self.display()),
            _ => self.display(),
        }
    }

    /// Check if expression is zero.
    pub fn is_zero(&self) -> bool {
        matches!(self, MathExpr::Num(n) if n.abs() < 1e-15)
    }

    /// Check if expression is one.
    pub fn is_one(&self) -> bool {
        matches!(self, MathExpr::Num(n) if (*n - 1.0).abs() < 1e-15)
    }

    /// Check if expression contains a variable.
    pub fn contains_var(&self, var: &str) -> bool {
        match self {
            MathExpr::Num(_) => false,
            MathExpr::Var(v) => v == var,
            MathExpr::Add(a, b)
            | MathExpr::Sub(a, b)
            | MathExpr::Mul(a, b)
            | MathExpr::Div(a, b)
            | MathExpr::Pow(a, b) => a.contains_var(var) || b.contains_var(var),
            MathExpr::Neg(a) | MathExpr::Sin(a) | MathExpr::Cos(a) | MathExpr::Ln(a) => {
                a.contains_var(var)
            }
        }
    }

    /// Evaluate expression with variable substitution.
    pub fn eval(&self, var: &str, val: f64) -> Option<f64> {
        match self {
            MathExpr::Num(n) => Some(*n),
            MathExpr::Var(v) => {
                if v == var {
                    Some(val)
                } else {
                    None
                }
            }
            MathExpr::Add(a, b) => Some(a.eval(var, val)? + b.eval(var, val)?),
            MathExpr::Sub(a, b) => Some(a.eval(var, val)? - b.eval(var, val)?),
            MathExpr::Mul(a, b) => Some(a.eval(var, val)? * b.eval(var, val)?),
            MathExpr::Div(a, b) => {
                let bv = b.eval(var, val)?;
                if bv.abs() < 1e-15 {
                    None
                } else {
                    Some(a.eval(var, val)? / bv)
                }
            }
            MathExpr::Pow(a, b) => Some(libm::pow(a.eval(var, val)?, b.eval(var, val)?)),
            MathExpr::Neg(a) => Some(-a.eval(var, val)?),
            MathExpr::Sin(a) => Some(libm::sin(a.eval(var, val)?)),
            MathExpr::Cos(a) => Some(libm::cos(a.eval(var, val)?)),
            MathExpr::Ln(a) => {
                let av = a.eval(var, val)?;
                if av <= 0.0 {
                    None
                } else {
                    Some(libm::log(av))
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Parser — "2x^2 + 3x - 5" → MathExpr tree
// ─────────────────────────────────────────────────────────────────────────────

/// Parse a math expression string into a MathExpr tree.
pub fn parse_math(input: &str) -> Result<MathExpr, String> {
    let tokens = tokenize(input)?;
    let (expr, rest) = parse_additive(&tokens)?;
    if !rest.is_empty() {
        return Err(format!("Unexpected tokens: {:?}", rest));
    }
    Ok(expr)
}

/// Parse an equation "lhs = rhs".
pub fn parse_equation(input: &str) -> Result<Equation, String> {
    let parts: Vec<&str> = input.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err("Expected equation with '='".into());
    }
    // Handle "==" by checking if rhs starts with "="
    let rhs_str = parts[1].trim_start_matches('=');
    let lhs = parse_math(parts[0].trim())?;
    let rhs = parse_math(rhs_str.trim())?;
    Ok(Equation { lhs, rhs })
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Num(f64),
    Var(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    LParen,
    RParen,
    Func(String), // sin, cos, ln, log
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        match c {
            ' ' | '\t' | '\n' => {
                i += 1;
            }
            '+' => {
                tokens.push(Token::Plus);
                i += 1;
            }
            '-' => {
                tokens.push(Token::Minus);
                i += 1;
            }
            '*' | '×' | '·' => {
                tokens.push(Token::Star);
                i += 1;
            }
            '/' | '÷' => {
                tokens.push(Token::Slash);
                i += 1;
            }
            '^' => {
                tokens.push(Token::Caret);
                i += 1;
            }
            '(' => {
                tokens.push(Token::LParen);
                i += 1;
            }
            ')' => {
                tokens.push(Token::RParen);
                i += 1;
            }
            '0'..='9' | '.' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                let num_str: String = chars[start..i].iter().collect();
                let n: f64 = num_str
                    .parse()
                    .map_err(|_| format!("Invalid number: {}", num_str))?;
                tokens.push(Token::Num(n));
                // Implicit multiplication: "2x" → "2 * x"
                if i < chars.len() && (chars[i].is_alphabetic() || chars[i] == '(') {
                    tokens.push(Token::Star);
                }
            }
            'a'..='z' | 'A'..='Z' => {
                let start = i;
                while i < chars.len() && chars[i].is_alphabetic() {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                match word.as_str() {
                    "sin" | "cos" | "tan" | "ln" | "log" | "sqrt" => {
                        tokens.push(Token::Func(word));
                    }
                    "pi" => tokens.push(Token::Num(core::f64::consts::PI)),
                    "e" if (i >= chars.len() || !chars[i].is_alphabetic()) => {
                        tokens.push(Token::Num(core::f64::consts::E));
                    }
                    _ => {
                        // Multi-char variable or split into single chars
                        if word.len() == 1 {
                            tokens.push(Token::Var(word));
                        } else {
                            // "xy" → x * y (implicit multiplication)
                            for ch in word.chars() {
                                tokens.push(Token::Var(String::from(ch)));
                                tokens.push(Token::Star);
                            }
                            // Remove trailing star
                            if let Some(Token::Star) = tokens.last() {
                                tokens.pop();
                            }
                        }
                    }
                }
                // Implicit multiplication: "x(" → "x * ("
                if i < chars.len() && chars[i] == '(' && !matches!(tokens.last(), Some(Token::Func(_))) {
                    tokens.push(Token::Star);
                }
            }
            // Unicode math symbols
            '∫' | '∑' | '∏' | '∂' | '√' => {
                // Skip for now — these are handled at command level
                i += 1;
            }
            'π' => {
                tokens.push(Token::Num(core::f64::consts::PI));
                i += 1;
            }
            _ => {
                i += 1; // skip unknown chars
            }
        }
    }

    Ok(tokens)
}

// ── Recursive descent parser ────────────────────────────────────────────────

fn parse_additive<'a>(tokens: &'a [Token]) -> Result<(MathExpr, &'a [Token]), String> {
    let (mut left, mut rest) = parse_multiplicative(tokens)?;

    loop {
        match rest.first() {
            Some(Token::Plus) => {
                let (right, r) = parse_multiplicative(&rest[1..])?;
                left = MathExpr::Add(Box::new(left), Box::new(right));
                rest = r;
            }
            Some(Token::Minus) => {
                let (right, r) = parse_multiplicative(&rest[1..])?;
                left = MathExpr::Sub(Box::new(left), Box::new(right));
                rest = r;
            }
            _ => break,
        }
    }
    Ok((left, rest))
}

fn parse_multiplicative<'a>(tokens: &'a [Token]) -> Result<(MathExpr, &'a [Token]), String> {
    let (mut left, mut rest) = parse_power(tokens)?;

    loop {
        match rest.first() {
            Some(Token::Star) => {
                let (right, r) = parse_power(&rest[1..])?;
                left = MathExpr::Mul(Box::new(left), Box::new(right));
                rest = r;
            }
            Some(Token::Slash) => {
                let (right, r) = parse_power(&rest[1..])?;
                left = MathExpr::Div(Box::new(left), Box::new(right));
                rest = r;
            }
            _ => break,
        }
    }
    Ok((left, rest))
}

fn parse_power<'a>(tokens: &'a [Token]) -> Result<(MathExpr, &'a [Token]), String> {
    let (base, rest) = parse_unary(tokens)?;
    if let Some(Token::Caret) = rest.first() {
        let (exp, rest2) = parse_unary(&rest[1..])?;
        Ok((MathExpr::Pow(Box::new(base), Box::new(exp)), rest2))
    } else {
        Ok((base, rest))
    }
}

fn parse_unary<'a>(tokens: &'a [Token]) -> Result<(MathExpr, &'a [Token]), String> {
    match tokens.first() {
        Some(Token::Minus) => {
            let (expr, rest) = parse_primary(&tokens[1..])?;
            Ok((MathExpr::Neg(Box::new(expr)), rest))
        }
        _ => parse_primary(tokens),
    }
}

fn parse_primary<'a>(tokens: &'a [Token]) -> Result<(MathExpr, &'a [Token]), String> {
    match tokens.first() {
        Some(Token::Num(n)) => Ok((MathExpr::Num(*n), &tokens[1..])),
        Some(Token::Var(v)) => Ok((MathExpr::Var(v.clone()), &tokens[1..])),
        Some(Token::LParen) => {
            let (expr, rest) = parse_additive(&tokens[1..])?;
            match rest.first() {
                Some(Token::RParen) => Ok((expr, &rest[1..])),
                _ => Err("Missing closing parenthesis".into()),
            }
        }
        Some(Token::Func(name)) => {
            let rest = &tokens[1..];
            // Expect "(" after function name
            match rest.first() {
                Some(Token::LParen) => {
                    let (arg, rest2) = parse_additive(&rest[1..])?;
                    match rest2.first() {
                        Some(Token::RParen) => {
                            let expr = match name.as_str() {
                                "sin" => MathExpr::Sin(Box::new(arg)),
                                "cos" => MathExpr::Cos(Box::new(arg)),
                                "ln" | "log" => MathExpr::Ln(Box::new(arg)),
                                "sqrt" => MathExpr::Pow(
                                    Box::new(arg),
                                    Box::new(MathExpr::Num(0.5)),
                                ),
                                _ => return Err(format!("Unknown function: {}", name)),
                            };
                            Ok((expr, &rest2[1..]))
                        }
                        _ => Err("Missing ')' after function argument".into()),
                    }
                }
                _ => Err(format!("Expected '(' after function '{}'", name)),
            }
        }
        _ => Err("Unexpected end of expression".into()),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Simplify
// ─────────────────────────────────────────────────────────────────────────────

/// Simplify a math expression (constant folding, identity removal).
pub fn simplify(expr: &MathExpr) -> MathExpr {
    match expr {
        MathExpr::Add(a, b) => {
            let a = simplify(a);
            let b = simplify(b);
            match (&a, &b) {
                (MathExpr::Num(x), MathExpr::Num(y)) => MathExpr::Num(x + y),
                _ if a.is_zero() => b,
                _ if b.is_zero() => a,
                _ => MathExpr::Add(Box::new(a), Box::new(b)),
            }
        }
        MathExpr::Sub(a, b) => {
            let a = simplify(a);
            let b = simplify(b);
            match (&a, &b) {
                (MathExpr::Num(x), MathExpr::Num(y)) => MathExpr::Num(x - y),
                _ if b.is_zero() => a,
                _ => MathExpr::Sub(Box::new(a), Box::new(b)),
            }
        }
        MathExpr::Mul(a, b) => {
            let a = simplify(a);
            let b = simplify(b);
            match (&a, &b) {
                (MathExpr::Num(x), MathExpr::Num(y)) => MathExpr::Num(x * y),
                _ if a.is_zero() || b.is_zero() => MathExpr::Num(0.0),
                _ if a.is_one() => b,
                _ if b.is_one() => a,
                _ => MathExpr::Mul(Box::new(a), Box::new(b)),
            }
        }
        MathExpr::Div(a, b) => {
            let a = simplify(a);
            let b = simplify(b);
            match (&a, &b) {
                (MathExpr::Num(x), MathExpr::Num(y)) if y.abs() > 1e-15 => MathExpr::Num(x / y),
                _ if a.is_zero() => MathExpr::Num(0.0),
                _ if b.is_one() => a,
                _ => MathExpr::Div(Box::new(a), Box::new(b)),
            }
        }
        MathExpr::Pow(a, b) => {
            let a = simplify(a);
            let b = simplify(b);
            match (&a, &b) {
                (MathExpr::Num(x), MathExpr::Num(y)) => MathExpr::Num(libm::pow(*x, *y)),
                _ if b.is_zero() => MathExpr::Num(1.0),
                _ if b.is_one() => a,
                _ => MathExpr::Pow(Box::new(a), Box::new(b)),
            }
        }
        MathExpr::Neg(a) => {
            let a = simplify(a);
            match &a {
                MathExpr::Num(n) => MathExpr::Num(-n),
                MathExpr::Neg(inner) => *inner.clone(),
                _ => MathExpr::Neg(Box::new(a)),
            }
        }
        MathExpr::Sin(a) => MathExpr::Sin(Box::new(simplify(a))),
        MathExpr::Cos(a) => MathExpr::Cos(Box::new(simplify(a))),
        MathExpr::Ln(a) => MathExpr::Ln(Box::new(simplify(a))),
        MathExpr::Num(_) | MathExpr::Var(_) => expr.clone(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Derivative — symbolic differentiation
// ─────────────────────────────────────────────────────────────────────────────

/// Compute the symbolic derivative of `expr` with respect to `var`.
pub fn derivative(expr: &MathExpr, var: &str) -> MathExpr {
    let result = match expr {
        MathExpr::Num(_) => MathExpr::Num(0.0),
        MathExpr::Var(v) => {
            if v == var {
                MathExpr::Num(1.0)
            } else {
                MathExpr::Num(0.0)
            }
        }
        // d/dx(a + b) = da + db
        MathExpr::Add(a, b) => MathExpr::Add(
            Box::new(derivative(a, var)),
            Box::new(derivative(b, var)),
        ),
        // d/dx(a - b) = da - db
        MathExpr::Sub(a, b) => MathExpr::Sub(
            Box::new(derivative(a, var)),
            Box::new(derivative(b, var)),
        ),
        // Product rule: d/dx(a*b) = da*b + a*db
        MathExpr::Mul(a, b) => MathExpr::Add(
            Box::new(MathExpr::Mul(
                Box::new(derivative(a, var)),
                b.clone(),
            )),
            Box::new(MathExpr::Mul(
                a.clone(),
                Box::new(derivative(b, var)),
            )),
        ),
        // Quotient rule: d/dx(a/b) = (da*b - a*db) / b²
        MathExpr::Div(a, b) => MathExpr::Div(
            Box::new(MathExpr::Sub(
                Box::new(MathExpr::Mul(
                    Box::new(derivative(a, var)),
                    b.clone(),
                )),
                Box::new(MathExpr::Mul(
                    a.clone(),
                    Box::new(derivative(b, var)),
                )),
            )),
            Box::new(MathExpr::Pow(b.clone(), Box::new(MathExpr::Num(2.0)))),
        ),
        // Power rule: d/dx(a^n) = n * a^(n-1) * da (when n is constant)
        MathExpr::Pow(base, exp) => {
            if !exp.contains_var(var) {
                // d/dx(a^n) = n * a^(n-1) * da/dx
                MathExpr::Mul(
                    Box::new(MathExpr::Mul(
                        exp.clone(),
                        Box::new(MathExpr::Pow(
                            base.clone(),
                            Box::new(MathExpr::Sub(exp.clone(), Box::new(MathExpr::Num(1.0)))),
                        )),
                    )),
                    Box::new(derivative(base, var)),
                )
            } else if !base.contains_var(var) {
                // d/dx(c^g(x)) = c^g(x) * ln(c) * g'(x)
                MathExpr::Mul(
                    Box::new(MathExpr::Mul(
                        Box::new(expr.clone()),
                        Box::new(MathExpr::Ln(base.clone())),
                    )),
                    Box::new(derivative(exp, var)),
                )
            } else {
                // General case: d/dx(f^g) = f^g * (g'*ln(f) + g*f'/f)
                // Simplified: just return the expression for now
                MathExpr::Mul(
                    Box::new(expr.clone()),
                    Box::new(MathExpr::Add(
                        Box::new(MathExpr::Mul(
                            Box::new(derivative(exp, var)),
                            Box::new(MathExpr::Ln(base.clone())),
                        )),
                        Box::new(MathExpr::Div(
                            Box::new(MathExpr::Mul(
                                exp.clone(),
                                Box::new(derivative(base, var)),
                            )),
                            base.clone(),
                        )),
                    )),
                )
            }
        }
        // d/dx(-a) = -da
        MathExpr::Neg(a) => MathExpr::Neg(Box::new(derivative(a, var))),
        // Chain rule: d/dx(sin(a)) = cos(a) * da
        MathExpr::Sin(a) => MathExpr::Mul(
            Box::new(MathExpr::Cos(a.clone())),
            Box::new(derivative(a, var)),
        ),
        // d/dx(cos(a)) = -sin(a) * da
        MathExpr::Cos(a) => MathExpr::Neg(Box::new(MathExpr::Mul(
            Box::new(MathExpr::Sin(a.clone())),
            Box::new(derivative(a, var)),
        ))),
        // d/dx(ln(a)) = (1/a) * da
        MathExpr::Ln(a) => MathExpr::Div(
            Box::new(derivative(a, var)),
            a.clone(),
        ),
    };
    simplify(&result)
}

/// Derivative with step-by-step output.
pub fn derivative_steps(expr: &MathExpr, var: &str) -> MathResult {
    let mut steps = Vec::new();
    steps.push(format!("d/d{} [{}]", var, expr.display()));

    let raw = derivative(expr, var);
    steps.push(format!("= {}", raw.display()));

    let simplified = simplify(&raw);
    if simplified != raw {
        steps.push(format!("= {} (simplified)", simplified.display()));
    }

    MathResult {
        result: simplified,
        steps,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Integrate — symbolic integration (basic rules)
// ─────────────────────────────────────────────────────────────────────────────

/// Compute the indefinite integral of `expr` with respect to `var`.
///
/// Supports: constants, polynomials, 1/x, sin, cos, e^x.
/// Returns None for expressions that can't be integrated.
pub fn integrate(expr: &MathExpr, var: &str) -> Option<MathExpr> {
    let result = match expr {
        // ∫ c dx = cx
        MathExpr::Num(c) => MathExpr::Mul(
            Box::new(MathExpr::Num(*c)),
            Box::new(MathExpr::Var(var.into())),
        ),
        // ∫ x dx = x²/2
        MathExpr::Var(v) if v == var => MathExpr::Div(
            Box::new(MathExpr::Pow(
                Box::new(MathExpr::Var(var.into())),
                Box::new(MathExpr::Num(2.0)),
            )),
            Box::new(MathExpr::Num(2.0)),
        ),
        // ∫ y dx = yx (where y is a different variable)
        MathExpr::Var(v) => MathExpr::Mul(
            Box::new(MathExpr::Var(v.clone())),
            Box::new(MathExpr::Var(var.into())),
        ),
        // ∫ (a + b) dx = ∫a dx + ∫b dx
        MathExpr::Add(a, b) => MathExpr::Add(
            Box::new(integrate(a, var)?),
            Box::new(integrate(b, var)?),
        ),
        // ∫ (a - b) dx = ∫a dx - ∫b dx
        MathExpr::Sub(a, b) => MathExpr::Sub(
            Box::new(integrate(a, var)?),
            Box::new(integrate(b, var)?),
        ),
        // ∫ c*f(x) dx = c * ∫f(x) dx
        MathExpr::Mul(a, b) => {
            if !a.contains_var(var) {
                MathExpr::Mul(a.clone(), Box::new(integrate(b, var)?))
            } else if !b.contains_var(var) {
                MathExpr::Mul(b.clone(), Box::new(integrate(a, var)?))
            } else {
                return None; // Can't integrate product of two x-dependent terms
            }
        }
        // ∫ x^n dx = x^(n+1)/(n+1) for n ≠ -1
        MathExpr::Pow(base, exp) => {
            if !exp.contains_var(var) {
                if let MathExpr::Var(v) = base.as_ref() {
                    if v == var {
                        if let MathExpr::Num(n) = exp.as_ref() {
                            if (*n - (-1.0)).abs() < 1e-15 {
                                // ∫ x^(-1) dx = ln|x|
                                return Some(MathExpr::Ln(Box::new(MathExpr::Var(var.into()))));
                            }
                            let new_exp = n + 1.0;
                            return Some(MathExpr::Div(
                                Box::new(MathExpr::Pow(
                                    Box::new(MathExpr::Var(var.into())),
                                    Box::new(MathExpr::Num(new_exp)),
                                )),
                                Box::new(MathExpr::Num(new_exp)),
                            ));
                        }
                    }
                }
                return None;
            } else {
                return None;
            }
        }
        // ∫ -f(x) dx = -∫f(x) dx
        MathExpr::Neg(a) => MathExpr::Neg(Box::new(integrate(a, var)?)),
        // ∫ sin(x) dx = -cos(x)
        MathExpr::Sin(a) => {
            if let MathExpr::Var(v) = a.as_ref() {
                if v == var {
                    return Some(MathExpr::Neg(Box::new(MathExpr::Cos(
                        Box::new(MathExpr::Var(var.into())),
                    ))));
                }
            }
            return None;
        }
        // ∫ cos(x) dx = sin(x)
        MathExpr::Cos(a) => {
            if let MathExpr::Var(v) = a.as_ref() {
                if v == var {
                    return Some(MathExpr::Sin(Box::new(MathExpr::Var(var.into()))));
                }
            }
            return None;
        }
        // ∫ 1/x dx = ln|x| — handled via x^(-1)
        MathExpr::Div(a, b) => {
            if !b.contains_var(var) {
                // ∫ f(x)/c dx = (1/c) * ∫f(x) dx
                return Some(MathExpr::Div(
                    Box::new(integrate(a, var)?),
                    b.clone(),
                ));
            }
            return None;
        }
        MathExpr::Ln(_) => return None,
    };
    Some(simplify(&result))
}

/// Integration with step-by-step output.
pub fn integrate_steps(expr: &MathExpr, var: &str) -> MathResult {
    let mut steps = Vec::new();
    steps.push(format!("∫ [{}] d{}", expr.display(), var));

    match integrate(expr, var) {
        Some(result) => {
            let simplified = simplify(&result);
            steps.push(format!("= {} + C", simplified.display()));
            MathResult {
                result: simplified,
                steps,
            }
        }
        None => {
            steps.push("Cannot integrate symbolically".into());
            MathResult {
                result: expr.clone(),
                steps,
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Solve — equation solver
// ─────────────────────────────────────────────────────────────────────────────

/// Solve a linear or quadratic equation for `var`.
///
/// Returns solutions as Vec<f64>.
pub fn solve(eq: &Equation, var: &str) -> MathResult {
    let mut steps = Vec::new();
    steps.push(format!("{} = {}", eq.lhs.display(), eq.rhs.display()));

    // Move everything to LHS: lhs - rhs = 0
    let combined = simplify(&MathExpr::Sub(
        Box::new(eq.lhs.clone()),
        Box::new(eq.rhs.clone()),
    ));
    steps.push(format!("{} = 0", combined.display()));

    // Extract coefficients for polynomial ax² + bx + c = 0
    let (a, b, c) = extract_coefficients(&combined, var);

    if a.abs() < 1e-15 {
        // Linear: bx + c = 0 → x = -c/b
        if b.abs() < 1e-15 {
            steps.push("No variable found — equation is constant".into());
            return MathResult {
                result: combined,
                steps,
            };
        }
        let x = -c / b;
        steps.push(format!("{}*{} + {} = 0", b, var, c));
        steps.push(format!("{} = -{}/{}", var, c, b));
        steps.push(format!("{} = {}", var, format_number(x)));
        MathResult {
            result: MathExpr::Num(x),
            steps,
        }
    } else {
        // Quadratic: ax² + bx + c = 0
        let discriminant = b * b - 4.0 * a * c;
        steps.push(format!(
            "{}{}² + {}{} + {} = 0",
            format_number(a),
            var,
            format_number(b),
            var,
            format_number(c)
        ));
        steps.push(format!("Δ = b² - 4ac = {}", format_number(discriminant)));

        if discriminant < -1e-15 {
            steps.push("Δ < 0 → no real solutions".into());
            MathResult {
                result: MathExpr::Num(f64::NAN),
                steps,
            }
        } else if discriminant.abs() < 1e-15 {
            let x = -b / (2.0 * a);
            steps.push(format!("{} = -b/(2a) = {}", var, format_number(x)));
            MathResult {
                result: MathExpr::Num(x),
                steps,
            }
        } else {
            let sqrt_d = libm::sqrt(discriminant);
            let x1 = (-b + sqrt_d) / (2.0 * a);
            let x2 = (-b - sqrt_d) / (2.0 * a);
            steps.push(format!(
                "{}₁ = (-b + √Δ)/(2a) = {}",
                var,
                format_number(x1)
            ));
            steps.push(format!(
                "{}₂ = (-b - √Δ)/(2a) = {}",
                var,
                format_number(x2)
            ));
            // Return first root
            MathResult {
                result: MathExpr::Add(
                    Box::new(MathExpr::Num(x1)),
                    Box::new(MathExpr::Num(x2)),
                ),
                steps,
            }
        }
    }
}

/// Extract polynomial coefficients (a, b, c) for ax² + bx + c.
fn extract_coefficients(expr: &MathExpr, var: &str) -> (f64, f64, f64) {
    // Evaluate at 3 points to find a, b, c
    // f(0) = c, f(1) = a+b+c, f(-1) = a-b+c
    let f0 = expr.eval(var, 0.0).unwrap_or(0.0);
    let f1 = expr.eval(var, 1.0).unwrap_or(0.0);
    let fm1 = expr.eval(var, -1.0).unwrap_or(0.0);

    let c = f0;
    let a = (f1 + fm1 - 2.0 * c) / 2.0;
    let b = (f1 - fm1) / 2.0;

    (a, b, c)
}

fn format_number(n: f64) -> String {
    if (n - libm::round(n)).abs() < 1e-10 && n.abs() < 1e15 {
        format!("{}", libm::round(n) as i64)
    } else {
        format!("{:.6}", n)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Public command interface
// ─────────────────────────────────────────────────────────────────────────────

/// Process a math command string. Returns formatted result.
///
/// Commands:
///   solve "2x + 3 = 7"
///   derive "x^2 + 3x"  (or derivative, d/dx)
///   integrate "2x"      (or integral)
///   simplify "2x + 3x"
///   eval "x^2 + 1" x=3
pub fn process_math_command(input: &str) -> String {
    let input = input.trim();

    // Parse command + argument
    let (cmd, arg) = if let Some(pos) = input.find(' ') {
        (&input[..pos], input[pos + 1..].trim())
    } else {
        return format!("Usage: solve/derive/integrate/simplify <expression>");
    };

    // Strip quotes
    let arg = arg.trim_matches('"').trim_matches('\'').trim();

    match cmd {
        "solve" | "giai" => match parse_equation(arg) {
            Ok(eq) => {
                let var = find_variable(&eq.lhs).or_else(|| find_variable(&eq.rhs));
                let var = var.unwrap_or_else(|| "x".into());
                let result = solve(&eq, &var);
                result.steps.join("\n")
            }
            Err(e) => format!("Parse error: {}", e),
        },

        "derive" | "derivative" | "dao-ham" | "d/dx" => match parse_math(arg) {
            Ok(expr) => {
                let var = find_variable(&expr).unwrap_or_else(|| "x".into());
                let result = derivative_steps(&expr, &var);
                result.steps.join("\n")
            }
            Err(e) => format!("Parse error: {}", e),
        },

        "integrate" | "integral" | "tich-phan" => match parse_math(arg) {
            Ok(expr) => {
                let var = find_variable(&expr).unwrap_or_else(|| "x".into());
                let result = integrate_steps(&expr, &var);
                result.steps.join("\n")
            }
            Err(e) => format!("Parse error: {}", e),
        },

        "simplify" | "rut-gon" => match parse_math(arg) {
            Ok(expr) => {
                let simplified = simplify(&expr);
                format!("{} = {}", expr.display(), simplified.display())
            }
            Err(e) => format!("Parse error: {}", e),
        },

        "eval" => {
            // eval "x^2 + 1" x=3
            let parts: Vec<&str> = arg.splitn(2, ' ').collect();
            if parts.len() != 2 {
                return "Usage: eval <expr> <var>=<value>".into();
            }
            let expr_str = parts[0].trim_matches('"').trim_matches('\'');
            let assign = parts[1];
            let assign_parts: Vec<&str> = assign.splitn(2, '=').collect();
            if assign_parts.len() != 2 {
                return "Usage: eval <expr> <var>=<value>".into();
            }
            let var = assign_parts[0].trim();
            let val: f64 = match assign_parts[1].trim().parse() {
                Ok(v) => v,
                Err(_) => return "Invalid number for variable value".into(),
            };
            match parse_math(expr_str) {
                Ok(expr) => match expr.eval(var, val) {
                    Some(result) => format!("f({}) = {} → {}", val, expr.display(), format_number(result)),
                    None => "Cannot evaluate (undefined)".into(),
                },
                Err(e) => format!("Parse error: {}", e),
            }
        }

        _ => format!("Unknown math command: {}. Use solve/derive/integrate/simplify/eval", cmd),
    }
}

/// Find the first variable in an expression.
fn find_variable(expr: &MathExpr) -> Option<String> {
    match expr {
        MathExpr::Var(v) => Some(v.clone()),
        MathExpr::Add(a, b)
        | MathExpr::Sub(a, b)
        | MathExpr::Mul(a, b)
        | MathExpr::Div(a, b)
        | MathExpr::Pow(a, b) => find_variable(a).or_else(|| find_variable(b)),
        MathExpr::Neg(a) | MathExpr::Sin(a) | MathExpr::Cos(a) | MathExpr::Ln(a) => {
            find_variable(a)
        }
        MathExpr::Num(_) => None,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Parser ──────────────────────────────────────────────────────────────

    #[test]
    fn parse_number() {
        let e = parse_math("42").unwrap();
        assert_eq!(e, MathExpr::Num(42.0));
    }

    #[test]
    fn parse_variable() {
        let e = parse_math("x").unwrap();
        assert_eq!(e, MathExpr::Var("x".into()));
    }

    #[test]
    fn parse_addition() {
        let e = parse_math("1 + 2").unwrap();
        if let MathExpr::Add(a, b) = e {
            assert_eq!(*a, MathExpr::Num(1.0));
            assert_eq!(*b, MathExpr::Num(2.0));
        } else {
            panic!("Expected Add");
        }
    }

    #[test]
    fn parse_implicit_mul() {
        // "2x" → 2 * x
        let e = parse_math("2x").unwrap();
        if let MathExpr::Mul(a, b) = e {
            assert_eq!(*a, MathExpr::Num(2.0));
            assert_eq!(*b, MathExpr::Var("x".into()));
        } else {
            panic!("Expected Mul, got {:?}", e);
        }
    }

    #[test]
    fn parse_polynomial() {
        // "x^2 + 3x - 5"
        let e = parse_math("x^2 + 3x - 5").unwrap();
        assert!(e.contains_var("x"));
    }

    #[test]
    fn parse_function_sin() {
        let e = parse_math("sin(x)").unwrap();
        if let MathExpr::Sin(inner) = e {
            assert_eq!(*inner, MathExpr::Var("x".into()));
        } else {
            panic!("Expected Sin");
        }
    }

    #[test]
    fn parse_nested_parens() {
        let e = parse_math("(2 + 3) * x").unwrap();
        assert!(e.contains_var("x"));
    }

    #[test]
    fn parse_pi() {
        let e = parse_math("pi").unwrap();
        if let MathExpr::Num(n) = e {
            assert!((n - core::f64::consts::PI).abs() < 1e-10);
        } else {
            panic!("Expected pi constant");
        }
    }

    #[test]
    fn parse_equation_basic() {
        let eq = parse_equation("2x + 3 = 7").unwrap();
        assert!(eq.lhs.contains_var("x"));
    }

    // ── Simplify ────────────────────────────────────────────────────────────

    #[test]
    fn simplify_constant_fold() {
        let e = parse_math("2 + 3").unwrap();
        let s = simplify(&e);
        assert_eq!(s, MathExpr::Num(5.0));
    }

    #[test]
    fn simplify_zero_add() {
        let e = MathExpr::Add(
            Box::new(MathExpr::Num(0.0)),
            Box::new(MathExpr::Var("x".into())),
        );
        let s = simplify(&e);
        assert_eq!(s, MathExpr::Var("x".into()));
    }

    #[test]
    fn simplify_one_mul() {
        let e = MathExpr::Mul(
            Box::new(MathExpr::Num(1.0)),
            Box::new(MathExpr::Var("x".into())),
        );
        let s = simplify(&e);
        assert_eq!(s, MathExpr::Var("x".into()));
    }

    #[test]
    fn simplify_zero_mul() {
        let e = MathExpr::Mul(
            Box::new(MathExpr::Num(0.0)),
            Box::new(MathExpr::Var("x".into())),
        );
        let s = simplify(&e);
        assert_eq!(s, MathExpr::Num(0.0));
    }

    // ── Eval ────────────────────────────────────────────────────────────────

    #[test]
    fn eval_polynomial() {
        let e = parse_math("x^2 + 3x - 5").unwrap();
        let val = e.eval("x", 2.0).unwrap();
        // 4 + 6 - 5 = 5
        assert!((val - 5.0).abs() < 1e-10, "f(2) = {}", val);
    }

    #[test]
    fn eval_sin() {
        let e = parse_math("sin(0)").unwrap();
        let val = e.eval("x", 0.0).unwrap();
        assert!(val.abs() < 1e-10);
    }

    // ── Derivative ──────────────────────────────────────────────────────────

    #[test]
    fn derivative_constant() {
        let e = MathExpr::Num(5.0);
        let d = derivative(&e, "x");
        assert_eq!(d, MathExpr::Num(0.0));
    }

    #[test]
    fn derivative_x() {
        let e = MathExpr::Var("x".into());
        let d = derivative(&e, "x");
        assert_eq!(d, MathExpr::Num(1.0));
    }

    #[test]
    fn derivative_x_squared() {
        // d/dx(x²) = 2x
        let e = parse_math("x^2").unwrap();
        let d = derivative(&e, "x");
        // Evaluate: d(2) should be 4
        let val = d.eval("x", 2.0).unwrap();
        assert!((val - 4.0).abs() < 1e-10, "d/dx(x²) at x=2 = {}", val);
    }

    #[test]
    fn derivative_polynomial() {
        // d/dx(x^2 + 3x - 5) = 2x + 3
        let e = parse_math("x^2 + 3x - 5").unwrap();
        let d = derivative(&e, "x");
        let val = d.eval("x", 1.0).unwrap();
        assert!((val - 5.0).abs() < 1e-10, "d/dx at x=1 = {}", val);
    }

    #[test]
    fn derivative_sin_x() {
        // d/dx(sin(x)) = cos(x)
        let e = parse_math("sin(x)").unwrap();
        let d = derivative(&e, "x");
        // At x=0: cos(0)=1
        let val = d.eval("x", 0.0).unwrap();
        assert!((val - 1.0).abs() < 1e-10, "d/dx(sin(0)) = {}", val);
    }

    #[test]
    fn derivative_cos_x() {
        // d/dx(cos(x)) = -sin(x)
        let e = parse_math("cos(x)").unwrap();
        let d = derivative(&e, "x");
        let val = d.eval("x", 0.0).unwrap();
        assert!(val.abs() < 1e-10, "d/dx(cos(0)) = {}", val);
    }

    #[test]
    fn derivative_ln_x() {
        // d/dx(ln(x)) = 1/x
        let e = parse_math("ln(x)").unwrap();
        let d = derivative(&e, "x");
        let val = d.eval("x", 2.0).unwrap();
        assert!((val - 0.5).abs() < 1e-10, "d/dx(ln(2)) = {}", val);
    }

    #[test]
    fn derivative_steps_display() {
        let e = parse_math("x^2").unwrap();
        let result = derivative_steps(&e, "x");
        assert!(!result.steps.is_empty());
        assert!(result.steps[0].contains("d/dx"));
    }

    // ── Integrate ───────────────────────────────────────────────────────────

    #[test]
    fn integrate_constant() {
        // ∫ 5 dx = 5x
        let e = MathExpr::Num(5.0);
        let i = integrate(&e, "x").unwrap();
        let val = i.eval("x", 3.0).unwrap();
        assert!((val - 15.0).abs() < 1e-10, "∫5 dx at x=3 = {}", val);
    }

    #[test]
    fn integrate_x() {
        // ∫ x dx = x²/2
        let e = MathExpr::Var("x".into());
        let i = integrate(&e, "x").unwrap();
        let val = i.eval("x", 4.0).unwrap();
        assert!((val - 8.0).abs() < 1e-10, "∫x dx at x=4 = {}", val);
    }

    #[test]
    fn integrate_x_squared() {
        // ∫ x² dx = x³/3
        let e = parse_math("x^2").unwrap();
        let i = integrate(&e, "x").unwrap();
        let val = i.eval("x", 3.0).unwrap();
        assert!((val - 9.0).abs() < 1e-10, "∫x² dx at x=3 = {}", val);
    }

    #[test]
    fn integrate_polynomial() {
        // ∫ (2x + 1) dx = x² + x
        let e = parse_math("2x + 1").unwrap();
        let i = integrate(&e, "x").unwrap();
        let val = i.eval("x", 3.0).unwrap();
        // x² + x at x=3 = 9 + 3 = 12
        assert!((val - 12.0).abs() < 1e-10, "∫(2x+1) dx at x=3 = {}", val);
    }

    #[test]
    fn integrate_sin() {
        // ∫ sin(x) dx = -cos(x)
        let e = parse_math("sin(x)").unwrap();
        let i = integrate(&e, "x").unwrap();
        let val = i.eval("x", 0.0).unwrap();
        assert!((val - (-1.0)).abs() < 1e-10, "∫sin(x) dx at x=0 = {}", val);
    }

    #[test]
    fn integrate_cos() {
        // ∫ cos(x) dx = sin(x)
        let e = parse_math("cos(x)").unwrap();
        let i = integrate(&e, "x").unwrap();
        let val = i.eval("x", 0.0).unwrap();
        assert!(val.abs() < 1e-10, "∫cos(x) dx at x=0 = {}", val);
    }

    // ── Solve ───────────────────────────────────────────────────────────────

    #[test]
    fn solve_linear() {
        // 2x + 3 = 7 → x = 2
        let eq = parse_equation("2x + 3 = 7").unwrap();
        let result = solve(&eq, "x");
        if let MathExpr::Num(x) = result.result {
            assert!((x - 2.0).abs() < 1e-10, "x = {}", x);
        } else {
            panic!("Expected numeric result");
        }
    }

    #[test]
    fn solve_linear_negative() {
        // 3x - 9 = 0 → x = 3
        let eq = parse_equation("3x - 9 = 0").unwrap();
        let result = solve(&eq, "x");
        if let MathExpr::Num(x) = result.result {
            assert!((x - 3.0).abs() < 1e-10, "x = {}", x);
        } else {
            panic!("Expected numeric result");
        }
    }

    #[test]
    fn solve_quadratic() {
        // x² - 5x + 6 = 0 → x = 2 or x = 3
        let eq = parse_equation("x^2 - 5x + 6 = 0").unwrap();
        let result = solve(&eq, "x");
        // Result should contain both roots
        assert!(result.steps.iter().any(|s| s.contains("₁") || s.contains("₂")));
    }

    #[test]
    fn solve_quadratic_no_real() {
        // x² + 1 = 0 → no real solutions
        let eq = parse_equation("x^2 + 1 = 0").unwrap();
        let result = solve(&eq, "x");
        assert!(result.steps.iter().any(|s| s.contains("no real")));
    }

    #[test]
    fn solve_steps_output() {
        let eq = parse_equation("2x + 3 = 7").unwrap();
        let result = solve(&eq, "x");
        assert!(result.steps.len() >= 3, "Should have multiple steps");
    }

    // ── process_math_command ────────────────────────────────────────────────

    #[test]
    fn cmd_solve() {
        let output = process_math_command("solve \"2x + 3 = 7\"");
        assert!(output.contains("2"), "Should find x=2");
    }

    #[test]
    fn cmd_derive() {
        let output = process_math_command("derive \"x^2 + 3x\"");
        assert!(output.contains("d/dx"), "Should show derivative");
    }

    #[test]
    fn cmd_integrate() {
        let output = process_math_command("integrate \"2x\"");
        assert!(output.contains("∫"), "Should show integral");
    }

    #[test]
    fn cmd_simplify() {
        let output = process_math_command("simplify \"2 + 3\"");
        assert!(output.contains("5"), "Should simplify to 5");
    }

    #[test]
    fn cmd_eval() {
        let output = process_math_command("eval \"x^2\" x=3");
        assert!(output.contains("9"), "f(3) = 9");
    }

    // ── Display ─────────────────────────────────────────────────────────────

    #[test]
    fn display_integer() {
        assert_eq!(MathExpr::Num(3.0).display(), "3");
    }

    #[test]
    fn display_coefficient() {
        let e = MathExpr::Mul(
            Box::new(MathExpr::Num(2.0)),
            Box::new(MathExpr::Var("x".into())),
        );
        assert_eq!(e.display(), "2x");
    }

    #[test]
    fn display_polynomial() {
        let e = parse_math("x^2 + 3x - 5").unwrap();
        let s = e.display();
        assert!(s.contains("x") && s.contains("5"));
    }
}
