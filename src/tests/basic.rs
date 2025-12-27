#[cfg(test)]
mod basic {
    use std::fmt::Debug;

    use crate::{app::*, contract::*};

    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{
        from_json, testing::mock_info, to_json_binary, wasm_execute, Addr, BalanceResponse, BankMsg, BankQuery, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdError, StdResult, Uint128, WasmMsg, WasmQuery
    };
    use cw20::MinterResponse;
    use thiserror::Error;

    #[cw_serde]
    pub struct InstantiateMsg2 {
        name: String,
    }

    #[cw_serde]
    pub enum ExecuteMsg {
        Mint {
            contract: String,
            recipient: String,
            amount: Uint128,
        },
    }

    #[derive(Error, Debug, PartialEq)]
    pub enum ContractError {
        #[error("{0}")]
        Std(#[from] StdError),
    }

    fn dummy_exec(
        _deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        match msg {
            ExecuteMsg::Mint {
                contract,
                recipient,
                amount,
            } => {
                let msg = &cw20_base::msg::ExecuteMsg::Mint { recipient, amount };
                let msg = wasm_execute(contract, msg, vec![])?;
                Ok(Response::default().add_message(msg))
            }
        }
    }

    fn dummy_query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        Ok(to_json_binary(&Empty {})?)
    }

    fn dummy_init2(
        mut deps: DepsMut,
        _env: Env,
        _info: MessageInfo,
        msg: InstantiateMsg2,
    ) -> Result<Response, ContractError> {
        Ok(Response::default())
    }

    fn dummy_exec2(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        Ok(Response::default())
    }

    fn dummy_query2(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        Ok(to_json_binary(&Empty {})?)
    }

    #[test]
    fn basic() {
        let mut app = App::new();

        let ADMIN = app.api.addr_make("admin");
        let RECIPIENT = app.api.addr_make("recipient");

        let DENOM = "utaco";

        let contract: Contract<
            cw20_base::msg::InstantiateMsg,
            cw20::Cw20ExecuteMsg,
            cw20_base::ContractError,
            cw20_base::msg::QueryMsg,
            Empty,
            Empty,
        > = Contract::with_query(
            Box::new(cw20_base::contract::instantiate),
            Box::new(cw20_base::contract::execute),
            Box::new(cw20_base::contract::query),
        );

        let mut contract2: Contract<Empty, ExecuteMsg, ContractError, Empty, Empty, Empty> =
            Contract::default();

        contract2.exec_fn = Box::new(dummy_exec);

        let code_id = app.router.wasm.add_contract(Box::new(contract));
        let code_id2 = app.router.wasm.add_contract(Box::new(contract2));

        let init_msg = cw20_base::msg::InstantiateMsg {
            name: "taco".into(),
            symbol: "taco".into(),
            decimals: 6,
            initial_balances: vec![],
            mint: Some(MinterResponse {
                minter: Addr::unchecked(ADMIN.clone()).to_string(),
                cap: None,
            }),
            marketing: None,
        };

        let res = app.execute(cosmwasm_std::CosmosMsg::Wasm(WasmMsg::Instantiate {
            admin: None,
            code_id,
            msg: to_json_binary(&init_msg).unwrap(),
            funds: vec![],
            label: "todo".to_string(),
        }));

        println!("{:#?}", res);

        let addr_contract = res[0].as_ref().unwrap().events[0].attributes[0]
            .value
            .clone();

        let mint_msg = cw20_base::msg::ExecuteMsg::Mint {
            recipient: RECIPIENT.to_string(),
            amount: Uint128::one(),
        };

        app.info = mock_info(&ADMIN.to_string(), &vec![]);

        let res = app.execute(cosmwasm_std::CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: addr_contract.clone(),
            msg: to_json_binary(&mint_msg).unwrap(),
            funds: vec![],
        }));

        println!("{:#?}", res);

        let res = app.execute(cosmwasm_std::CosmosMsg::Wasm(WasmMsg::Instantiate {
            admin: None,
            code_id: code_id2,
            msg: to_json_binary(&InstantiateMsg2 {
                name: String::from("taco"),
            })
            .unwrap(),
            funds: vec![],
            label: "todo!()".to_string(),
        }));

        println!("{:#?}", res);

        let addr_contract2 = res[0].as_ref().unwrap().events[0].attributes[0]
            .value
            .clone();

        let mint_msg = ExecuteMsg::Mint {
            recipient: RECIPIENT.to_string(),
            amount: Uint128::one(),
            contract: addr_contract.to_string(),
        };

        let res = app.execute(cosmwasm_std::CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: addr_contract2.clone(),
            msg: to_json_binary(&mint_msg).unwrap(),
            funds: vec![],
        }));

        println!("{:#?}", res);

        // let res = app.query(cosmwasm_std::QueryRequest::Wasm(WasmQuery::ContractInfo {
        //     contract_addr: addr_contract2,
        // }));

        // println!(
        //     "{:#?}",
        //     from_binary::<cosmwasm_std::ContractInfoResponse>(&res.unwrap()).unwrap()
        // );

        let res = app.query(cosmwasm_std::QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: addr_contract,
            msg: to_json_binary(&cw20_base::msg::QueryMsg::Balance {
                address: ADMIN.to_string(),
            })
            .unwrap(),
        }));

        println!(
            "{:#?}",
            from_json::<cw20::BalanceResponse>(&res.unwrap()).unwrap()
        );

        // let res = app
        //     .exec::<cw20_base::msg::ExecuteMsg>(addr_contract.clone(), to_vec(&mint_msg).unwrap());

        // println!("{:#?}", res);

        // let query_msg = cw20_base::msg::QueryMsg::Balance {
        //     address: "sender".to_string(),
        // };
        // let res = app
        //     .query::<cw20_base::msg::QueryMsg>(addr_contract.clone(), to_vec(&query_msg).unwrap());

        // println!(
        //     "{:#?}",
        //     from_json::<cw20::BalanceResponse>(&res.unwrap()).unwrap()
        // );

        // let res = app.exec::<ExecuteMsg>(addr_contract2, to_vec(&mint_msg).unwrap());

        // println!("{:#?}", res);

        let res = app.query(cosmwasm_std::QueryRequest::Bank(BankQuery::Balance {
            address: ADMIN.to_string(),
            denom: DENOM.into(),
        }));
        let res = from_json::<BalanceResponse>(&res.unwrap());

        println!("{:#?}", res);

        assert!(false)
    }
}
