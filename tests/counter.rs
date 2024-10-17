use buyable_counter::{
    interface::BuyableCounterI,
    msg::{ExecuteMsgFns, GetCountResponse, InstantiateMsg, QueryMsg, QueryMsgFns},
    ContractError,
};
// Use prelude to get all the necessary imports
use cw_orch::anyhow;
use cw_orch::prelude::*;

use cosmwasm_std::{coins, Addr};

// consts for testing
const USER: &str = "user";
const ADMIN: &str = "admin";

#[test]
fn count() -> anyhow::Result<()> {
    // Create a user
    let user = Addr::unchecked(USER);
    // Create the mock. This will be our chain object throughout
    let mock = Mock::new(ADMIN);

    let contract = BuyableCounterI::deploy(
        mock.clone(),
        InstantiateMsg {
            count: 1,
            price: Coin::new(1000u128, "earth"),
        },
        mock.sender.clone(),
    )?;

    // Increment the count of the contract
    contract
        // Set the caller to user
        .call_as(&user)
        // Call the increment function (auto-generated function provided by ExecuteMsgFns)
        .increment()?;

    // Get the count.
    use buyable_counter::msg::QueryMsgFns;
    let count1 = contract.get_count()?;

    // or query it manually
    let count2: GetCountResponse = contract.query(&QueryMsg::GetCount {})?;
    assert_eq!(count1.count, count2.count);

    // Or get it manually from the chain
    let count3: GetCountResponse = mock.query(&QueryMsg::GetCount {}, &contract.address()?)?;
    assert_eq!(count1.count, count3.count);

    // Check the count
    assert_eq!(count1.count, 2);
    // Reset
    use buyable_counter::msg::ExecuteMsgFns;
    contract.reset(0)?;

    let count = contract.get_count()?;
    assert_eq!(count.count, 0);

    // Check negative case
    let exec_res: Result<cw_orch::mock::cw_multi_test::AppResponse, CwOrchError> =
        contract.call_as(&user).reset(0);

    let expected_err = ContractError::Unauthorized {};
    assert_eq!(
        exec_res.unwrap_err().downcast::<ContractError>()?,
        expected_err
    );

    Ok(())
}

#[test]
fn buy_admin() -> anyhow::Result<()> {
    // #4
    // Create the mock. This will be our chain object throughout
    let mock = Mock::new(ADMIN);
    let user = mock.addr_make(USER);

    let mut contract = BuyableCounterI::deploy(
        mock.clone(),
        InstantiateMsg {
            count: 0,
            price: Coin::new(1000u128, "earth"),
        },
        mock.sender.clone(),
    )?;

    contract.set_sender(&user);

    mock.set_balance(&user, coins(1001, "earth"))?;

    let can_buy = contract.can_buy(user.to_string())?;
    assert_eq!(can_buy.can_buy, true);

    // not enough funds
    let res = contract.buy_admin(coins(1000, "earth").as_slice());

    let expected_err = ContractError::InvalidPayment {
        expected: Coin::new(1000u128, "earth"),
        received: Coin::new(1000u128, "earth"),
    };
    assert_eq!(res.unwrap_err().downcast::<ContractError>()?, expected_err);

    contract.buy_admin(coins(1001, "earth").as_slice())?;

    contract.reset(0)?;

    let count = contract.get_count()?;
    assert_eq!(count.count, 0);

    let can_buy = contract.can_buy(user.to_string())?;
    assert_eq!(can_buy.can_buy, false);

    Ok(())
}
