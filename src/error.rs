use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

/// Represents a generic error that occurred while trying to compile a Yuri shader.
pub enum YuriCompileError {
	Parse(YuriParseError),
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

impl From<YuriParseError> for YuriCompileError {
	fn from(value: YuriParseError) -> Self {
		Self::Parse(value)
	}
}

impl From<YuriSemanticError> for YuriCompileError {
	fn from(value: YuriSemanticError) -> Self {
		Self::Semantic(value)
	}
}

/// Represents an error that occurred while processing the logical aspects of a Yuri syntax tree.
#[derive(Debug)]
pub enum YuriSemanticError {
}

impl Display for YuriSemanticError {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		// TODO
		f.write_fmt(format_args!(""))
	}
}

impl Error for YuriSemanticError {}

/// Represents an error that occurred while parsing the Yuri shader syntax from string input.
#[derive(Debug)]
pub enum YuriParseError {
	Unknown
}

impl Display for YuriParseError {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		// TODO
		f.write_fmt(format_args!(""))
	}
}

impl Error for YuriParseError {}