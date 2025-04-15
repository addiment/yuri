use crate::error::{YuriCompileError, YuriParseError, YuriSemanticError};
use crate::parser::{ShaderModule, YuriAst};

pub mod error;
mod codegen;
pub mod parser;

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

    pub fn lex(input: &str) -> Result<YuriAst, YuriParseError> {
        parser::lex(input)
    }

    pub fn parse(input: &YuriAst) -> Result<ShaderModule, YuriParseError> {
        todo!()
    }

    pub fn compile(ast: &ShaderModule) -> Result<Self, YuriSemanticError> {
        todo!()
    }
}