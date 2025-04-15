use std::num::ParseFloatError;
use std::ops::{Neg, Range};
use crate::error::YuriParseError;
// our grammar:
// Unsigned = any unsigned integer literal
// Signed = any signed integer literal
// Float = any float literal
// Number = Unsigned|Signed|Float
// WS = whitespace/comments
// Ident = any valid identifier (including the primitive types and member access)
// Array = K + WS? + "[" + WS? + (Unsigned|Ident) + WS? + "]"
// Complex = "<|" + WS? + Ident + WS? + ":" + WS? + Type + WS? + "|>"
// Type = Primitive|Array|Complex|Ident
// Property = "prop" + WS + Ident + WS? + ":" + WS? + Type
// Variable = ("export" + WS)? + "let" + WS + Ident + WS? + (":" + WS? + Type)? + WS? + "=" + WS? + Expression + WS?
// Block = "{" + WS? + (Statement + WS?)* + WS? + (Statement|Expression)? + WS? +"}"
// Expression = Ident|Block|Literal
// Annotation = "@" + Ident
// Statement = (Variable|Expression) + ";"
// Module = "module" + WS + Ident + WS? + "{" + Shader + "}"
// Declaration = (Variable|Property|Function|Import|Module) + ";"
// Shader = (Declaration|WS)*

// "u"|"i"|"f"|"u2"|"i2"|"f2"|"u3"|"i3"|"f3"|"u4"|"i4"|"f4"|"m2"|"m3"|"m4"

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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Keyword {
	Fn,
	Let,
	Prop,

	Loop,
	Map,
	Filter,

	Import,
	Export,
	Module,

	Return,
	/// Reserved
	Core,

	TypeBool,
	TypeF,
	TypeU,
	TypeI,
	TypeF2,
	TypeU2,
	TypeI2,
	TypeF3,
	TypeU3,
	TypeI3,
	TypeF4,
	TypeU4,
	TypeI4,
	TypeM2,
	TypeM3,
	TypeM4,
	TypeSampler1,
	TypeSampler2,
	TypeSampler3,
	TypeSampler4,
}

impl Keyword {
	pub const ALL: [Keyword; 31] = { use Keyword::*; [
		Fn,
		Let,
		Prop,

		Loop,
		Map,
		Filter,

		Import,
		Export,
		Module,

		Return,
		Core,

		TypeBool,
		TypeF,
		TypeU,
		TypeI,
		TypeF2,
		TypeU2,
		TypeI2,
		TypeF3,
		TypeU3,
		TypeI3,
		TypeF4,
		TypeU4,
		TypeI4,
		TypeM2,
		TypeM3,
		TypeM4,
		TypeSampler1,
		TypeSampler2,
		TypeSampler3,
		TypeSampler4,
	] };
	const fn stringify(&self) -> &'static str {
		match self {
			Keyword::Fn 			=> "fn",
			Keyword::Let 			=> "let",
			Keyword::Prop 			=> "prop",
			Keyword::Loop 			=> "loop",
			Keyword::Map 			=> "map",
			Keyword::Filter 		=> "filter",
			Keyword::Import 		=> "import",
			Keyword::Export 		=> "export",
			Keyword::Module 		=> "module",
			Keyword::Return 		=> "return",
			Keyword::TypeBool 		=> "bool",
			Keyword::TypeF 			=> "f",
			Keyword::TypeU 			=> "u",
			Keyword::TypeI 			=> "i",
			Keyword::TypeF2 		=> "f2",
			Keyword::TypeU2 		=> "u2",
			Keyword::TypeI2 		=> "i2",
			Keyword::TypeF3 		=> "f3",
			Keyword::TypeU3 		=> "u3",
			Keyword::TypeI3 		=> "i3",
			Keyword::TypeF4 		=> "f4",
			Keyword::TypeU4 		=> "u4",
			Keyword::TypeI4 		=> "i4",
			Keyword::TypeM2 		=> "m2",
			Keyword::TypeM3 		=> "m3",
			Keyword::TypeM4 		=> "m4",
			Keyword::TypeSampler1 	=> "sampler1",
			Keyword::TypeSampler2 	=> "sampler2",
			Keyword::TypeSampler3 	=> "sampler3",
			Keyword::TypeSampler4 	=> "sampler4",
			Keyword::Core 			=> "core",
		}
	}
	pub fn string_to_keyword(string: &str) -> Option<Self> {
		// TODO: this is slow as all hell because Rust needs better enum tools.
		Self::ALL.iter()
			.find(|k| k.stringify() == string)
			.copied()
	}
}

