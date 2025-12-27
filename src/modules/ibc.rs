//use std::collections::HashMap;

use crate::{app::AppResponse, env::RobotoEnv, module::ModuleLogic};

use cosmwasm_std::{
    testing::{MockApi, MockQuerier, MockStorage},
    Empty, IbcMsg, IbcQuery, MessageInfo,
};

use anyhow::bail;

#[derive(Default)]
pub struct Ibc {
    pub channels: Empty,
}

impl ModuleLogic for Ibc {
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
        bail!("ibc query unsupported")
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
