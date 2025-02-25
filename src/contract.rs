#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128};
use cw2::set_contract_version;
use cw_paginate_storage::paginate_map;


use crate::error::ContractError;
use crate::msg::{ ExecuteMsg, GetRefferalResponse, InstantiateMsg, IsWhitelistedResponse, QueryMsg};
use crate::state::{WhitelistData, ADMIN, REFFERALS, WHITELIST};



const ONE_INJ: Uint128 = Uint128::new(1_000_000_000_000_000_000);

const CONTRACT_NAME: &str = "crates.io:early-access";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let admin_addr = deps.api.addr_validate(&msg.admin)?;
    ADMIN.save(deps.storage, &admin_addr)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", msg.admin))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::JoinWhitelist { ref_code } => execute_join_whitelist(deps, info,ref_code),
        ExecuteMsg::Withdraw {} => execute_withdraw(deps, env, info),
    }
}

pub fn execute_join_whitelist(deps: DepsMut, info: MessageInfo,ref_code:String) -> Result<Response, ContractError>{
    
    if WHITELIST
        .may_load(deps.storage, info.sender.to_string())?
        .unwrap_or(false)
    {
        return Err(ContractError::AlreadyWhitelisted {});
    }


    let mut found_inj = false;
    for coin in info.funds.iter() {
        if coin.denom == "inj" {
            
            if coin.amount != ONE_INJ {
                return Err(ContractError::PaymentError {});
            }
            found_inj = true;
        }
    }
    if !found_inj {
        return Err(ContractError::PaymentFailed {});
    }

    
    let referrer_opt = REFFERALS.may_load(deps.storage, ref_code.to_string())?;

    match referrer_opt {
        Some(mut referrer) => {
            referrer.count += 1; 
            REFFERALS.save(deps.storage, ref_code.to_string(), &referrer)?;
        }
        None => {
            if ref_code.to_string() != String::from(""){
                return Err(ContractError::InvalidRefCode {});
            }
        }
    }
    
    

    let sender = info.sender.to_string();

    let ref_code_self = format!("jecta{}", &sender[3..]);
    let new_entry = WhitelistData {
        ref_code: ref_code_self.to_string(),
        ref_address: sender.clone(),
        count: 0,
    };

    REFFERALS.save(deps.storage, ref_code_self.to_string(), &new_entry)?;

    WHITELIST.save(deps.storage, info.sender.to_string(), &true)?;

    Ok(Response::new()
        .add_attribute("action", "join_whitelist")
        .add_attribute("ref_code", ref_code_self)
        .add_attribute("address", info.sender.to_string()))
        
}



pub fn execute_withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    
    let admin_addr = ADMIN.load(deps.storage)?;
    if info.sender != admin_addr {
        return Err(ContractError::Unauthorized {});
    }

    let contract_balance = deps.querier.query_all_balances(env.contract.address)?;
    
    
    let inj_balance = contract_balance
        .into_iter()
        .find(|coin| coin.denom == "inj");

    
    let inj_balance = match inj_balance {
        Some(balance) if balance.amount.u128() > 0 => balance,
        _ => return Err(ContractError::Unauthorized {}),
    };

    let bank_msg = BankMsg::Send {
        to_address: admin_addr.to_string(),
        amount: vec![inj_balance], 
    };

    Ok(Response::new()
        .add_message(bank_msg)
        .add_attribute("action", "withdraw")
        .add_attribute("admin", admin_addr.to_string()))
}






#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::IsWhitelisted{address}=>{to_json_binary(&query_is_whitelisted(deps,address)?)}
        QueryMsg::ListWhitelisted{start_after,limit}=>{to_json_binary(&query_list_whitelisted(deps,start_after,limit)?)}
        QueryMsg::ListReferrals{start_after,limit}=>{to_json_binary(&query_list_ref_codes(deps,start_after,limit)?)},
        QueryMsg::GetRefferal { ref_code } => {to_json_binary(&query_get_refferal(deps,ref_code)?)},
    }
}

pub fn query_get_refferal(deps: Deps, ref_code: String) -> StdResult<GetRefferalResponse> {
    let referrer_opt = REFFERALS.load(deps.storage, ref_code.to_string())?;
    
    Ok(GetRefferalResponse{ ref_code, ref_address: referrer_opt.ref_address, count: referrer_opt.count })
}

pub fn query_list_whitelisted(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Binary> {
    to_json_binary(&paginate_map(
        deps,
        &WHITELIST,
        start_after,
        limit,
        cosmwasm_std::Order::Descending,
    )?)
}

pub fn query_list_ref_codes(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Binary> {
    to_json_binary(&paginate_map(
        deps,
        &REFFERALS,
        start_after,
        limit,
        cosmwasm_std::Order::Descending,
    )?)
}

pub fn query_is_whitelisted(deps: Deps, address: String) -> StdResult<IsWhitelistedResponse> {
    let addr = deps.api.addr_validate(&address)?;
    let is_whitelisted = WHITELIST
        .may_load(deps.storage, addr.to_string())?
        .unwrap_or(false);
    Ok(IsWhitelistedResponse { is_whitelisted })
}

