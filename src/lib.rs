#![cfg_attr(not(any(feature = "export-abi", test)), no_main)]
extern crate alloc;

mod erc721;
mod ownable;

use crate::erc721::{Erc721, Erc721Error, Erc721Params};
use crate::ownable::Ownable;
use alloy_primitives::{Address, Uint, U256};
use erc721::{InvalidSaleSetup, MaxMintForAddress, SaleNotOpen};
use ownable::OwnableError;
use stylus_sdk::{block, msg, prelude::*};

#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

struct PhysicalRedeemParams;
impl Erc721Params for PhysicalRedeemParams {
    const NAME: &'static str = "Infinite Rainbows";
    const SYMBOL: &'static str = "IR";

    fn token_uri(token_id: U256) -> String {
        "https://arweave.net/Lphk33KAeogfxwWtc4X05UazApLO8ohbHVctV7_rkS4".to_string()
    }
}

sol_storage! {
    #[entrypoint]
    struct PhysicalRedeem {
        #[borrow] // Allows erc721 to access StylusTestNFT's storage and make calls
        Erc721<PhysicalRedeemParams> erc721;
        #[borrow]
        Ownable ownable;
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

pub enum CombinedError {
    Erc721Error(Erc721Error),
    OwnableError(OwnableError),
}

impl From<Erc721Error> for CombinedError {
    fn from(err: Erc721Error) -> Self {
        CombinedError::Erc721Error(err)
    }
}

impl From<OwnableError> for CombinedError {
    fn from(err: OwnableError) -> Self {
        CombinedError::OwnableError(err)
    }
}

impl From<CombinedError> for Vec<u8> {
    fn from(err: CombinedError) -> Vec<u8> {
        match err {
            CombinedError::OwnableError(e) => OwnableError::into(e),
            CombinedError::Erc721Error(e) => Erc721Error::into(e),
        }
    }
}

#[external]
#[inherit(Erc721<PhysicalRedeemParams>)]
impl PhysicalRedeem {
    /// Mints an NFT
    pub fn mint(&mut self) -> Result<(), CombinedError> {
        let minter = msg::sender();

        let current_time = block::timestamp();
        if self.sale_info.start_time.get() == U256::from(0)
            || Uint::<256, 4>::from(current_time) < self.sale_info.start_time.get()
            || Uint::<256, 4>::from(current_time) >= self.sale_info.end_time.get()
        {
            return Err(CombinedError::Erc721Error(Erc721Error::SaleNotOpen(
                SaleNotOpen {},
            )));
        }

        let mut num_minted = self.num_minted_by_address.setter(msg::sender());
        if self.sale_info.max_per_address.get() - num_minted.get() == U256::from(0) {
            return Err(CombinedError::Erc721Error(Erc721Error::MaxMintForAddress(
                MaxMintForAddress {},
            )));
        }

        let new_num_minted = num_minted.get() + U256::from(1);
        num_minted.set(new_num_minted);
        self.erc721.mint(minter)?;
        Ok(())
    }

    pub fn redeem_physical(&mut self, token_id: U256) -> Result<(), Erc721Error> {
        self.erc721.burn(msg::sender(), token_id)?;
        Ok(())
    }

    pub fn admin_redeem_physical(
        &mut self,
        user: Address,
        token_id: U256,
    ) -> Result<(), CombinedError> {
        self.ownable.only_owner()?;
        self.erc721.burn(user, token_id)?;
        Ok(())
    }

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
    ) -> Result<(), CombinedError> {
        self.ownable.only_owner()?;
        if start_time == U256::from(0)
            || end_time == U256::from(0)
            || max_per_address == U256::from(0)
        {
            return Err(CombinedError::Erc721Error(Erc721Error::InvalidSaleSetup(
                InvalidSaleSetup {},
            )));
        }

        if start_time >= end_time {
            return Err(CombinedError::Erc721Error(Erc721Error::InvalidSaleSetup(
                InvalidSaleSetup {},
            )));
        }

        self.sale_info.start_time.set(start_time);
        self.sale_info.end_time.set(end_time);
        self.sale_info.price.set(price);
        self.sale_info.max_per_address.set(max_per_address);
        Ok(())
    }

    pub fn num_minted_by_address(&self, address: Address) -> Result<U256, Erc721Error> {
        Ok(self.num_minted_by_address.get(address))
    }

    pub fn initialize(&mut self) -> Result<(), OwnableError> {
        self.ownable.initialize(msg::sender())?;
        Ok(())
    }
}
