//! This is a basic recursive-descent parser I implemented in Rust for a college course.
//! I'm keeping it here in the source tree so that I don't forget how to write a parser.
//!
//! ...and yes, the assignment was NOT in Rust :)

use std::fmt::{Display, Formatter};
use regex::Regex;
use UnaryExprType::{Cos, Group, Log, Sin};
use crate::BinaryExprType::{Add, Div, Exp, Mul, Sub};

// S → S+M | S-M | M
// M → M*E | M/E | E
// E → P^E | P | log P
// P → (S) | L | V
// L → <float>
// V → x

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum BinaryExprType {
	Exp,
	Mul,
	Div,
	Add,
	Sub,
}

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum UnaryExprType {
	Group,
	Log,
	Cos,
	Sin,
}

#[derive(PartialEq, Debug)]
pub enum Expression {
	Literal(f64),
	Variable,
	Unary {
		t: UnaryExprType,
		v: Box<Expression>,
	},
	Binary {
		t: BinaryExprType,
		a: Box<Expression>,
		b: Box<Expression>,
	},
}

impl Display for Expression {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Expression::Literal(n) => { write!(f, "{n}") }
			Expression::Variable => { write!(f, "x") }
			Expression::Unary { t, v } => {
				match t {
					Log => { write!(f, "log({v})") }
					Group => { write!(f, "({v})") }
					Cos => { write!(f, "cos({v})") }
					Sin => { write!(f, "sin({v})") }
				}
			}
			Expression::Binary { t, a, b } => {
				match t {
					Exp => { write!(f, "({a} ^ {b})") }
					Mul => { write!(f, "({a} * {b})") }
					Div => { write!(f, "({a} / {b})") }
					Add => { write!(f, "({a} + {b})") }
					Sub => { write!(f, "({a} - {b})") }
				}
			}
		}
	}
}

impl Expression {
	fn evaluate(&self, x: f64) -> f64 {
		match self {
			Expression::Literal(n) => *n,
			Expression::Variable => x,
			Expression::Unary { t, v } => {
				match t {
					Group => v.evaluate(x),
					Log => v.evaluate(x).ln(),
					Cos => v.evaluate(x).cos(),
					Sin => v.evaluate(x).sin(),
				}
			}
			Expression::Binary { t, a, b } => {
				match t {
					Exp => a.evaluate(x).powf(b.evaluate(x)),
					Mul => a.evaluate(x) * b.evaluate(x),
					Div => a.evaluate(x) / b.evaluate(x),
					Add => a.evaluate(x) + b.evaluate(x),
					Sub => a.evaluate(x) - b.evaluate(x),
				}
			}
		}
	}
}

fn bin_parse(
	input: &str,
	delim: char,
	bin_type: BinaryExprType,
	indent: usize,
	a: fn(&str, usize) -> Option<Expression>,
	b: fn(&str, usize) -> Option<Expression>,
) -> Option<Expression> {
	let chunks: Vec<&str> = input.split(delim).collect();
	if chunks.len() < 2 {
		return None;
	}
	for i in (1..chunks.len()).rev() {
		let (prefix, suffix) = chunks.split_at(i);
		let prefix = prefix.join(&String::from(delim));
		let suffix = suffix.join(&String::from(delim));

		println!("prefix {prefix}");
		println!("suffix {suffix}");

		let res_a = a(&prefix, indent);
		let res_b = b(&suffix, indent);

		if res_a.is_none() || res_b.is_none() {
			continue;
		}

		return Some(Expression::Binary {
			t: bin_type,
			a: Box::new(res_a.unwrap()),
			b: Box::new(res_b.unwrap()),
		});
	}

	None
}

pub fn parse_s(input: &str, indent: usize) -> Option<Expression> {
	println!("{}S -> \"{input}\"", "    ".repeat(indent));
	// S -> S+M | S-M | M
	let res = bin_parse(input, '+', Add, indent + 1, parse_s, parse_s);
	if let Some(res) = res {
		return Some(res);
	}

	let res = bin_parse(input, '-', Sub, indent + 1, parse_s, parse_s);
	if let Some(res) = res {
		return Some(res);
	}

	parse_m(input, indent)
}

