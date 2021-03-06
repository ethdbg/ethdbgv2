use failure::Error;
use log::*;
use lunarity_ast::{Program, SourceUnit, ContractPart, FunctionDefinition};
use super::err::SolidityError;
use crate::{Ast, CharOffset, AstItem, AstType, AbstractFunction, Mutation, SourceRange};

pub struct SolidityAst<'ast> {
    program: Program<'ast>,
}

impl<'ast> SolidityAst<'ast> {
    pub fn new(source: &str) -> Result<Self, SolidityError> {
        let program: Program<'ast> = lunarity_parser::parse(source)
            .map_err(|e| SolidityError::AstParse(format!("{:?}", e)))?;
        Ok(Self { program })
    }
}


impl<'ast> Ast for SolidityAst<'ast> {

    /// get a variable declaration
    fn variable(&self, name: &str) -> Result<AstItem, Error> {
        unimplemented!();
    }

    /// Get a contract declaration
    fn contract(&self, name: &str) -> Result<AstItem, Error> {
        unimplemented!();
    }

    /// Access a Function via a Closure
    fn function(&self, name: &str, fun: &mut FnMut(Result<&AbstractFunction, Error>) -> bool) -> Result<AstItem, Error> {
        unimplemented!();
    }


    /// Find a contract a character offset in the source file
    fn find_contract(&self, offset: CharOffset) -> Option<AstItem> {

        for node in self.program.body().iter() {
            match node.value {
                SourceUnit::ContractDefinition(c) => {
                    info!("Node {} {}", node.start, node.end);
                    if offset >= node.start as usize && offset <=node.end as usize {
                        info!("Body: {} {}", c.body.first_element()?.start, c.body.first_element()?.end);
                        return Some(AstItem {
                            variant: AstType::Contract,
                            name: c.name.value.to_string(),
                            location: (node.start as usize, node.end as usize)
                        });
                    }
                },
                _ => (),
            }
        }
        None
    }

    fn find_function(&self, fun: &mut FnMut(&AbstractFunction) -> bool) -> Option<AstItem> {
        for node in self.program.body().iter() {
            match node.value {
                SourceUnit::ContractDefinition(c) => {
                    for cnode in c.body.iter() {
                        match cnode.value {
                            ContractPart::FunctionDefinition(f) => {
                                info!("Observing Function {:?} with parameters {:?} at {} - {}",
                                      f.name, f.params, cnode.start, cnode.end);
                                if fun(&f as &AbstractFunction) {
                                    return Some(AstItem {
                                        variant: AstType::Function,
                                        name: f.name.unwrap().value.to_string(),
                                        location: (cnode.start as usize, cnode.end as usize)
                                    });
                                }
                            },
                            _ => (),
                        }
                    }
                },
                _=> (),
            }
        }
        None
    }
}

impl<'ast> AbstractFunction for FunctionDefinition<'ast> {

    /// Name of the function
    fn name(&self) -> String {
        // TODO: dig through lunarity to figure out why name would ever be empty
        self.name.expect("Name should never be empty; qed").value.to_string()
    }

    /// Parameters of function
    fn params(&self) -> ethabi::Param {
        unimplemented!();
    }

    /// Function Returns
    fn returns(&self) -> ethabi::Param {
        unimplemented!();
    }

    /// Any mutations to state that occur within the function
    fn mutations(&self) -> Box<Iterator<Item=Mutation>> {
        unimplemented!();
    }

    /// Source Location range of function
    fn location(&self) -> SourceRange {
        (
            self.block.expect("Should not be empty; qed").start as usize,
            self.block.expect("Should not be empty; qed").end as usize
        )
    }
}

#[cfg(test)]

mod tests {
    use super::*;
    use speculate::speculate;
    const TEST_CONTRACT:&'static str = include_str!("../test_files/simple.sol");
    speculate! {
        before {
            #[allow(unused_must_use)] {
                pretty_env_logger::try_init();
            }
            let ast = SolidityAst::new(TEST_CONTRACT).unwrap();
        }

        it "can find a contract" {
            let contract = ast.find_contract(150).unwrap();
            assert_eq!(contract, AstItem {
                name: "SimpleStorage".to_string(),
                variant: AstType::Contract,
                location: (25, 591)
            });
        }

        it "can find a function" {
            let function = ast.find_function(&mut |func| {
                let (start, end) = func.location();
                if start <= 200 && end >= 200 {
                    return true;
                }
                false
            });

            assert_eq!(function, Some(AstItem {
                name: "set".to_string(),
                variant: AstType::Function,
                location: (150, 510)
            }));
        }
    }
}


