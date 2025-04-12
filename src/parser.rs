use crate::error::YuriParseError;

#[derive(Debug)]
enum Else {
	Block(Vec<Statement>),
	If(Box<IfExpression>),
}

#[derive(Debug)]
pub enum CompositeSize {
	Two,
	Three,
	Four
}

#[derive(Debug)]
pub enum NumberType {
	Float,
	Signed,
	Unsigned,
}

#[derive(Debug)]
pub enum YuriType {
	Unit,
	Scalar(NumberType),
	Vector(NumberType, CompositeSize),
	Array(Box<YuriType>, usize),
	Complex(Vec<(String, YuriType)>)
}

// "if" statements are incredibly annoying syntactically.
// I wish I could put this inside an enum variant, but I need two extra structs!
#[derive(Debug)]
struct IfExpression {
	condition: Box<Expression>,
	block: Vec<Statement>,
	block_else: Option<Else>,
}

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
struct VariableDeclaration {
	name: String,
	explicit_type: Option<YuriType>,
	inferred_type: Option<YuriType>,
	value: Expression,
	exported: bool,
}

/// A statement is a syntax element that can only occur in blocks.
#[derive(Debug)]
enum Statement {
	Expression(Expression),
	Variable(VariableDeclaration),
	Return(Expression),
}

#[derive(Debug)]
struct FunctionDeclaration {
	name: String,
	return_type: YuriType,
	arguments: Vec<(String, YuriType)>,
	exported: bool,
}

#[derive(Debug)]
struct PropertyDeclaration {
	name: String,
	property_type: YuriType,
}

#[derive(Debug)]
struct ImportDeclaration {
	module: String,
}

#[derive(Default)]
pub struct ShaderModule {
	imports: Vec<ImportDeclaration>,
	properties: Vec<PropertyDeclaration>,
	globals: Vec<VariableDeclaration>,
	functions: Vec<FunctionDeclaration>,
}

// TODO: implement a system to heuristically determine what might be fishy
struct ParseHesitation {}

pub fn parse(input: &str) -> Result<ShaderModule, YuriParseError> {
	let mut state = ShaderModule::default();

	let chars: Vec<char> = input.chars().collect();
	let mut seek = 0;
	let mut last_seek = usize::MAX;
	while seek < chars.len() {
		if seek == last_seek {
			return Err(YuriParseError::ParserBug {
				explanation: format!(concat!(
					"The parser got stuck while parsing a token, ",
					"and can no longer complete. For developers: ",
					"make sure that the input seek is set correctly! ",
					"The current seek is {}"
				), seek)
			})
		}
		last_seek = seek;
		state.parse_outer(&chars, &mut seek)?;
	}

	Ok(state)
}

type Input<'a> = &'a Vec<char>;

impl ShaderModule {
	/// Moves the seek forward until it hits a non-whitespace token.
	/// If the function encounters comments, it will treat them as whitespace.
	/// Block comments will generate a parse error if they are not terminated before EOF.
	/// Will return true if once it hits
	fn take_whitespace(input: Input, seek: &mut usize, fail_on_eof: bool) -> Result<bool, YuriParseError> {
		// ParseHesitation::EndOfFile
		while *seek < input.len() {
			let ch = input.get(*seek);
			if ch.is_none() {
				// eof
				return if fail_on_eof {
					Err(YuriParseError::UnexpectedEndOfFile)
				} else {
					Ok(false)
				}
			}
			let ch = *ch.unwrap();
			if ch.is_whitespace() {
				*seek += 1;
				// keep going
				continue;
			} else if ch == '#' {
				// comment, line or block?
				let next = if let Some(next) = input.get(*seek + 1) {
					// seek to the next character since we know it exists
					*seek += 1;
					*next
				} else {
					// edge case where we have a hash at the VERY END of the file.
					// why would anyone do this???
					// it's technically valid though so let's make sure we validate it
					return if fail_on_eof {
						Err(YuriParseError::UnexpectedEndOfFile)
					} else {
						Ok(false)
					}
				};

				if next == '#' {
					// block comment
					loop {
						let end_pos = input.iter()
							.skip(*seek)
							.position(|c| *c == '#');
						if end_pos.is_none() {
							// EOF
							return Err(YuriParseError::UnexpectedEndOfFile);
						}
						let end_pos = end_pos.unwrap();
						// we seek to the position where we found the next hash.
						// note that this is NOT the character after,
						// we account for that when we assign the next character
						*seek += end_pos;
						// make sure there's still no EOF
						let next = if let Some(next) = input.get(*seek + 1) {
							// seek to next character
							*seek += 1;
							*next
						} else {
							// EOF
							return Err(YuriParseError::UnexpectedEndOfFile);
						};

						if next == '#' {
							// move the seek to after the block comment ends,
							// or else we'll interpret it as the beginning of another comment.
							*seek += 1;
							break;
						}
					}
				} else {
					// line comment
					let end_pos = input.iter()
						.skip(*seek)
						.position(|c| *c == '\n');
					match end_pos {
						// EOF
						None => return if fail_on_eof {
							Err(YuriParseError::UnexpectedEndOfFile)
						} else {
							Ok(false)
						},
						Some(index) => {
							// note that end_pos is relative here;
							// the indices we just skipped we need to add back.
							// and- if we actually find the index of the newline,
							// we'll consume it too. that's where the +1 comes from.
							*seek += index + 1;
						}
					}
				}
			} else {
				// non-whitespace and not a comment
				return Ok(true);
			}
		}
		Ok(false)
	}

