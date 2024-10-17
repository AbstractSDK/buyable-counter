use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cosmwasm_std::Empty;
use cw_orch::interface;
use cw_orch::prelude::*;

#[interface(InstantiateMsg, ExecuteMsg, QueryMsg, Empty, id = "buyable-counter")]
pub struct BuyableCounterI;

impl<Chain: CwEnv> Uploadable for BuyableCounterI<Chain> {
    /// Return the path to the wasm file corresponding to the contract
    fn wasm(_info: &ChainInfoOwned) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("buyable_counter")
            .unwrap()
    }
    /// Returns a CosmWasm contract wrapper
    fn wrapper() -> Box<dyn MockContract<Empty>> {
        Box::new(ContractWrapper::new_with_empty(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        ))
    }
}

impl<Chain: CwEnv> BuyableCounterI<Chain> {
    /// Instantiate the contract in any CosmWasm environment
    pub fn deploy(
        chain: Chain,
        instantiate_msg: InstantiateMsg,
        admin: Addr,
    ) -> cw_orch::anyhow::Result<BuyableCounterI<Chain>> {
        // Construct the  interface
        let contract = BuyableCounterI::new(chain.clone());

        // Upload the contract
        contract.upload()?;

        // Instantiate the contract
        contract.instantiate(&instantiate_msg, Some(&admin), &[])?;

        // Return the interface
        Ok(contract)
    }
}
