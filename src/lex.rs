//! The Yuri lexer/tokenizer module. This was written entirely by hand,
//! ensuring maximum portability and even maximum-er jank.
use std::ops::{Neg, Range};
use crate::error::{YuriLexError, YuriLexErrorType};

type Input<'a> = &'a[char];

pub type YuriAst = Vec<YuriToken>;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Keyword {
	Fn,
	Let,
	Prop,

	Loop,
	Fold,
	Map,
	Filter,
	Switch,
	If,
	Else,
	Return,

	Import,
	Export,
	Module,

	/// Reserved
	Core,
	And,
	Xor,
	Or,
	Nor,

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
	pub const ALL: [Keyword; 35] = { use Keyword::*; [
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
		And,
		Xor,
		Or,
		Nor,

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
			Keyword::Fold 			=> "fold",
			Keyword::Filter 		=> "filter",
			Keyword::Import 		=> "import",
			Keyword::Export 		=> "export",
			Keyword::Module 		=> "module",
			Keyword::Return 		=> "return",
			Keyword::If 			=> "if",
			Keyword::And 			=> "and",
			Keyword::Xor 			=> "xor",
			Keyword::Or 			=> "or",
			Keyword::Nor 			=> "nor",
			Keyword::Else	 		=> "else",
			Keyword::Switch 		=> "switch",
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
	/// A token that we can't fully recognize, usually caused by an error.
	Unknown(YuriLexError),
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
	/// ?
	Optional,

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

/// This function is an affront to god. But so are lesbians, so it doesn't really matter.
/// I am afraid to split this up into multiple other functions because
/// of how the pieces all fit together. I totally could, but it's fine like this.
fn take_token(input: Input, seek: &mut usize) -> YuriToken {
	let initial_seek = *seek;
	let tt = match input[*seek] {
		'(' => { *seek += 1; YuriTokenType::OpenParen },
		')' => { *seek += 1; YuriTokenType::CloseParen },
		'{' => { *seek += 1; YuriTokenType::OpenBrace },
		'}' => { *seek += 1; YuriTokenType::CloseBrace },
		'[' => { *seek += 1; YuriTokenType::OpenSquare },
		']' => { *seek += 1; YuriTokenType::CloseSquare },
		':' => { *seek += 1; YuriTokenType::TypeHint },
		';' => { *seek += 1; YuriTokenType::Terminator },
		',' => { *seek += 1; YuriTokenType::Separator },
		'+' => { *seek += 1; YuriTokenType::Operator(String::from("+")) }
		'/' => { *seek += 1; YuriTokenType::Operator(String::from("/")) }
		'^' => { *seek += 1; YuriTokenType::Operator(String::from("^")) }
		'!' => { *seek += 1; YuriTokenType::Operator(String::from("!")) }
		'%' => { *seek += 1; YuriTokenType::Operator(String::from("%")) }
		'?' => { *seek += 1; YuriTokenType::Optional }

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
		'*' => {
			*seek += 1;
			match input.get(*seek) {
				Some('*') => { *seek += 1; YuriTokenType::Operator(String::from("**")) }
				None | Some(_) => YuriTokenType::Operator(String::from("*"))
			}
		}
		'-' | '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
			let earliest_seek = *seek;
			// this control flow is really annoying to model without repetition.
			let ch = input[*seek];

			let subtraction_negation = if ch == '-' {
				if let Some(next) = input.get(*seek + 1) {
					if next.is_digit(10) {
						None
					} else {
						*seek += 1;
						Some(YuriTokenType::Operator(String::from("-")))
					}
				} else {
					*seek += 1;
					Some(YuriTokenType::Operator(String::from("-")))
				}
			} else {
				None
			};
			let special_number_formats = if ch == '0' {
				match input.get(*seek + 1) {
					// hex number
					Some('x') => {
						let hex_start_seek = *seek;
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
							return YuriToken::new(
								YuriTokenType::Unknown(YuriLexError {
									error_type: YuriLexErrorType::InvalidNumericLiteral,
									description: Some("0 or more digits must follow a hexadecimal literal prefix (written %)".to_string()),
									markers: vec![hex_start_seek..*seek],
								}),
								earliest_seek..(*seek + 1)
							);
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
							return YuriToken::new(
								YuriTokenType::Unknown(YuriLexError {
									error_type: YuriLexErrorType::NumberOutOfBounds,
									description: Some("The hexadecimal number % can't be stored in a 32-bit unsigned integer value.".to_string()),
									markers: vec![hex_start_seek..*seek],
								}),
								earliest_seek..(*seek + 1)
							);
						})
					}
					// binary number
					Some('b') => {
						let binary_start_seek = *seek;
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
							return YuriToken::new(
								YuriTokenType::Unknown(YuriLexError {
									error_type: YuriLexErrorType::InvalidNumericLiteral,
									description: Some("0 or more digits must follow a binary literal prefix (written %)".to_string()),
									markers: vec![binary_start_seek..*seek],
								}),
								earliest_seek..(*seek + 1)
							);
						} else if digits.len() > 32 {
							return YuriToken::new(
								YuriTokenType::Unknown(YuriLexError {
									error_type: YuriLexErrorType::NumberOutOfBounds,
									description: Some("The binary number % can't be stored in a 32-bit unsigned integer value.".to_string()),
									markers: vec![binary_start_seek..*seek],
								}),
								earliest_seek..(*seek + 1)
							);
						}
						let mut sum = 0;
						for i in 0..digits.len() {
							if digits[i] {
								sum |= 1 << (digits.len() - i - 1);
							}
						}
						Some(YuriTokenType::BinaryNumber(sum))
					}
					// EOF
					None => { *seek += 1; Some(YuriTokenType::UnsignedNumber(0)) },
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
							return YuriToken::new(
								YuriTokenType::Unknown(YuriLexError {
									error_type: YuriLexErrorType::InvalidNumericLiteral,
									description: Some("More than one decimal point found in numeric literal (first is %, next is %)".to_string()),
									markers: vec![*seek..(*seek + 1), number_start_seek..(number_start_seek + 1)],
								}),
								earliest_seek..(*seek + 1)
							);
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
							return YuriToken::new(
								YuriTokenType::Unknown(YuriLexError {
									error_type: YuriLexErrorType::NumberOutOfBounds,
									description: Some("The number % can't be stored in a 32-bit floating-point value.".to_string()),
									markers: vec![number_start_seek..*seek],
								}),
								earliest_seek..(*seek + 1)
							);
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
							return YuriToken::new(
								YuriTokenType::Unknown(YuriLexError {
									error_type: YuriLexErrorType::NumberOutOfBounds,
									description: Some("The number % can't be stored in a 32-bit signed integer value.".to_string()),
									markers: vec![number_start_seek..*seek],
								}),
								earliest_seek..(*seek + 1)
							);
						}
					} else {
						if let Ok(sum) = u32::try_from(sum) {
							YuriTokenType::UnsignedNumber(sum)
						} else {
							return YuriToken::new(
								YuriTokenType::Unknown(YuriLexError {
									error_type: YuriLexErrorType::NumberOutOfBounds,
									description: Some("The number % can't be stored in a 32-bit unsigned integer value.".to_string()),
									markers: vec![number_start_seek..*seek],
								}),
								earliest_seek..(*seek + 1)
							);
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
					return YuriToken::new(
						YuriTokenType::Unknown(YuriLexError {
							error_type: YuriLexErrorType::IncompleteAnnotation,
							description: Some("The annotation % is missing a proper identifier".to_string()),
							markers: vec![*seek..(*seek + 1)],
						}),
						*seek - 1..(*seek + 1)
					);
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
				// TODO: do a greedy check of this by recursively calling take_token until we stop,
				//		 that would prevent a chain of errors caused by a sequence of weird chars
				YuriTokenType::Unknown(YuriLexError {
					error_type: YuriLexErrorType::UnknownToken,
					description: Some(format!("Unexpected/unknown character \'{ch}\' %")),
					markers: vec![*seek..(*seek + 1)],
				})
			}
		}
	};
	YuriToken::new(tt, initial_seek..*seek)
}