	/// Parses an outermost declaration in the shader.
	/// This can be any one of:
	/// - an import statement
	/// - a function declaration
	/// - a global property (aka. uniform)
	/// - a global variable (constant)
	fn parse_outer(&mut self, input: Input, seek: &mut usize) -> Result<(), YuriParseError> {


		// let function_decl = self.parse_function_declaration(input, seek);
		// let variable_decl = self.parse_function_declaration(input, seek);
		Ok(())
	}

	// fn parse_function_declaration(input: Input, seek: &mut usize) -> Result<FunctionDeclaration, YuriParseError> {
	//
	// }
	//
	fn parse_variable_declaration(input: Input, mut seek: usize) -> Result<VariableDeclaration, YuriParseError> {
		Self::take_whitespace(input, &mut seek, true)?;
		let is_exported = if input[seek] == 'e' {
			// make sure the keyword matches
			let keyword: Box<[char]> = "export".chars().collect();
			if *keyword == input[seek..(seek + keyword.len())] {
				// if we get to this
				seek += keyword.len();
				Self::take_whitespace(input, &mut seek, true)?
			} else {
				// if your variable declaration doesn't start with "let" or "export," it's invalid.
				return Err(YuriParseError::InvalidVariableDeclaration)
			}
		} else {
			false
		};
		Self::take_whitespace(input, &mut seek, true)?;
		Ok(VariableDeclaration {
			name: "".to_string(),
			explicit_type: None,
			inferred_type: None,
			value: Expression::Variable(String::from("TODO")),
			exported: is_exported
		})
	}
}

#[cfg(test)]
mod test {
	use crate::error::YuriParseError;
	use crate::parser::ShaderModule;

	fn cvc(s: &str) -> Vec<char> {
		s.chars().collect()
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
			let input = cvc(s);
			let mut seek = 0;
			assert_eq!(
				ShaderModule::take_whitespace(&input, &mut seek, false),
				Ok(false),
				"input was \"{s}\""
			);
		}

		for s in [
			"a",
			" a",
			"a# ",
			" #a\na ",
			"## asdas#d## asd#",
			"## asdasd## asd#",
		] {
			println!("testing \"{s}\"");
			let input = cvc(s);
			let mut seek = 0;
			assert_eq!(
				ShaderModule::take_whitespace(&input, &mut seek, false),
				Ok(true),
				"input was \"{s}\""
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
			println!("testing \"{s}\"");
			let input = cvc(s);
			let mut seek = 0;
			assert_eq!(
				ShaderModule::take_whitespace(&input, &mut seek, false),
				Err(YuriParseError::UnexpectedEndOfFile),
				"input was \"{s}\""
			);
		}

		let s = "## asdas#d## asd#";
		println!("testing \"{s}\"");
		let input = cvc(s);
		let mut seek = 0;
		assert_eq!(
			ShaderModule::take_whitespace(&input, &mut seek, false),
			Ok(true),
			"input was \"{s}\""
		);
		assert_eq!(
			seek,
			13
		);
	}

	// #[test]
	// fn verify_take_variable() {
	// 	{
	// 		let s = "export a = 123;";
	// 		let input = cvc(s);
	// 		let res = ShaderModule::parse_variable_declaration(&input, 0);
	// 		assert_eq!(
	// 			res,
	// 			Ok(false),
	// 			"input was \"{s}\""
	// 		);
	// 	}
	// }
}