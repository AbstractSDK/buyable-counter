use buyable_counter::{
    interface::BuyableCounterI,
    msg::{ExecuteMsgFns, InstantiateMsg, QueryMsgFns},
};
use cw_orch::{anyhow, prelude::*};

pub fn main() -> anyhow::Result<()> {
    // #5
    dotenv::dotenv().ok(); // Used to load the `.env` file if any
    env_logger::init(); // Used to log contract and chain interactions

    let network = networks::PION_1;
    let chain = DaemonBuilder::new(network.clone()).build()?;

    let counter = BuyableCounterI::deploy(
        chain.clone(),
        InstantiateMsg {
            count: 0,
            price: Coin::new(1u128, network.gas_denom),
        },
        chain.sender_addr(),
    )?;

    counter.increment()?;

    let count = counter.get_count()?;
    assert_eq!(count.count, 1);

    Ok(())
}
