use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::ops::Range;

/// Represents a generic error that occurred while trying to compile a Yuri shader.
#[derive(Eq, PartialEq)]
pub enum YuriCompileError {
	Parse(YuriLexError),
	Semantic(YuriSemanticError)
}

impl Debug for YuriCompileError {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		match self {
			YuriCompileError::Parse(err) => Debug::fmt(err, f),
			YuriCompileError::Semantic(err) => Debug::fmt(err, f),
		}
	}
}

impl Display for YuriCompileError {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		match self {
			YuriCompileError::Parse(err) => Display::fmt(err, f),
			YuriCompileError::Semantic(err) => Display::fmt(err, f),
		}
	}
}

impl Error for YuriCompileError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		Some(match self {
			YuriCompileError::Parse(err) => err,
			YuriCompileError::Semantic(err) => err
		})
	}
}

impl From<YuriLexError> for YuriCompileError {
	fn from(value: YuriLexError) -> Self {
		Self::Parse(value)
	}
}

impl From<YuriSemanticError> for YuriCompileError {
	fn from(value: YuriSemanticError) -> Self {
		Self::Semantic(value)
	}
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum YuriSemanticErrorType {
	UnexpectedToken	
}

/// Represents an error that occurred while processing the logical aspects of a Yuri syntax tree.
#[derive(Debug, Eq, PartialEq)]
pub struct YuriSemanticError {
	pub(crate) error_type: YuriSemanticErrorType,
	pub(crate) description: Option<String>,
	pub(crate) markers: Vec<Range<usize>>,
}

impl Display for YuriSemanticError {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		// TODO
		f.write_fmt(format_args!(""))
	}
}

impl Error for YuriSemanticError {}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum YuriLexErrorType {
	UnknownToken,
	InvalidVariableDeclaration,
	UnexpectedEndOfFile,
	NumberOutOfBounds,
	InvalidNumericLiteral,
	IncompleteAnnotation,
}

/// Represents an error that occurred while parsing the Yuri shader syntax from string input.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct YuriLexError {
	pub(crate) error_type: YuriLexErrorType,
	pub(crate) description: Option<String>,
	pub(crate) markers: Vec<Range<usize>>,
}

impl YuriLexError {
	pub fn error_type(&self) -> YuriLexErrorType {
		self.error_type
	}
}

impl Display for YuriLexError {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		// TODO
		f.write_fmt(format_args!(""))
	}
}

impl Error for YuriLexError {}