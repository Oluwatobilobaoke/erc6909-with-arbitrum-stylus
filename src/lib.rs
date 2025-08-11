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
    event OperatorSet(address indexed owner, address indexed sender, bool approved);

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
      fn _approve(&mut self, owner: Address, spender: Address, id: U256,  value: U256) -> Result<(), ERC6909Error> {
        if owner.is_zero() {
            return Err(ERC6909Error::ERC6909InvalidSender(ERC6909InvalidSender {
                sender: owner,
            }));
        }

        if spender.is_zero() {
            return Err(ERC6909Error::ERC6909InvalidReciver(ERC6909InvalidReciver {
                reciver: spender,
            }));
        }

        self._allowances.setter(owner).setter(spender).insert(id, value);
        log(self.vm(), Approval { owner, spender, id, value });
        Ok(())
    }

    fn _set_operator(&mut self, owner: Address, sender: Address, approved: bool) -> Result<(), ERC6909Error> {
        if owner.is_zero() {
            return Err(ERC6909Error::ERC6909InvalidSender(ERC6909InvalidSender {
                sender: owner,
            }));
        }

        if sender.is_zero() {
            return Err(ERC6909Error::ERC6909InvalidSender(ERC6909InvalidSender {
                sender: sender,
            }));
        }

        self._operator_approvals.setter(owner).insert(sender, approved);
        log(self.vm(), OperatorSet { owner, sender, approved });
        Ok(())
    }

    fn _spend_allowance(&mut self, owner: Address, spender: Address, id: U256, value: U256) -> Result<(), ERC6909Error> {
        let allowance = self.allowance(owner, spender, id);
        if allowance < value {
            return Err(ERC6909Error::ERC6909InsufficientAllowance(ERC6909InsufficientAllowance {
                owner, allowance, needed: value, id
            }));
        }

        self._allowances.setter(owner).setter(spender).insert(id, allowance - value);
        Ok(())
    }

   fn _update(&mut self, from: Address, to: Address, id: U256, value: U256) -> Result<(), ERC6909Error> {
    // if from is not zero, check if from has enough balance
    if !from.is_zero() {
      let balance = self._balance.setter(from).get(id);
      if balance < value {
        return Err(ERC6909Error::ERC6909InsufficientBalance(ERC6909InsufficientBalance {
          sender: from, balance, needed: value, id
        }));
      }
      self._balance.setter(from).insert(id, balance - value);
    }

    if !to.is_zero() {
      let balance = self._balance.setter(to).get(id);
      self._balance.setter(to).insert(id, balance + value);
    }

    log(self.vm(), Transfer { from, to, id, value });
    Ok(())
  }

  fn _burn(&mut self, owner: Address, id: U256, value: U256) -> Result<(), ERC6909Error> {
    let balance = self._balance.setter(owner).get(id);
    if balance < value {
      return Err(ERC6909Error::ERC6909InsufficientBalance(ERC6909InsufficientBalance {
        sender: owner, balance, needed: value, id
      }));
    }
    self._update(owner, Address::ZERO, id, value)?;
    Ok(())
  }

  fn _mint(&mut self, to: Address, id: U256, value: U256) -> Result<(), ERC6909Error> {
    self._update(Address::ZERO, to, id, value)?;
    Ok(())
  }

  fn _transfer(&mut self, from: Address, to: Address, id: U256, value: U256) -> Result<(), ERC6909Error> {
    if from.is_zero() {
      return Err(ERC6909Error::ERC6909InvalidSender(ERC6909InvalidSender {
        sender: from,
      }));
    }

    if to.is_zero() {
      return Err(ERC6909Error::ERC6909InvalidReciver(ERC6909InvalidReciver {
        reciver: to,
      }));
    }

    self._update(from, to, id, value)?;
    Ok(())
  }
        
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

   pub fn is_operator(&self, owner: Address, spender: Address) -> bool {
    self._operator_approvals.getter(owner).get(spender)
   }

   pub fn approve(&mut self, spender: Address, id: U256, value: U256) -> Result<(), ERC6909Error> {
    self._approve(self.vm().msg_sender(), spender, id, value)
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
