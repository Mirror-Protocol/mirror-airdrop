#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
    Uint128, WasmMsg,
};

use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, IsClaimedResponse, LatestStageResponse,
    MerkleRootResponse, QueryMsg,
};
use crate::state::{Config, CLAIM_INDEX, CONFIG, LATEST_STAGE, MERKLE_ROOT};

use cw20::Cw20ExecuteMsg;
use sha3::Digest;
use std::convert::TryInto;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    CONFIG.save(
        deps.storage,
        &Config {
            owner: deps.api.addr_canonicalize(&msg.owner)?,
            mirror_token: deps.api.addr_canonicalize(&msg.mirror_token)?,
        },
    )?;

    let stage: u8 = 0;
    LATEST_STAGE.save(deps.storage, &stage)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig { owner } => update_config(deps, env, info, owner),
        ExecuteMsg::UpdateMerkleRoot { stage, merkle_root } => {
            update_merkle_root(deps, env, info, stage, merkle_root)
        }
        ExecuteMsg::RegisterMerkleRoot { merkle_root } => {
            register_merkle_root(deps, env, info, merkle_root)
        }
        ExecuteMsg::Claim {
            stage,
            amount,
            proof,
        } => claim(deps, env, info, stage, amount, proof),
    }
}

pub fn update_merkle_root(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    stage: u8,
    merkle_root: String,
) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;
    if deps.api.addr_canonicalize(info.sender.as_str())? != config.owner {
        return Err(StdError::generic_err("unauthorized"));
    }

    MERKLE_ROOT.save(deps.storage, &[stage], &merkle_root)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "update_merkle_root"),
        ("stage", &stage.to_string()),
        ("merkle_root", &merkle_root),
    ]))
}

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    owner: Option<String>,
) -> StdResult<Response> {
    let mut config: Config = CONFIG.load(deps.storage)?;
    if deps.api.addr_canonicalize(info.sender.as_str())? != config.owner {
        return Err(StdError::generic_err("unauthorized"));
    }

    if let Some(owner) = owner {
        config.owner = deps.api.addr_canonicalize(&owner)?;
    }

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new().add_attribute("action", "update_config"))
}

pub fn register_merkle_root(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    merkle_root: String,
) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;
    if deps.api.addr_canonicalize(info.sender.as_str())? != config.owner {
        return Err(StdError::generic_err("unauthorized"));
    }

    let latest_stage: u8 = LATEST_STAGE.load(deps.storage)?;
    let stage = latest_stage + 1;

    MERKLE_ROOT.save(deps.storage, &[stage], &merkle_root)?;
    LATEST_STAGE.save(deps.storage, &stage)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "register_merkle_root"),
        ("stage", &stage.to_string()),
        ("merkle_root", &merkle_root),
    ]))
}

pub fn claim(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    stage: u8,
    amount: Uint128,
    proof: Vec<String>,
) -> StdResult<Response> {
    let config: Config = CONFIG.load(deps.storage)?;
    let merkle_root: String = MERKLE_ROOT.load(deps.storage, &[stage])?;

    let user_raw = deps.api.addr_canonicalize(info.sender.as_str())?;

    // If user claimed target stage, return err
    if CLAIM_INDEX
        .may_load(deps.storage, (user_raw.as_slice(), &[stage]))?
        .unwrap_or(false)
    {
        return Err(StdError::generic_err("already claimed"));
    }

    let user_input: String = info.sender.to_string() + &amount.to_string();
    let mut hash: [u8; 32] = sha3::Keccak256::digest(user_input.as_bytes())
        .as_slice()
        .try_into()
        .expect("Wrong length");

    for p in proof {
        let mut proof_buf: [u8; 32] = [0; 32];
        hex::decode_to_slice(p, &mut proof_buf).unwrap();
        hash = if bytes_cmp(hash, proof_buf) == std::cmp::Ordering::Less {
            sha3::Keccak256::digest(&[hash, proof_buf].concat())
                .as_slice()
                .try_into()
                .expect("Wrong length")
        } else {
            sha3::Keccak256::digest(&[proof_buf, hash].concat())
                .as_slice()
                .try_into()
                .expect("Wrong length")
        };
    }

    let mut root_buf: [u8; 32] = [0; 32];
    hex::decode_to_slice(merkle_root, &mut root_buf).unwrap();
    if root_buf != hash {
        return Err(StdError::generic_err("Verification is failed"));
    }

    // Update claim index to the current stage
    CLAIM_INDEX.save(deps.storage, (user_raw.as_slice(), &[stage]), &true)?;

    Ok(Response::new()
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&config.mirror_token)?.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: info.sender.to_string(),
                amount,
            })?,
        }))
        .add_attributes(vec![
            ("action", "claim"),
            ("stage", &stage.to_string()),
            ("address", info.sender.as_str()),
            ("amount", &amount.to_string()),
        ]))
}

fn bytes_cmp(a: [u8; 32], b: [u8; 32]) -> std::cmp::Ordering {
    let mut i = 0;
    while i < 32 {
        match a[i].cmp(&b[i]) {
            std::cmp::Ordering::Greater => return std::cmp::Ordering::Greater,
            std::cmp::Ordering::Less => return std::cmp::Ordering::Less,
            _ => {}
        }

        i += 1;
    }

    std::cmp::Ordering::Equal
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps, env)?),
        QueryMsg::MerkleRoot { stage } => to_binary(&query_merkle_root(deps, env, stage)?),
        QueryMsg::LatestStage {} => to_binary(&query_latest_stage(deps, env)?),
        QueryMsg::IsClaimed { stage, address } => {
            to_binary(&query_is_claimed(deps, env, stage, address)?)
        }
    }
}

pub fn query_config(deps: Deps, _env: Env) -> StdResult<ConfigResponse> {
    let state = CONFIG.load(deps.storage)?;
    let resp = ConfigResponse {
        owner: deps.api.addr_humanize(&state.owner)?.to_string(),
        mirror_token: deps.api.addr_humanize(&state.mirror_token)?.to_string(),
    };

    Ok(resp)
}

pub fn query_merkle_root(deps: Deps, _env: Env, stage: u8) -> StdResult<MerkleRootResponse> {
    let merkle_root = MERKLE_ROOT.load(deps.storage, &[stage])?;
    let resp = MerkleRootResponse { stage, merkle_root };

    Ok(resp)
}

pub fn query_latest_stage(deps: Deps, _env: Env) -> StdResult<LatestStageResponse> {
    let latest_stage = LATEST_STAGE.load(deps.storage)?;
    let resp = LatestStageResponse { latest_stage };

    Ok(resp)
}

pub fn query_is_claimed(
    deps: Deps,
    _env: Env,
    stage: u8,
    address: String,
) -> StdResult<IsClaimedResponse> {
    let user_raw = deps.api.addr_canonicalize(&address)?;
    let resp = IsClaimedResponse {
        is_claimed: CLAIM_INDEX
            .may_load(deps.storage, (user_raw.as_slice(), &[stage]))?
            .unwrap_or(false),
    };

    Ok(resp)
}