pub(super) fn lex_input(input_string: &str) -> Result<YuriAst, YuriLexError> {
	let mut ast = YuriAst::new();
	let input: Vec<char> = input_string.chars().collect();
	let mut seek = take_whitespace(&input, 0)?;
	while seek < input.len() {
		let tok = take_token(&input, &mut seek);
		let tok = tok;
		ast.push(tok);

		seek = take_whitespace(&input, seek)?;
	}
	Ok(ast)
}

/// Moves the seek forward until it hits a non-whitespace token.
/// If the function encounters comments, it will treat them as whitespace.
/// Block comments will generate a lex error if they are not terminated before EOF.
/// Will return true if once it hits
fn take_whitespace(input: Input, mut seek: usize) -> Result<usize, YuriLexError> {
	while seek < input.len() {
		let ch = input[seek];
		if ch.is_whitespace() {
			// keep going.
			// safe since we know there's at least 1 char
			seek += 1;
			continue;
		} else if ch == '#' {
			seek += 1;

			// edge case where we have a hash at the VERY END of the file.
			// why would anyone do this???
			// it's technically valid though so let's make sure we validate it
			let next = if let Some(next) = input.get(seek) {
				*next
			} else {
				return Ok(input.len())
			};
			// comment, line or block?
			if next == '#' {
				let block_comment_start = seek;
				seek += 1;
				// block comment
				static ERROR_STRING: &str = "Missing closing block for block comment (started %). Add `##` to the end of the comment/file to fix this.";
				loop {
					if let Some(end_pos) = input.iter()
						.skip(seek)
						.position(|c| *c == '#') {
						seek += end_pos + 1;
					} else {
						// EOF
						return Err(YuriLexError {
							error_type: YuriLexErrorType::UnexpectedEndOfFile,
							description: Some(ERROR_STRING.to_string()),
							markers: vec![block_comment_start..block_comment_start + 2],
						});
					};

					// we seek to the position where we found the next hash.
					// note that this is NOT the character after,
					// we account for that when we assign the next character
					let next = match input.get(seek) {
						None => return Err(YuriLexError {
							error_type: YuriLexErrorType::UnexpectedEndOfFile,
							description: Some(ERROR_STRING.to_string()),
							markers: vec![block_comment_start..block_comment_start + 2],
						}),
						Some(ch) => *ch
					};

					if next == '#' {
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
					None => return Ok(input.len()),
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
	Ok(seek)
}

#[cfg(test)]
mod test {
	use crate::error::{YuriLexError, YuriLexErrorType};
	use crate::lex::{take_whitespace, YuriTokenType};
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
				take_whitespace(&input, 0),
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
				take_whitespace(&input, 0),
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
			let take = take_whitespace(&input, 0);
			assert_eq!(
				take.unwrap_err().error_type,
				YuriLexErrorType::UnexpectedEndOfFile,
				"input was \"{}\"", s.escape_default()
			);
		}

		let s = "## asdas#d## asd#";
		println!("testing \"{}\"", s.escape_default());
		let input = cvc(s);
		assert_eq!(
			take_whitespace(&input, 0),
			Ok(13),
			"input was \"{}\"", s.escape_default()
		);
	}
}