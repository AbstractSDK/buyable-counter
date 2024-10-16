#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetCountResponse, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:buyable-counter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        count: msg.count,
        owner: info.sender.clone(),
        last_price: msg.price, // #1
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("count", msg.count.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Increment {} => execute::increment(deps),
        ExecuteMsg::Reset { count } => execute::reset(deps, info, count),
        ExecuteMsg::BuyAdmin {} => execute::buy_admin(deps, info), // #1
    }
}

pub mod execute {
    use cosmwasm_std::Coin;

    use super::*;

    pub fn increment(deps: DepsMut) -> Result<Response, ContractError> {
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            state.count += 1;
            Ok(state)
        })?;

        Ok(Response::new().add_attribute("action", "increment"))
    }

    pub fn reset(deps: DepsMut, info: MessageInfo, count: i32) -> Result<Response, ContractError> {
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            if info.sender != state.owner {
                return Err(ContractError::Unauthorized {});
            }
            state.count = count;
            Ok(state)
        })?;
        Ok(Response::new().add_attribute("action", "reset"))
    }

    // #1
    pub fn buy_admin(deps: DepsMut, mut info: MessageInfo) -> Result<Response, ContractError> {
        // #2
        let state = STATE.load(deps.storage)?;
        let price = state.last_price;
        let paid = info.funds.pop().ok_or(ContractError::InvalidPayment {
            expected: price.clone(),
            received: Coin::new(0u128, ""),
        })?;

        if paid.denom != price.denom || paid.amount < price.amount {
            return Err(ContractError::InvalidPayment {
                expected: price,
                received: paid.clone(),
            });
        }

        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            state.owner = info.sender.clone();
            state.last_price = paid.clone();
            Ok(state)
        })?;

        Ok(Response::new().add_attribute("action", "buy_admin"))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_json_binary(&query::count(deps)?),
    }
}

pub mod query {
    use super::*;

    pub fn count(deps: Deps) -> StdResult<GetCountResponse> {
        let state = STATE.load(deps.storage)?;
        Ok(GetCountResponse { count: state.count })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
    use cosmwasm_std::{coins, from_json, Coin};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            count: 17,
            price: Coin::new(1000u128, "earth"),
        };
        let info = message_info(&deps.api.addr_make("creator"), &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_json(&res).unwrap();
        assert_eq!(17, value.count);
    }

    #[test]
    fn increment() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            count: 17,
            price: Coin::new(1000u128, "earth"),
        };
        let info = message_info(&deps.api.addr_make("creator"), &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = message_info(&deps.api.addr_make("anyone"), &coins(2, "token"));
        let msg = ExecuteMsg::Increment {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_json(&res).unwrap();
        assert_eq!(18, value.count);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            count: 17,
            price: Coin::new(1000u128, "earth"),
        };
        let info = message_info(&deps.api.addr_make("creator"), &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let unauth_info = message_info(&deps.api.addr_make("anyone"), &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_info = message_info(&deps.api.addr_make("creator"), &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        // should now be 5
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_json(&res).unwrap();
        assert_eq!(5, value.count);
    }

    #[test]
    fn buy_admin() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            count: 17,
            price: Coin::new(1000u128, "earth"),
        };
        let info = message_info(&deps.api.addr_make("creator"), &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let new_admin = deps.api.addr_make("new_admin");
        let info = message_info(&new_admin, &coins(0, "earth"));
        let msg = ExecuteMsg::BuyAdmin {};
        let res = execute(deps.as_mut(), mock_env(), info, msg.clone());
        match res {
            Err(ContractError::InvalidPayment { expected, received }) => {
                assert_eq!(expected, Coin::new(1000u128, "earth"));
                assert_eq!(received, Coin::new(0u128, "earth"));
            }
            _ => panic!("Must return invalid payment error"),
        }

        let info = message_info(&new_admin, &coins(1000, "earth"));
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        // reset count with new admin
        let msg = ExecuteMsg::Reset { count: 5 };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: GetCountResponse = from_json(&res).unwrap();
        assert_eq!(5, value.count);
    }
}