pub fn parse_m(input: &str, indent: usize) -> Option<Expression> {
	println!("{}M -> \"{input}\"", "    ".repeat(indent));
	// M -> M*E | M/E | E
	let res = bin_parse(input, '*', Mul, indent + 1, parse_m, parse_m);
	if let Some(res) = res {
		return Some(res);
	}

	let res = bin_parse(input, '/', Div, indent + 1, parse_m, parse_m);
	if let Some(res) = res {
		return Some(res);
	}

	parse_e(input, indent)
}

pub fn parse_e(input: &str, indent: usize) -> Option<Expression> {
	println!("{}E -> \"{input}\"", "    ".repeat(indent));
	// E -> P^E | log P | P
	let res = bin_parse(input, '^', Exp, indent + 1, parse_e, parse_e);
	if let Some(res) = res {
		return Some(res);
	}

	if input.starts_with("log") {
		let expr = parse_plv(&input[3..input.len()], indent);
		if let Some(expr) = expr {
			return Some(Expression::Unary {
				t: Log,
				v: Box::from(expr),
			});
		}
	}

	if input.starts_with("cos") {
		let expr = parse_plv(&input[3..input.len()], indent);
		if let Some(expr) = expr {
			return Some(Expression::Unary {
				t: Cos,
				v: Box::from(expr),
			});
		}
	}

	if input.starts_with("sin") {
		let expr = parse_plv(&input[3..input.len()], indent);
		if let Some(expr) = expr {
			return Some(Expression::Unary {
				t: Sin,
				v: Box::from(expr),
			});
		}
	}

	parse_plv(input, indent)
}

pub fn parse_plv(input: &str, indent: usize) -> Option<Expression> {
	println!("{}PLV -> \"{input}\"", "    ".repeat(indent));
	// P -> (S) | L | V
	// L -> <float>
	// V -> x

	if input.starts_with('(') {
		let chars: Vec<char> = input.chars().collect();

		let mut counter = 1i32;
		let mut i = 1;

		while i < chars.len() {
			let char = chars[i];
			if char == '(' {
				counter += 1;
			} else if char == ')' {
				counter -= 1;
			}
			if counter == 0 {
				break;
			}
			i += 1;
		}
		if counter != 0 {
			return None;
		}
		return parse_s(&input[1..i], indent);
	}

	// handles both L and V
	let regexp = Regex::new("(?<var>x)|(?<num>[0-9]+(?:\\.[0-9]+)?)").unwrap();
	if let Some(captures) = regexp.captures(input) {
		if let Some(x) = captures.name("var") {
			if x.len() < input.len() {
				return None;
			}
			println!("{}x: {}", "    ".repeat(indent + 1), &input[x.range()]);
			return Some(Expression::Variable);
		} else if let Some(n) = captures.name("num") {
			if n.len() < input.len() {
				return None;
			}
			println!("{}n: {}", "    ".repeat(indent + 1), &input[n.range()]);
			return Some(Expression::Literal(input[n.range()].parse().unwrap()));
		}
	}

	panic!("No pattern matches for symbol P|L|V (input {input})");
}

fn main() {
	let test_expr_s = "10*x^3 + 2*(15+x) + log(69)";
	let test_parse_res = parse_s(&*test_expr_s.replace(' ', ""), 0).unwrap();
	println!("\"{test_parse_res}\"");
}

#[cfg(test)]
mod test {
	use crate::parse_s;

	#[test]
	fn test1() {
		let test_expr_s = "10*x^3 + 2*(15+x) + log(69)";
		let test_parse_res = parse_s(&*test_expr_s.replace(' ', ""), 0).unwrap();
		println!("\"{test_parse_res}\"");
		// assert_eq!(parse_s(&*test_expr_s.replace(' ', "")), test_expr_d);
	}
}
