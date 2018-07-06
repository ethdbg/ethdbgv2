// Copyright 2015-2018 Andrew Plaa (U.S.A) Ltd.
// This file is part of EDB.
//
// EDB is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// EDB is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with EDB. If not, see <http://www.gnu.org/licenses/>.


//! An Extension to the parity interpreter for debugging 

use vm;
use evm::{CostType};
use evm::interpreter::{Interpreter, SharedCache, InterpreterResult};
use vm::{ActionParams};
use vm::{Vm, GasLeft};
//use vm::tests::{FakeExt};
use std::sync::Arc;


/// A wrapper around Parity's Evm Interpreter implementation
pub struct InterpreterExt<Cost: CostType> {
    interpreter: Interpreter<Cost>,
    cache: Arc<SharedCache>,
    params: ActionParams,
    pub pos: usize,
}

impl<Cost: CostType> InterpreterExt<Cost> {
    
    pub fn new(params: ActionParams, cache: Arc<SharedCache>, ext: &vm::Ext)
        -> vm::Result<InterpreterExt<Cost>> 
    {
        Ok(InterpreterExt {
            params: params.clone(),
            cache: cache.clone(),
            interpreter: Interpreter::new(params, cache, ext).unwrap(),
            pos: 0,
        })
    }

    /// runs code without stopping at any position
    // pass through for vm::Vm exec
    pub fn run_code(&mut self, ext: &mut vm::Ext) -> vm::Result<GasLeft> {
        self.interpreter.exec(ext)
    }
    
    /// go back in execution to a position
    // actually just restarts vm until a pos
    // the most inefficient function so far
    pub fn step_back(&mut self, pos: usize, ext: &mut vm::Ext) {
        // Might be an issue, if cache isn't really a cache and used as a 
        // reference in Parity somewhere
        self.interpreter = Interpreter::new(self.params.clone(), self.cache.clone(), ext).unwrap();
        let new_pos: usize = self.pos - pos;
        self.pos = 0;
        self.run_code_until(ext, new_pos);
    }

    /// run code until a byte position
    /// stops before byte position
    pub fn run_code_until(&mut self, ext: &mut vm::Ext, pos: usize) 
        -> Option<vm::Result<GasLeft>>
    {
        while self.pos < pos {
            let result = self.interpreter.step(ext);
            match result {
                InterpreterResult::Continue => {},
                InterpreterResult::Done(value) => return Some(value),
                InterpreterResult::Stopped 
                    => panic!("Attempted to execute an already stopped VM.")
            }
            self.pos += 1;
        }
        None
    }
}

// some tests taken from ethcore::evm::tests
#[cfg(test)]
mod tests {
    use ethereum_types::{U256, H256, Address};
    use rustc_hex::FromHex;
    use vm::tests::{FakeExt, FakeCall, FakeCallType, test_finalize};
    use vm::{ActionParams};
    use std::sync::Arc;
    use evm::interpreter::{SharedCache};
    use std::str::FromStr;

    #[test]
    fn it_should_run_code() {
        let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
	    let code = "7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff01600055".from_hex().unwrap();

        let mut params = ActionParams::default();
        params.address = address.clone();
        params.gas = U256::from(100_000);
        params.code = Some(Arc::new(code));
        let mut ext = FakeExt::new();
        let cache = Arc::new(SharedCache::default());
        
        let gas_left = {
            let mut vm = super::InterpreterExt::<usize>::new(params, cache.clone(), &ext).unwrap();
            test_finalize(vm.run_code(&mut ext)).unwrap()
        };

        assert_eq!(gas_left, U256::from(79_988));
    //    assert_store(&ext, 0, "fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffe");
    }
    
    // just random code
    // contains bad instruction
    // this code segment becomes important in InstructionManager and Emulator
    #[test]
    #[should_panic]
    fn it_should_run_solidity_basic() {
        let address = Address::from_str("0f572e5295c57f15886f9b263e2f6d2d6c7b5ec6").unwrap();
        let code = "60806040526004361061006d576000357c0100000000000000000000000000000000000000000000000000000000900463ffffffff16806360fe47b1146100725780636d4ce63c1461009f5780639fc8192c146100ca578063c2d2c2ea146100f7578063dffeadd014610122575b600080fd5b34801561007e57600080fd5b5061009d60048036038101908080359060200190929190505050610139565b005b3480156100ab57600080fd5b506100b461014d565b6040518082815260200191505060405180910390f35b3480156100d657600080fd5b506100f560048036038101908080359060200190929190505050610156565b005b34801561010357600080fd5b5061010c610179565b6040518082815260200191505060405180910390f35b34801561012e57600080fd5b50610137610183565b005b806000819055506001810160018190555050565b60008054905090565b80600281905550600a600254016002819055506000546002540360028190555050565b6000600154905090565b61018d6014610139565b6101976032610156565b5600a165627a7a7230582073220057da31267f028c5802e52e8b0f18aac96f30d1dcc4cc9c9d2cfe5b28d40029".from_hex().unwrap();

        let mut params = ActionParams::default();
        params.address = address.clone();
        params.gas = U256::from(100_000_000);
        params.code = Some(Arc::new(code));
        let mut ext = FakeExt::new();
        let cache = Arc::new(SharedCache::default());

        let gas_left = {
            let mut vm = super::InterpreterExt::<usize>::new(params, cache.clone(), &ext).unwrap();
            test_finalize(vm.run_code(&mut ext)).unwrap()
        };
        println!("Gas Left: {}", gas_left);
    }
}


