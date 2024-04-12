//use std::collections::HashMap;

use crate::{app::AppResponse, env::RobotoEnv, module::ModuleLogic};

use cosmwasm_std::{
    testing::{MockApi, MockQuerier, MockStorage},
    Empty, IbcMsg, IbcQuery, MessageInfo,
};

#[derive(Default)]
pub struct ibc {
    pub channels: Empty,
}

impl ModuleLogic for ibc {
    type ExecM = IbcMsg;
    type QueryM = IbcQuery;
    type SudoM = Empty;

    fn execute(
        &mut self,
        _api: &MockApi,
        _storage: &mut MockStorage,
        _querier: &MockQuerier,
        _env: &mut RobotoEnv,
        _info: &MessageInfo,
        _msg: Self::ExecM,
    ) -> anyhow::Result<AppResponse> {
        Ok(AppResponse::default())
    }

    fn query(
        &self,
        _api: &MockApi,
        _storage: &mut MockStorage,
        _querier: &MockQuerier,
        _env: &mut RobotoEnv,
        _msg: Self::QueryM,
    ) -> anyhow::Result<cosmwasm_std::Binary> {
        todo!()
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