impl From<Keyword> for &'static str {
	fn from(value: Keyword) -> Self {
		value.stringify()
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct YuriToken {
	pub token_type: YuriTokenType,
	pub location: Range<usize>
}

impl YuriToken {
	pub fn new(token_type: YuriTokenType, location: Range<usize>) -> Self {
		Self {
			token_type,
			location
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum YuriTokenType {
	Unknown,
	/// (
	OpenParen,
	/// )
	CloseParen,
	/// {
	OpenBrace,
	/// }
	CloseBrace,
	/// [
	OpenSquare,
	/// ]
	CloseSquare,
	/// <|
	OpenTri,
	/// |>
	CloseTri,
	/// ;
	Terminator,
	/// ,
	Separator,
	/// =
	Assignment,
	/// :
	TypeHint,

	HexNumber(u32),
	BinaryNumber(u32),
	UnsignedNumber(u32),
	SignedNumber(i32),
	DecimalNumber(f32),

	Keyword(Keyword),
	Annotation(String),
	Identifier(String),
	Operator(String),
}

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
pub struct ShaderModule {
	imports: Vec<ImportDeclaration>,
	properties: Vec<PropertyDeclaration>,
	globals: Vec<VariableDeclaration>,
	functions: Vec<FunctionDeclaration>,
}

type Input<'a> = &'a[char];

pub type YuriAst = Vec<YuriToken>;

fn get_or_eof(input: Input, seek: usize) -> Result<char, YuriParseError> {
	match input.get(seek) {
		None => Err(YuriParseError::UnexpectedEndOfFile),
		Some(ch) => Ok(*ch)
	}
}


/// This function is an affront to god. But so are lesbians, so it doesn't really matter.
/// I am afraid to split this up into multiple other functions because
/// of how the pieces all fit together. I totally could, but it's fine like this.
fn take_token(input: Input, seek: &mut usize) -> Result<YuriToken, YuriParseError> {
	let initial_seek = *seek;
	let tt = match input[*seek] {
		'(' => { *seek += 1; YuriTokenType::OpenParen },
		')' => { *seek += 1; YuriTokenType::CloseParen },
		'{' => { *seek += 1; YuriTokenType::OpenParen },
		'}' => { *seek += 1; YuriTokenType::CloseParen },
		'[' => { *seek += 1; YuriTokenType::OpenParen },
		']' => { *seek += 1; YuriTokenType::CloseParen },
		':' => { *seek += 1; YuriTokenType::TypeHint },
		';' => { *seek += 1; YuriTokenType::Terminator },
		',' => { *seek += 1; YuriTokenType::Separator },
		'*' => { *seek += 1; YuriTokenType::Operator(String::from("*")) }
		'+' => { *seek += 1; YuriTokenType::Operator(String::from("+")) }
		'/' => { *seek += 1; YuriTokenType::Operator(String::from("/")) }
		'^' => { *seek += 1; YuriTokenType::Operator(String::from("^")) }
		'!' => { *seek += 1; YuriTokenType::Operator(String::from("!")) }

		'|' => {
			*seek += 1;
			match input.get(*seek) {
				// logical OR
				Some('|') => { *seek += 1; YuriTokenType::Operator(String::from("||")) },
				// close |> complex
				Some('>') => { *seek += 1; YuriTokenType::CloseTri }
				// binary OR followed by something else
				// some other character-- we'll try processing it again.
				None | Some(_) => YuriTokenType::Operator(String::from("|"))
			}
		},
		'<' => {
			*seek += 1;
			match input.get(*seek) {
				// open <| complex
				Some('|') => { *seek += 1; YuriTokenType::OpenTri },
				// shl op
				Some('<') => { *seek += 1; YuriTokenType::Operator(String::from("<<")) }
				None | Some(_) => YuriTokenType::Operator(String::from("<"))
			}
		},
		'=' => {
			*seek += 1;
			match input.get(*seek) {
				Some('=') => { *seek += 1; YuriTokenType::Operator(String::from("==")) },
				None | Some(_) => YuriTokenType::Assignment,
			}
		},
		'&' => {
			*seek += 1;
			match input.get(*seek) {
				Some('&') => { *seek += 1; YuriTokenType::Operator(String::from("&&")) }
				None | Some(_) => YuriTokenType::Operator(String::from("&"))
			}
		},
		'-' | '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
			// this control flow is really annoying to model without repetition.
			let ch = input[*seek];

			let subtraction_negation = if ch == '-' {
				if let Some(next) = input.get(*seek + 1) {
					if next.is_digit(10) {
						None
					} else {
						Some(YuriTokenType::Operator(String::from("-")))
					}
				} else {
					Some(YuriTokenType::Operator(String::from("-")))
				}
			} else {
				None
			};
			let special_number_formats = if ch == '0' {
				match input.get(*seek + 1) {
					// EOF
					None => Some(YuriTokenType::UnsignedNumber(0)),
					// hex number
					Some('x') => {
						*seek += 2;
						let mut digits = Vec::new();

						while let Some(ch) = input.get(*seek) {
							if *ch == '_' {
								*seek += 1;
							} else if let Some(digit) = ch.to_digit(16) {
								if !digits.is_empty() || digit != 0 {
									println!("got digit {digit}");
									digits.push(digit);
								} else {
									println!("skipping leading 0")
								}
								*seek += 1;
							} else {
								break;
							}
						};

						if digits.is_empty() {
							return Err(YuriParseError::InvalidDigit);
						}

						let mut sum: i64 = 0;
						for i in 0..digits.len() {
							let exp = digits.len() as u32 - i as u32 - 1;
							let digit_at = digits[i];
							println!("digit {digit_at} (index {i}) with exp {exp}");
							if digit_at != 0 {
								sum += digit_at as i64 * 16i64.pow(exp);
							}
						}
						Some(if let Ok(sum) = u32::try_from(sum) {
							YuriTokenType::HexNumber(sum)
						} else {
							return Err(YuriParseError::NumberOutOfBounds);
						})
					}
					// binary number
					Some('b') => {
						*seek += 2;
						let mut digits = Vec::<bool>::new();
						while let Some(ch) = input.get(*seek) {
							match ch {
								'0' => {
									*seek += 1;
									if !digits.is_empty() {
										digits.push(false);
									}
								}
								'1' => { *seek += 1; digits.push(true); }
								'_' => { *seek += 1; }
								_ => break
							}
						};
						if digits.is_empty() {
							return Err(YuriParseError::InvalidNumericLiteral);
						} else if digits.len() > 32 {
							return Err(YuriParseError::NumberOutOfBounds);
						}
						let mut sum = 0;
						for i in 0..digits.len() {
							if digits[i] {
								sum |= 1 << (digits.len() - i - 1);
							}
						}
						Some(YuriTokenType::BinaryNumber(sum))
					}
					_ => None,
				}
			} else { None };

			let exceptions = special_number_formats.or(subtraction_negation);

			if let Some(tok) = exceptions {
				tok
			} else {
				let mut digits = Vec::new();
				let mut decimal_point: Option<usize> = None;
				if ch == '-' {
					*seek += 1;
				}
				let number_start_seek = *seek;
				// parse digits
				while let Some(ch) = input.get(*seek) {
					if *ch == '_' {
						*seek += 1;
					} else if *ch == '.' {
						if decimal_point.is_some() {
							return Err(YuriParseError::InvalidNumericLiteral);
						}
						decimal_point = Some(digits.len());
						*seek += 1;
					} else if let Some(digit) = ch.to_digit(10) {
						let is_next_decimal = input.get(*seek + 1)
							.map_or(false, |ch| *ch == '.');
						if !digits.is_empty() || digit != 0 || is_next_decimal{
							digits.push(digit);
						}
						*seek += 1;
					} else {
						break;
					}
				};

				if digits.is_empty() {
					return Err(YuriParseError::InvalidDigit);
				}

				if let Some(_) = decimal_point {
					let strung = input[number_start_seek..*seek]
						.iter()
						.collect::<String>();
					match strung
						.parse::<f32>() {
						Ok(num) => YuriTokenType::DecimalNumber(if ch == '-' {
							println!("float: {strung}");
							num.neg()
						} else {
							num
						}),
						Err(_) => {
							return Err(YuriParseError::NumberOutOfBounds);
						}
					}
				} else {
					let mut sum: i64 = 0;
					for i in 0..digits.len() {
						let exp = digits.len() as u32 - i as u32 - 1;
						let digit_at = digits[i];
						if digit_at != 0 {
							sum += digit_at as i64 * 10i64.pow(exp);
						}
					}
					// negation
					if ch == '-' {
						if let Ok(sum) = i32::try_from(sum * -1) {
							YuriTokenType::SignedNumber(sum)
						} else {
							return Err(YuriParseError::NumberOutOfBounds);
						}
					} else {
						if let Ok(sum) = u32::try_from(sum) {
							YuriTokenType::UnsignedNumber(sum)
						} else {
							return Err(YuriParseError::NumberOutOfBounds);
						}
					}
				}
			}
		}
		ch => {
			// YuriToken::Unknown
			fn take_ident(input: Input, seek: &mut usize) -> Option<String> {
				let ch = *input.get(*seek)?;
				if ch.is_alphabetic() || ch == '_' {
					let mut ident = Vec::new();
					while let Some(ch) = input.get(*seek) {
						if ch.is_alphanumeric() || *ch == '_' || *ch == '.' {
							ident.push(*ch);
							*seek += 1;
						} else {
							break;
						}
					}
					Some(ident.into_iter().collect())
				} else {
					None
				}
			}

			if ch == '@' {
				*seek += 1;
				let annotation = take_ident(input, seek);
				if annotation.is_none() {
					return Err(YuriParseError::IncompleteAnnotation);
				}
				YuriTokenType::Annotation(annotation.unwrap())
			} else if let Some(ident) = take_ident(input, seek) {
				if let Some(kw) = Keyword::string_to_keyword(&ident) {
					YuriTokenType::Keyword(kw)
				} else {
					YuriTokenType::Identifier(ident)
				}
			} else {
				*seek += 1;
				YuriTokenType::Unknown
			}
		}
	};
	Ok(YuriToken::new(tt, initial_seek..*seek))
}

pub fn lex(input_string: &str) -> Result<YuriAst, YuriParseError> {
	let mut ast = YuriAst::new();
	let input: Vec<char> = input_string.chars().collect();
	let mut seek = take_whitespace(&input, 0, false)?;
	while seek < input.len() {
		let tok = take_token(&input, &mut seek);
		if tok.is_err() {
			eprintln!("error while parsing, seek = {seek}");
			tok?;
			unreachable!()
		}
		let tok = tok?;
		ast.push(tok);

		seek = take_whitespace(&input, seek, false)?;
	}

	Ok(ast)
}

/// Moves the seek forward until it hits a non-whitespace token.
/// If the function encounters comments, it will treat them as whitespace.
/// Block comments will generate a parse error if they are not terminated before EOF.
/// Will return true if once it hits
fn take_whitespace(input: Input, mut seek: usize, fail_on_eof: bool) -> Result<usize, YuriParseError> {
	// ParseHesitation::EndOfFile
	while seek < input.len() {
		let ch = input[seek];
		if ch.is_whitespace() {
			// keep going.
			// safe since we know there's at least 1 char
			seek += 1;
			// println!("took \'{}\'", ch.escape_default());
			continue;
		} else if ch == '#' {
			// println!("took comment start");
			seek += 1;

			// edge case where we have a hash at the VERY END of the file.
			// why would anyone do this???
			// it's technically valid though so let's make sure we validate it
			let next = if let Some(next) = input.get(seek) {
				*next
			} else if fail_on_eof {
				return Err(YuriParseError::UnexpectedEndOfFile)
			} else {
				return Ok(input.len())
			};
			// comment, line or block?
			// println!("took next \'{}\'", next.escape_default());
			if next == '#' {
				seek += 1;
				// println!("taking block comment");
				// block comment
				loop {
					// println!("in block comment; 0 is \'{}\'", input[0].escape_default());
					if let Some(end_pos) = input.iter()
						.skip(seek)
						.position(|c| *c == '#') {
						seek += end_pos + 1;
					} else {
						// EOF
						return Err(YuriParseError::UnexpectedEndOfFile);
					};
					// println!("in block comment; seeking {end_pos} forward");
					// we seek to the position where we found the next hash.
					// note that this is NOT the character after,
					// we account for that when we assign the next character
					let next = get_or_eof(input, seek)?;

					if next == '#' {
						// println!("ending");
						// move the seek to after the block comment ends,
						// or else we'll interpret it as the beginning of another comment.
						seek += 1;
						break;
					}
				}
			} else {
				// line comment
				let end_pos = input.iter()
					.skip(seek)
					.position(|c| *c == '\n');
				match end_pos {
					// EOF
					None => return if fail_on_eof {
						Err(YuriParseError::UnexpectedEndOfFile)
					} else {
						Ok(input.len())
					},
					Some(index) => {
						// note that end_pos is relative here;
						// the indices we just skipped we need to add back.
						// and- if we actually find the index of the newline,
						// we'll consume it too. that's where the +1 comes from.
						seek += index + 1;
					}
				}
			}
		} else {
			// non-whitespace and not a comment
			return Ok(seek);
		}
	}
	if fail_on_eof {
		Err(YuriParseError::UnexpectedEndOfFile)
	} else {
		Ok(seek)
	}
}

#[cfg(test)]
mod test {
	use crate::error::YuriParseError;
	use crate::parser::{take_whitespace, YuriTokenType};
	use crate::YuriShader;

	fn cvc(s: &str) -> Vec<char> {
		s.chars().collect()
	}

	#[test]
	fn lex_numbers() {
		for _ in 0..1_345_678 {
			// TODO: numbers break when too big/small
			let val = rand::random_range(-123456789.0f32..=123456789.0f32);
			let vas = val.to_string();
			let ast = YuriShader::lex(&vas).unwrap();
			assert_eq!(ast.len(), 1);
			let tt = &ast[0].token_type;
			match tt {
				YuriTokenType::DecimalNumber(n) => { assert_eq!(*n, val); }
				YuriTokenType::SignedNumber(n) => { assert_eq!(*n as f32, val); }
				YuriTokenType::UnsignedNumber(n) => { assert_eq!(*n as f32, val); }
				_ => unreachable!("not decimal: {tt:?} (from {val})")
			}
		}
	}

	#[test]
	fn verify_take_whitespace() {
		for s in [
			"",
			" ",
			"#",
			"# ",
			" #",
			" # ",
			"#a",
			" #a ",
			"# ##",
			" ## a# ## ",
			" ## a## ",
			" ## a# # ## ",
			" # ",
			"\n",
			"#\n",
			"# \n\n",
			" # \n",
			"#a\n",
			"\n# ##\n",
			" ## \na#\n \n ## ",
			" ## \na## ",
			" #\n ",
		] {
			println!("! testing \"{}\"", s.escape_default());
			let input = cvc(s);
			assert_eq!(
				take_whitespace(&input, 0, false),
				Ok(input.len()),
				"input was \"{}\"", s.escape_default()
			);
		}

		for s in [
			("a", 0),
			(" a", 1),
			("a# ", 0),
			(" #a\na ", 4),
			("## asdas#d## asd#", 13),
			("## asdasd## asd#", 12),
		] {
			println!("! testing \"{}\"", s.0);
			let input = cvc(s.0);
			assert_eq!(
				take_whitespace(&input, 0, false),
				Ok(s.1),
				"input was \"{}\"", s.0.escape_default()
			);
		}

		for s in [
			"##",
			"## asdasd# asd#",
			"## \n#",
			"## #",
			" ##",
			"\n#\n ##\n",
		] {
			println!("! testing \"{}\"", s.escape_default());
			let input = cvc(s);
			assert_eq!(
				take_whitespace(&input, 0, false),
				Err(YuriParseError::UnexpectedEndOfFile),
				"input was \"{}\"", s.escape_default()
			);
		}

		let s = "## asdas#d## asd#";
		println!("testing \"{}\"", s.escape_default());
		let input = cvc(s);
		assert_eq!(
			take_whitespace(&input, 0, false),
			Ok(13),
			"input was \"{}\"", s.escape_default()
		);
	}
}