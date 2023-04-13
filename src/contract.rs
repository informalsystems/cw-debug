#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetMessageResponse, InstantiateMsg, QueryMsg};
use crate::state::{State, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-debug";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        message: msg.message.clone(),
        setter: info.sender.clone(),
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("message", msg.message))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetMessage { message, sudo } => execute::set_message(deps, info, message, sudo),
    }
}

pub mod execute {
    use super::*;

    pub fn set_message(
        deps: DepsMut,
        info: MessageInfo,
        message: String,
        sudo: bool,
    ) -> Result<Response, ContractError> {
        STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
            if sudo && info.sender != state.owner {
                return Err(ContractError::Unauthorized {});
            }
            state.message = message;
            state.setter = info.sender;
            Ok(state)
        })?;
        Ok(Response::new().add_attribute("action", "set_message"))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetMessage {} => to_binary(&query::get_message(deps)?),
    }
}

pub mod query {
    use super::*;

    pub fn get_message(deps: Deps) -> StdResult<GetMessageResponse> {
        let state = STATE.load(deps.storage)?;
        Ok(GetMessageResponse {
            message: state.message,
            setter: state.setter.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            message: "Hello World".to_owned(),
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetMessage {}).unwrap();
        let value: GetMessageResponse = from_binary(&res).unwrap();
        assert_eq!("Hello World".to_owned(), value.message);
    }

    #[test]
    fn is_sudo_creator_update() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            message: "Hello World".to_owned(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // update the message
        let info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::SetMessage {
            message: "Hello Mars".to_owned(),
            sudo: true,
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should update the message
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetMessage {}).unwrap();
        let value: GetMessageResponse = from_binary(&res).unwrap();
        assert_eq!("Hello Mars".to_owned(), value.message);
    }

    #[test]
    #[should_panic(expected = "Unauthorized")]
    fn is_sudo_unauthorized_update() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            message: "Hello World".to_owned(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // update the message
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::SetMessage {
            message: "Hello Mars".to_owned(),
            sudo: true,
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    }

    #[test]
    fn not_sudo_unauthorized_update() {
        let mut deps = mock_dependencies();

        let msg = InstantiateMsg {
            message: "Hello World".to_owned(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // update the message
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::SetMessage {
            message: "Hello Mars".to_owned(),
            sudo: false,
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should update the message
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetMessage {}).unwrap();
        let value: GetMessageResponse = from_binary(&res).unwrap();
        assert_eq!("Hello Mars".to_owned(), value.message);
    }
}
