use std::collections::HashMap;

use crate::{
    app::AppResponse, contract::ContractLogicWrapper, env::RobotoEnv, module::ModuleLogic,
};
use anyhow::bail;
use cosmwasm_std::{
    testing::{MockApi, MockQuerier, MockStorage},
    to_binary, Addr, Binary, ContractInfoResponse, DepsMut, Empty, Event, MessageInfo,
    QuerierWrapper, WasmMsg, WasmQuery,
};
use schemars::JsonSchema;

#[derive(Clone, std::fmt::Debug, PartialEq, Eq, JsonSchema)]
pub struct WasmSudo {
    pub contract_addr: Addr,
    pub msg: Binary,
}

#[derive(Default)]
pub struct Wasm {
    pub code_ids: HashMap<u64, ContractLogicWrapper>,
    pub addresses: HashMap<String, u64>,
}

impl Wasm {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_contract(&mut self, contract: ContractLogicWrapper) -> u64 {
        let total = self.code_ids.len() as u64;
        self.code_ids.insert(total, contract);
        total
    }

    pub fn new_address(&mut self, code_id: u64) -> String {
        let addr = format!("contract{}", code_id);
        self.addresses.insert(addr.clone(), code_id);
        addr
    }

    pub fn find_by_code(&self, code_id: u64) -> Option<&ContractLogicWrapper> {
        self.code_ids.get(&code_id)
    }

    pub fn find_by_address(&self, address: impl Into<String>) -> Option<&ContractLogicWrapper> {
        self.code_ids
            .get(&self.addresses.get(&address.into()).unwrap())
    }
}

impl ModuleLogic for Wasm {
    type ExecM = WasmMsg;
    type QueryM = WasmQuery;
    type SudoM = WasmSudo;

    fn execute(
        &mut self,
        api: &MockApi,
        storage: &mut MockStorage,
        querier: &MockQuerier,
        env: &mut RobotoEnv,
        info: &MessageInfo,
        msg: WasmMsg,
    ) -> anyhow::Result<AppResponse> {
        match msg {
            WasmMsg::Instantiate {
                admin: _admin,
                code_id,
                msg,
                funds: _funds,
                label: _label,
            } => {
                let Some(contract) = self.find_by_code(code_id) else {
                    panic!("contract with code_id {} not found", code_id)
                };

                let deps: DepsMut<'_, _> = DepsMut {
                    api: api,
                    storage: &mut *storage,
                    querier: QuerierWrapper::<Empty>::new(querier),
                };

                let contract_res =
                    contract.instantiate(deps, env.inner(), info.clone(), msg.to_vec())?;

                let addr = self.new_address(code_id);

                let mut app_res = AppResponse::new();

                app_res.responses.push(contract_res);

                app_res.events.push(
                    Event::new("instantiate")
                        .add_attribute("contract_address", &addr)
                        .add_attribute("code_id", code_id.to_string()),
                );

                env.increase_tx();

                Ok(app_res)
            }
            WasmMsg::Execute {
                contract_addr,
                msg,
                // TODO: Implement this
                funds: _funds,
            } => {
                let Some(contract) = self.find_by_address(contract_addr.clone()) else {
                    panic!("contract with contract_addr {} not found", contract_addr)
                };

                env.set_contract(&contract_addr);

                let deps: DepsMut<'_, _> = DepsMut {
                    api: api,
                    storage: &mut *storage,
                    querier: QuerierWrapper::<Empty>::new(querier),
                };

                let contract_res =
                    contract.execute(deps, env.inner(), info.clone(), msg.to_vec())?;

                let mut app_res = AppResponse::new();

                app_res
                    .events
                    .push(Event::new("execute").add_attribute("contract_address", &contract_addr));

                app_res.responses.push(contract_res);

                env.increase_tx();

                Ok(app_res)
            }
            // WasmMsg::Migrate {
            //     contract_addr,
            //     new_code_id,
            //     msg,
            // } => todo!(),
            // WasmMsg::UpdateAdmin {
            //     contract_addr,
            //     admin,
            // } => todo!(),
            // WasmMsg::ClearAdmin { contract_addr } => todo!(),
            _ => bail!("wasm execute unsupported"),
        }
    }

    fn query(
        &self,
        api: &MockApi,
        storage: &mut MockStorage,
        querier: &MockQuerier,
        env: &mut RobotoEnv,
        msg: Self::QueryM,
    ) -> anyhow::Result<cosmwasm_std::Binary> {
        match msg {
            WasmQuery::Smart { contract_addr, msg } => {
                let Some(contract) = self.find_by_address(contract_addr.clone()) else {
                    panic!("contract with contract_addr {} not found", contract_addr)
                };

                env.set_contract(&contract_addr);

                let deps: DepsMut<'_, _> = DepsMut {
                    api: api,
                    storage: &mut *storage,
                    querier: QuerierWrapper::<Empty>::new(querier),
                };

                contract.query(deps.as_ref(), env.inner(), msg.to_vec())
            }
            WasmQuery::ContractInfo { contract_addr } => {
                let Some(code_id) = self.addresses.get(&contract_addr) else {
                    panic!("contract with contract_addr {} not found", contract_addr)
                };

                let mut res = ContractInfoResponse::default();
                res.code_id = *code_id;

                // TODO: fix this two
                res.creator = "".into();
                res.admin = None;

                to_binary(&res).map_err(Into::into)
            }
            // TODO: this is not priority for now
            // WasmQuery::Raw { contract_addr, key } => todo!(),
            _ => bail!("wasm query unsupported"),
        }
    }

    fn sudo(
        &mut self,
        _api: &MockApi,
        _storage: &mut MockStorage,
        _querier: &MockQuerier,
        _block: &cosmwasm_std::BlockInfo,
        _msg: Self::SudoM,
    ) -> anyhow::Result<AppResponse> {
        Ok(AppResponse::default())
    }
}
