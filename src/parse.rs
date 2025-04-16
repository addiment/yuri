// our grammar:
// Unsigned = any unsigned integer literal
// Signed = any signed integer literal
// Float = any float literal
// Number = Unsigned|Signed|Float
// WS = whitespace/comments
// Ident = any valid identifier (including the primitive types and ".")
// Array = Type + WS? + "[" + WS? + (Unsigned|Ident) + WS? + "]"
// Complex = "<|" + WS? + Ident + WS? + ":" + WS? + Type + WS? + "|>"
// Type = Primitive|Array|Complex|Ident
// Property = "prop" + WS + Ident + WS? + ":" + WS? + Type
// Variable = ("export" + WS)? + "let" + WS + Ident + WS? + (":" + WS? + Type)? + WS? + "=" + WS? + Expression + WS?
// Function = "fn" + WS + Ident + WS? + "(" + WS? + ((Ident + WS? ":" + ) + ",")* + ")" + WS? + (":" + WS? + Type)? + WS? + Block
// Block = "{" + WS? + (Statement + WS?)* + WS? + (Statement|Expression)? + WS? +"}"
// BinarySymbolOperator = "*"|"/"|"+"|"-"|"%"|"**"
// BinaryKeywordOperator = "and"|"xor"|"or"
// BinaryExpression = Expression + ((WS? + BinaryMathOperator + WS?)|(WS + BinaryKeywordOperator + WS)) + Expression
// UnaryOperator = "!"|"-"
// UnaryExpression = UnaryOperator + WS? + Expression
// Expression = Ident|Block|Literal|BinaryExpression|UnaryExpression
// Annotation = "@" + Ident
// Statement = (Variable|Expression) + ";"
// Module = "module" + WS + Ident + WS? + "{" + Shader + "}"
// Declaration = (Variable|Property|Function|Import|Module) + ";"
// Shader = (Declaration|WS)*

// "u"|"i"|"f"|"u2"|"i2"|"f2"|"u3"|"i3"|"f3"|"u4"|"i4"|"f4"|"m2"|"m3"|"m4"

use std::collections::VecDeque;
use crate::error::{YuriLexError, YuriSemanticError};
use crate::lex::{Keyword, YuriAst, YuriToken, YuriTokenType};
use crate::lex::YuriTokenType::Identifier;

#[derive(Debug, Clone)]
pub enum CompositeSize {
	Two,
	Three,
	Four
}

#[derive(Debug, Clone)]
pub enum NumberType {
	Float,
	Signed,
	Unsigned,
}

#[derive(Debug, Clone)]
pub enum YuriType {
	Unit,
	Scalar(NumberType),
	Vector(NumberType, CompositeSize),
	Array(Box<YuriType>, usize),
	Complex(Vec<(String, YuriType)>)
}

// "if" statements are incredibly annoying syntactically.
// I wish I could put this inside an enum variant, but I need two extra structs!
#[derive(Debug, Clone)]
struct IfExpression {
	condition: Box<Expression>,
	block: Vec<Statement>,
	block_else: Option<Else>,
}

#[derive(Debug, Clone)]
enum Else {
	Block(Vec<Statement>),
	If(Box<IfExpression>),
}

#[derive(Debug, Clone)]
enum Literal {
	DecimalNumber(i64),
	HexNumber(i64),
	BinaryNumber(i64),
	FloatNumber(f32),
	Boolean(bool),
	Vector {
		dimensions: CompositeSize,
		contents: Vec<Expression>,
	},
	Matrix {
		dimensions: CompositeSize,
		contents: Vec<Expression>,
	},
	Array {
		array_type: YuriType,
		contents: Vec<Expression>,
	},
}

#[derive(Debug, Clone)]
enum Expression {
	Literal(Literal),
	Variable(String),
	FunctionCall {
		function_name: String,
		arguments: Vec<Expression>
	},
	Block(Vec<Statement>),
	If(IfExpression),
	Loop {
		count: Box<Expression>,
		block: Vec<Statement>,
	},
	Fold {
		initial: Box<Expression>,
		items: Box<Expression>,
		block: Vec<Statement>
	},
	Map {
		initial: Box<Expression>,
		block: Vec<Statement>
	},
	Filter {
		initial: Box<Expression>,
		block: Vec<Statement>
	},
}

#[derive(Debug, Clone)]
struct VariableDeclaration {
	name: String,
	explicit_type: Option<YuriType>,
	inferred_type: Option<YuriType>,
	value: Expression,
	exported: bool,
}

/// A statement is a syntax element that can only occur in blocks.
#[derive(Debug, Clone)]
enum Statement {
	Expression(Expression),
	Variable(VariableDeclaration),
	Return(Expression),
}

#[derive(Debug, Clone)]
enum BinaryOperator {
	Plus,
	Minus,
	Times,
	Divided,
	Exponent,
}

#[derive(Debug, Clone)]
enum UnaryOperator {
	Negate,
	Not,
}

#[derive(Debug, Clone)]
enum Operator {
	Unary(UnaryOperator),
	Binary(BinaryOperator),
}

// impl Operator {
// 	fn precedence(&self) -> u8 {
// 		match self {
// 			Operator::Unary(op) => match op {
// 				UnaryOperator::Negate => {}
// 				UnaryOperator::Not => {}
// 			},
// 			Operator::Binary(op) => match op {
//
// 			},
// 		}
// 	}
// }


#[derive(Debug, Clone)]
struct FunctionDeclaration {
	name: String,
	return_type: YuriType,
	arguments: Vec<(String, YuriType)>,
	exported: bool,
}

#[derive(Debug, Clone)]
struct PropertyDeclaration {
	name: String,
	property_type: YuriType,
}

#[derive(Debug, Clone)]
struct ImportDeclaration {
	module: String,
}

#[derive(Debug, Default, Clone)]
pub struct YuriModule {
	imports: Vec<ImportDeclaration>,
	properties: Vec<PropertyDeclaration>,
	globals: Vec<VariableDeclaration>,
	functions: Vec<FunctionDeclaration>,
	submodules: Vec<(String, YuriModule)>
}

pub(super) fn parse_input(ast: &YuriAst) -> Result<YuriModule, YuriSemanticError> {
	fn parse_input_recursive(ast: &mut VecDeque<YuriToken>, current_module: &mut YuriModule) {
		let mut annotations = Vec::new();
		while let Some(mut tok) = ast.pop_front() {
			if let YuriTokenType::Annotation(ann) = tok.token_type {
				annotations.push(ann);
				continue;
			}

			if let YuriTokenType::Keyword(kw) = tok.token_type {
				// module declaration
				if kw == Keyword::Module {
					let name = if let Some(name) = ast.pop_front() {
						if let Identifier(name) = name.token_type {
							name
						} else {
							eprintln!("unexpected token (expected module name) {name:?}");
							break;
						}
					} else {
						break;
					};
					let mut submodule = YuriModule::default();
					// NOTE: this could cause stack overflow
					// if modules are nested a comically large amount
					parse_input_recursive(ast, &mut submodule);
					current_module.submodules.push((name, submodule));
					continue;
				}

				let is_exported = kw == Keyword::Export;

			}
		}
	}
	let mut ast = VecDeque::from(ast.clone());
	let mut module_state = YuriModule::default();
	parse_input_recursive(&mut ast, &mut module_state);
	Ok(module_state)
}