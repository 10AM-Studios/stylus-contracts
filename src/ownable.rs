use alloc::vec::Vec;
use stylus_sdk::{
    alloy_primitives::Address, alloy_sol_types::sol, call::MethodError, evm, msg, prelude::*,
};

const ZERO_ADDRESS: Address = Address::ZERO;

sol_storage! {
    pub struct Ownable {
        address owner;
        bool initialized;
    }
}

// Declare events and Solidity error types
sol! {
    event OwnershipTransferred(address indexed previous_owner, address indexed new_owner);

    error OwnableUnauthorizedAccount(address account);
    error OwnableInvalidOwner(address owner);
    error OwnableAlreadyInitialized();
}

#[derive(SolidityError)]
pub enum OwnableError {
    OwnableUnauthorizedAccount(OwnableUnauthorizedAccount),
    OwnableInvalidOwner(OwnableInvalidOwner),
    OwnableAlreadyInitialized(OwnableAlreadyInitialized),
}

// Internal methods
impl Ownable {
    pub fn only_owner(&mut self) -> Result<(), OwnableError> {
        if msg::sender() != self.owner.get() {
            return Err(OwnableError::OwnableUnauthorizedAccount(
                OwnableUnauthorizedAccount {
                    account: msg::sender(),
                },
            ));
        }

        Ok(())
    }

    pub fn transfer_ownership_impl(&mut self, new_owner: Address) {
        let old_owner = self.owner.get();
        self.owner.set(new_owner);
        evm::log(OwnershipTransferred {
            previous_owner: old_owner,
            new_owner: new_owner,
        });
    }
}

// External methods
#[public]
impl Ownable {
    pub fn renounce_ownership(&mut self) -> Result<(), OwnableError> {
        self.only_owner()?;

        self.transfer_ownership_impl(ZERO_ADDRESS);
        Ok(())
    }

    pub fn transfer_ownership(&mut self, new_owner: Address) -> Result<(), OwnableError> {
        self.only_owner()?;
        self.transfer_ownership_impl(new_owner);
        Ok(())
    }

    pub fn owner(&self) -> Result<Address, Vec<u8>> {
        Ok(self.owner.get())
    }

    pub fn initialize(&mut self, initial_owner: Address) -> Result<(), OwnableError> {
        if self.initialized.get() {
            return Err(OwnableError::OwnableAlreadyInitialized(
                OwnableAlreadyInitialized {},
            ));
        }
        if initial_owner == ZERO_ADDRESS {
            return Err(OwnableError::OwnableInvalidOwner(OwnableInvalidOwner {
                owner: initial_owner,
            }));
        }
        self.transfer_ownership_impl(initial_owner);
        self.initialized.set(true);
        Ok(())
    }
}
