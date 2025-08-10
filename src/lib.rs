// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::{string::String, vec::Vec};
use alloy_primitives::{Address, U256};
use alloy_sol_types::sol;
/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::prelude::*;

// Define some persistent storage using the Solidity ABI.
// `Counter` will be the entrypoint.
sol_storage! {
    #[entrypoint]
    pub struct ERC6909 {
      address owner;
      string name;
      string symbol;
      uint8 decimals; 
      mapping(address => mapping(uint256 => uint256)) _balance;
      mapping(address => mapping(address => bool)) _operator_approvals;
      mapping(address => mapping(address => mapping(uint256 => uint256))) _allowances;

    
    }
}

sol! {
    #[derive(Debug)]
    error ERC6909InsufficientBalance(address sender, uint256 balance, uint256 needed, uint256 id);
    
    #[derive(Debug)]
    error ERC6909InvalidSender(address sender);
    
    #[derive(Debug)]
    error ERC6909InvalidReciver(address reciver);

    #[derive(Debug)]
    error ERC6909InvalidApprover(address approver);
    #[derive(Debug)]
    error ERC6909InvalidSpender(address spender);
    
    #[derive(Debug)]
    error ERC6909InsufficientAllowance(address owner, uint256 allowance, uint256 needed, uint256 id);

    event Transfer(address indexed from, address indexed to, uint256 indexed id, uint256 value);
    event Approval(address indexed owner, address indexed spender, uint256 indexed id, uint256 value);
}

// Define the Rust-equivalent of the Solidity errors
#[derive(SolidityError, Debug)]
pub enum ERC6909Error {
    ERC6909InsufficientBalance(ERC6909InsufficientBalance),
    ERC6909InvalidSender(ERC6909InvalidSender),
    ERC6909InvalidReciver(ERC6909InvalidReciver),
    ERC6909InvalidApprover(ERC6909InvalidApprover),
    ERC6909InvalidSpender(ERC6909InvalidSpender),
    ERC6909InsufficientAllowance(ERC6909InsufficientAllowance),
}

impl ERC6909 {

}

/// Declare that `Counter` is a contract with the following external methods.
#[public]
impl ERC6909 {
   pub fn balance_of(&self, owner: Address, id: U256) -> U256 {
    self._balance.getter(owner).get(id)
   }

   pub fn allowance(&self, owner: Address, spender: Address, id: U256) -> U256 {
    self._allowances.getter(owner).getter(spender).get(id)
   }
   

}


#[cfg(test)]
mod test {
    use super::*;

    #[no_mangle]
    pub unsafe extern "C" fn emit_log(_pointer: *const u8, _len: usize, _: usize) {}

    #[test]
    fn test_erc6909() {
      use stylus_sdk::testing::*;
    }

    
}
