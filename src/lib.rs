use crate::error::{YuriCompileError, YuriLexError, YuriSemanticError};
use crate::lex::YuriAst;
use crate::parse::YuriModule;

pub mod error;
pub mod lex;
pub mod parse;
pub mod compile;

pub struct YuriShader {
    
}

impl YuriShader {
    /// Wrapper around the [YuriShader::lex], [YuriShader::parse] and [YuriShader::compile] methods,
    /// chaining them together in the simplest possible way.
    pub fn new(input: &str) -> Result<Self, YuriCompileError> {
        let ast = Self::lex(input)?;
        let shader = Self::parse(&ast)?;
        Ok(Self::compile(&shader)?)
    }

    pub fn lex(input: &str) -> Result<YuriAst, YuriLexError> {
        lex::lex_input(input)
    }

    pub fn parse(input: &YuriAst) -> Result<YuriModule, YuriLexError> {
        todo!()
    }

    pub fn compile(ast: &YuriModule) -> Result<Self, YuriSemanticError> {
        todo!()
    }
}