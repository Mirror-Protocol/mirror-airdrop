use crate::contract::{handle, init, query};
use crate::msg::{ConfigResponse, HandleMsg, InitMsg, QueryMsg};
use cosmwasm_std::testing::{mock_dependencies, mock_env};
use cosmwasm_std::{from_binary, log, to_binary, CosmosMsg, HumanAddr, StdError, Uint128, WasmMsg};
use cw20::Cw20HandleMsg;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(20, &[]);

    let msg = InitMsg {
        owner: HumanAddr("owner0000".to_string()),
        mirror_token: HumanAddr("mirror0000".to_string()),
        merkle_root: "85e33930e7a8f015316cb4a53a4c45d26a69f299fc4c83f17357e1fd62e8fd95".to_string(),
    };

    let env = mock_env("addr0000", &[]);

    // we can just call .unwrap() to assert this was a success
    let _res = init(&mut deps, env, msg).unwrap();

    // it worked, let's query the state
    let res = query(&deps, QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!("owner0000", config.owner.as_str());
    assert_eq!("mirror0000", config.mirror_token.as_str());
    assert_eq!(
        "85e33930e7a8f015316cb4a53a4c45d26a69f299fc4c83f17357e1fd62e8fd95".to_string(),
        config.merkle_root
    );
}

#[test]
fn update_config() {
    let mut deps = mock_dependencies(20, &[]);

    let msg = InitMsg {
        owner: HumanAddr::from("owner0000"),
        mirror_token: HumanAddr::from("mirror0000"),
        merkle_root: "85e33930e7a8f015316cb4a53a4c45d26a69f299fc4c83f17357e1fd62e8fd95".to_string(),
    };

    let env = mock_env("addr0000", &[]);
    let _res = init(&mut deps, env.clone(), msg).unwrap();

    // update owner
    let env = mock_env("owner0000", &[]);
    let msg = HandleMsg::UpdateConfig {
        owner: Some(HumanAddr("owner0001".to_string())),
        merkle_root: Some(
            "85e33930e7a8f015316cb4a53a4c45d26a69f299fc4c83f17357e1fd62e8fd94".to_string(),
        ),
    };

    let res = handle(&mut deps, env, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(&deps, QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!("owner0001", config.owner.as_str());
    assert_eq!(
        "85e33930e7a8f015316cb4a53a4c45d26a69f299fc4c83f17357e1fd62e8fd94".to_string(),
        config.merkle_root
    );

    // Unauthorzied err
    let env = mock_env("owner0000", &[]);
    let msg = HandleMsg::UpdateConfig {
        owner: None,
        merkle_root: None,
    };

    let res = handle(&mut deps, env, msg);
    match res {
        Err(StdError::Unauthorized { .. }) => {}
        _ => panic!("Must return unauthorized error"),
    }
}

#[test]
fn claim() {
    let mut deps = mock_dependencies(20, &[]);

    let msg = InitMsg {
        owner: HumanAddr::from("owner0000"),
        mirror_token: HumanAddr::from("mirror0000"),
        merkle_root: "85e33930e7a8f015316cb4a53a4c45d26a69f299fc4c83f17357e1fd62e8fd95".to_string(),
    };

    let env = mock_env("addr0000", &[]);
    let _res = init(&mut deps, env, msg).unwrap();

    let msg = HandleMsg::Claim {
        amount: Uint128::from(1000001u128),
        proof: vec![
            "b8ee25ffbee5ee215c4ad992fe582f20175868bc310ad9b2b7bdf440a224b2df".to_string(),
            "98d73e0a035f23c490fef5e307f6e74652b9d3688c2aa5bff70eaa65956a24e1".to_string(),
            "f328b89c766a62b8f1c768fefa1139c9562c6e05bab57a2af87f35e83f9e9dcf".to_string(),
            "fe19ca2434f87cadb0431311ac9a484792525eb66a952e257f68bf02b4561950".to_string(),
        ],
    };

    let env = mock_env(
        "terra1qfqa2eu9wp272ha93lj4yhcenrc6ymng079nu8".to_string(),
        &[],
    );
    let res = handle(&mut deps, env, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: HumanAddr::from("mirror0000"),
            send: vec![],
            msg: to_binary(&Cw20HandleMsg::Transfer {
                recipient: HumanAddr::from("terra1qfqa2eu9wp272ha93lj4yhcenrc6ymng079nu8"),
                amount: Uint128::from(1000001u128),
            })
            .unwrap(),
        })]
    );

    assert_eq!(
        res.log,
        vec![
            log("action", "claim"),
            log("address", "terra1qfqa2eu9wp272ha93lj4yhcenrc6ymng079nu8"),
            log("amount", "1000001")
        ]
    );
}
