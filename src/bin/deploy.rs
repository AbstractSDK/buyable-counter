use buyable_counter::{
    interface::BuyableCounterI,
    msg::{ExecuteMsgFns, InstantiateMsg, QueryMsgFns},
};
use cosmwasm_std::coin;
use cw_orch::{anyhow, prelude::*};

pub fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok(); // Used to load the `.env` file if any
    env_logger::init(); // Used to log contract and chain interactions

    let network = networks::PION_1;
    let chain = DaemonBuilder::new(network.clone()).build()?;

    let counter = BuyableCounterI::new(chain);

    counter.upload()?;
    counter.instantiate(&InstantiateMsg { count: 0, price: coin(1, network.gas_denom) }, None, &[])?;

    counter.increment()?;

    let count = counter.get_count()?;
    assert_eq!(count.count, 1);

    Ok(())
}
