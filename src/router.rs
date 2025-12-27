use cosmwasm_std::{
    testing::{MockApi, MockQuerier, MockStorage},
    Binary, ContractResult, CosmosMsg, Empty, MessageInfo, QuerierResult, QueryRequest,
    SystemResult,
};

use crate::{app::AppResponse, env::RobotoEnv, module::ModuleLogic, modules};

pub struct Router {
    pub wasm: modules::wasm::Wasm,
    pub bank: modules::bank::Bank,
}

impl Router {
    pub fn new() -> Self {
        Self {
            wasm: modules::wasm::Wasm::new(),
            bank: modules::bank::Bank::new(),
            // custom: Custom { some: None },
        }
    }

    pub fn handle_execute(
        &mut self,
        api: &MockApi,
        storage: &mut MockStorage,
        querier: &MockQuerier,
        env: &mut RobotoEnv,
        info: &MessageInfo,
        msg: CosmosMsg,
    ) -> anyhow::Result<AppResponse> {
        match msg {
            CosmosMsg::Wasm(msg) => self.wasm.execute(api, storage, querier, env, info, msg),
            CosmosMsg::Bank(msg) => self.bank.execute(api, storage, querier, env, info, msg),
            #[cfg(feature = "staking")]
            CosmosMsg::Staking(_) => todo!(),
            #[cfg(feature = "distribution")]
            CosmosMsg::Distribution(_) => todo!(),
            #[cfg(feature = "gov")]
            CosmosMsg::Gov(_) => todo!(),
            #[cfg(feature = "ibc")]
            CosmosMsg::Ibc(_) => todo!(),
            #[cfg(feature = "stargate")]
            CosmosMsg::Stargate { type_url, value } => todo!(),
            _ => todo!(),
        }
    }
}

// impl<CustomQuery: DeserializeOwned> Querier for Router<CustomQuery> {
//     fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
//         let request: QueryRequest<CustomQuery> = match from_slice(bin_request) {
//             Ok(v) => v,
//             Err(e) => {
//                 return SystemResult::Err(SystemError::InvalidRequest {
//                     error: format!("Parsing query request: {}", e),
//                     request: bin_request.into(),
//                 })
//             }
//         };

//         self.handle_query(&request)
//     }
// }

impl Router {
    pub fn handle_query(
        &self,
        api: &MockApi,
        storage: &mut MockStorage,
        querier: &MockQuerier,
        env: &mut RobotoEnv,
        request: &QueryRequest<Empty>,
    ) -> QuerierResult {
        let query_res = match &request {
            QueryRequest::Wasm(msg) => self.wasm.query(api, storage, querier, env, msg.clone()),
            QueryRequest::Bank(msg) => self.bank.query(api, storage, querier, env, msg.clone()),
            #[cfg(feature = "custom")]
            QueryRequest::Custom(_) => todo!(),
            // QueryRequest::Bank(bank_query) => self.bank.query(bank_query),
            // QueryRequest::Custom(custom_query) => (*self.custom_handler)(custom_query),
            // #[cfg(feature = "staking")]
            // QueryRequest::Staking(staking_query) => self.staking.query(staking_query),
            // #[cfg(feature = "ibc")]
            // QueryRequest::Ibc(msg) => self.ibc.query(msg),
            // #[cfg(feature = "stargate")]
            // QueryRequest::Stargate { path, data } => todo!(),
            _ => panic!("this module is not implemented"),
        };

        let contract_result: ContractResult<Binary> = query_res.into();

        SystemResult::Ok(contract_result)
    }
}

#[cfg(test)]
mod test_router {}
