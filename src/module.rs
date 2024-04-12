use cosmwasm_std::{
    testing::{MockApi, MockQuerier, MockStorage},
    BlockInfo, // Addr, Api, Binary, BlockInfo, MessageInfo, Querier, Storage,
    MessageInfo,
};

use crate::{app::AppResponse, env::RobotoEnv};

// type FnExec<Exec> =
//     dyn Fn(&dyn Api, &mut dyn Storage, &BlockInfo, Addr, Exec) -> anyhow::Result<AppResponse>;

// type FnQuery<Query> = dyn Fn(
//     &dyn Api,
//     &mut dyn Storage,
//     &dyn Querier,
//     &BlockInfo,
//     Addr,
//     Query,
// ) -> anyhow::Result<Binary>;

// type FnSudo<Sudo> = dyn Fn(
//     &dyn Api,
//     &mut dyn Storage,
//     &BlockInfo,
//     Sudo,
// ) -> anyhow::Result<AppResponse>;

// pub struct Module<Exec, Query, Sudo> {
// pub struct Module<Exec, Query> {
//     pub execute_fn: Box<FnExec<Exec>>,
//     pub query_fn: Box<FnQuery<Query>>,
//     // pub sudo_fn: Box<FnSudo<Sudo>>
// }

pub trait ModuleLogic {
    type ExecM;
    type QueryM;
    type SudoM;

    fn execute(
        &mut self,
        api: &MockApi,
        storage: &mut MockStorage,
        querier: &MockQuerier,
        env: &mut RobotoEnv,
        info: &MessageInfo,
        msg: Self::ExecM,
    ) -> anyhow::Result<AppResponse>;

    fn query(
        &self,
        api: &MockApi,
        storage: &mut MockStorage,
        querier: &MockQuerier,
        env: &mut RobotoEnv,
        msg: Self::QueryM,
    ) -> anyhow::Result<cosmwasm_std::Binary>;

    fn sudo(
        &mut self,
        api: &MockApi,
        storage: &mut MockStorage,
        querier: &MockQuerier,
        block: &BlockInfo,
        msg: Self::SudoM,
    ) -> anyhow::Result<AppResponse>;
}
