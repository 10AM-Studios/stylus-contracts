// Only run this as a WASM if the export-abi feature is not set.
#![cfg_attr(not(any(feature = "export-abi", test)), no_main)]
extern crate alloc;

// Modules and imports
mod erc721;

use crate::erc721::{Erc721, Erc721Error, Erc721Params};
use alloy_primitives::{Uint, U256};
use alloy_sol_types::sol;
use erc721::{MaxMintForAddress, SaleNotOpen};
/// Import the Stylus SDK along with alloy primitive types for use in our program.
use stylus_sdk::{block, msg, prelude::*};

/// Initializes a custom, global allocator for Rust programs compiled to WASM.
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

/// Immutable definitions
struct PhysicalRedeemParams;
impl Erc721Params for PhysicalRedeemParams {
    const NAME: &'static str = "PhysicalRedeem";
    const SYMBOL: &'static str = "SP";

    fn token_uri(token_id: U256) -> String {
        format!("{}{}{}", "https://my-nft-metadata.com/", token_id, ".json")
    }
}

// Define the entrypoint as a Solidity storage object. The sol_storage! macro
// will generate Rust-equivalent structs with all fields mapped to Solidity-equivalent
// storage slots and types.
sol_storage! {
    #[entrypoint]
    struct PhysicalRedeem {
        #[borrow] // Allows erc721 to access StylusTestNFT's storage and make calls
        Erc721<PhysicalRedeemParams> erc721;
        SaleInfo sale_info;
        mapping(address => uint256) num_minted_by_address;
    }

    struct SaleInfo {
        uint256 start_time;
        uint256 end_time;
        uint256 price;
        uint256 max_per_address;
    }
}

sol! {
    error InvalidSaleSetup();
}

#[derive(SolidityError)]
pub enum PhysicalRedeemError {
    InvalidSaleSetup(InvalidSaleSetup),
}

#[external]
#[inherit(Erc721<PhysicalRedeemParams>)]
impl PhysicalRedeem {
    /// Mints an NFT
    pub fn mint(&mut self) -> Result<(), Erc721Error> {
        let minter = msg::sender();

        let current_time = block::timestamp();
        if self.sale_info.start_time.get() == U256::from(0)
            || Uint::<256, 4>::from(current_time) < self.sale_info.start_time.get()
            || Uint::<256, 4>::from(current_time) >= self.sale_info.end_time.get()
        {
            return Err(Erc721Error::SaleNotOpen(SaleNotOpen {}));
        }

        let mut num_minted = self.num_minted_by_address.setter(msg::sender());
        if self.sale_info.max_per_address.get() - num_minted.get() == U256::from(0) {
            return Err(Erc721Error::MaxMintForAddress(MaxMintForAddress {}));
        }

        let new_num_minted = num_minted.get() + U256::from(1);
        num_minted.set(new_num_minted);
        self.erc721.mint(minter)?;
        Ok(())
    }

    /// Burns an NFT
    pub fn burn(&mut self, token_id: U256) -> Result<(), Erc721Error> {
        // This function checks that msg::sender() owns the specified token_id
        self.erc721.burn(msg::sender(), token_id)?;
        Ok(())
    }

    /// Total supply
    pub fn total_supply(&mut self) -> Result<U256, Erc721Error> {
        Ok(self.erc721.total_supply.get())
    }

    pub fn sale_info(&self) -> Result<(U256, U256, U256, U256), Erc721Error> {
        let start_time = self.sale_info.start_time.get();
        let end_time = self.sale_info.end_time.get();
        let price = self.sale_info.price.get();
        let max_per_address = self.sale_info.max_per_address.get();
        Ok((start_time, end_time, price, max_per_address))
    }

    pub fn set_sale_info(
        &mut self,
        start_time: U256,
        end_time: U256,
        price: U256,
        max_per_address: U256,
    ) -> Result<(), PhysicalRedeemError> {
        if start_time == U256::from(0)
            || end_time == U256::from(0)
            || max_per_address == U256::from(0)
        {
            return Err(PhysicalRedeemError::InvalidSaleSetup(InvalidSaleSetup {}));
        }

        if start_time >= end_time {
            return Err(PhysicalRedeemError::InvalidSaleSetup(InvalidSaleSetup {}));
        }

        self.sale_info.start_time.set(start_time);
        self.sale_info.end_time.set(end_time);
        self.sale_info.price.set(price);
        self.sale_info.max_per_address.set(max_per_address);
        Ok(())
    }
}
