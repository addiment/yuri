use crate::error::{YuriCompileError, YuriParseError, YuriSemanticError};
use crate::parser::ShaderModule;

pub mod error;
mod codegen;
mod parser;

pub struct YuriShader {
    
}

impl YuriShader {
    /// Wrapper around the [YuriShader::parse] and [YuriShader::compile] methods,
    /// chaining them together in the simplest possible way.
    pub fn new(input: &str) -> Result<Self, YuriCompileError> {
        let ast = Self::parse(input)?;
        Ok(Self::compile(&ast)?)
    }

    pub fn parse(input: &str) -> Result<ShaderModule, YuriParseError> {
        parser::parse(input)
    }

    pub fn compile(ast: &ShaderModule) -> Result<Self, YuriSemanticError> {
        Ok(Self{})
    }
}