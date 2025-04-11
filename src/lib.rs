use crate::error::{YuriCompileError, YuriParseError, YuriSemanticError};

pub mod error;
mod codegen;

pub struct YuriShader {
    
}

#[derive(Debug)]
pub struct YuriAst {

}

impl YuriShader {
    pub fn new(input: &str) -> Result<Self, YuriCompileError> {
        let ast = Self::parse(input)?;
        Ok(Self::compile(&ast)?)
    }

    pub fn parse(input: &str) -> Result<YuriAst, YuriParseError> {
        Ok(YuriAst{})
    }

    pub fn compile(ast: &YuriAst) -> Result<Self, YuriSemanticError> {
        Ok(Self{})
    }
}