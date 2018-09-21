use super::{SourceMap, Language, err::{LanguageError, SourceMapError}};
use std::{
    boxed::Box,
    str::FromStr,
    marker::PhantomData,
};
use codespan::{ CodeMap, FileMap, FileName, ByteIndex, LineIndex };

/// Functions for Bytecode source map
pub struct BytecodeSourceMap{
    code_map: CodeMap,
    map: Vec<Mapping>
}


impl BytecodeSourceMap {
    pub fn new(lang: &Language<Err=Into<LanguageError>>) -> Self {
        let code_map = CodeMap::new();
        code_map.add_filemap(FileName::Real(lang.file_path()), lang.source.into());
        Self {

        }
    }
}

#[derive(Debug)]
/// represents the source mapping of one file and contract
pub struct Mapping {
    /// File mapping is stored in
    file: String,
    /// ContractName of mapping
    contract_name: String,
    index: usize,
    map: Box<dyn SourceMap<Err=Into<LanguageError>>>,
}

/// struct representing one bytecode instruction and it's position in the source code
pub struct Instruction {
    /// Start Byte  offset in source
    pub start: usize,
    /// Length of code in source
    pub length: usize,
    /// Index of file in Solidity Compiler Output
    pub source_index: SourceIndex,
    /// Type of jump, if any
    pub jump: Jump,
    /// Position of this in bytecode (as an offset)
    pub position: usize,
}

impl From<(usize, usize, SourceIndex, Jump, usize)> for Instruction {
    fn from(values: (usize, usize, SourceIndex, Jump, usize)) -> Instruction {
        Instruction {
            start: values.0,
            length: values.1,
            source_index: values.2,
            jump: values.3,
            position: values.4
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SourceIndex {
    NoSource,
    Source(usize)
}

impl Default for SourceIndex {
    fn default() -> SourceIndex {
        SourceIndex::NoSource
    }
}

impl FromStr for SourceIndex {
    type Err = LanguageError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "-1" => Ok(SourceIndex::NoSource),
            _ => Ok(SourceIndex::Source(s.parse()?))
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Jump {
    IntoFunc,
    ReturnFunc,
    NormJump,
}

impl Default for Jump {
    fn default() -> Jump {
        Jump::NormJump
    }
}

impl ToString for Jump {
    fn to_string(&self) -> String {
        match self {
            Jump::IntoFunc   => "i".to_string(),
            Jump::ReturnFunc => "o".to_string(),
            Jump::NormJump   => "-".to_string(),
        }
    }
}

impl FromStr for Jump {
    type Err = LanguageError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "i" => Ok(Jump::IntoFunc),
            "o" => Ok(Jump::ReturnFunc),
            "-" => Ok(Jump::NormJump),
            u @ _ => {
                Err(LanguageError::SourceMap(SourceMapError::UnknownJumpVariant(u.to_string())))
            }
        }
    }
}
