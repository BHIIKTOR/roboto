use std::fmt::{Debug, Display};

use anyhow::{anyhow, bail, Result as AnyResult};

use cosmwasm_std::{
    from_slice, to_binary, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Reply, Response,
    StdResult,
};

use serde::de::DeserializeOwned;

type FnInit<Init, Error> = dyn Fn(DepsMut, Env, MessageInfo, Init) -> Result<Response, Error>;
type FnExec<Exec, Error> = dyn Fn(DepsMut, Env, MessageInfo, Exec) -> Result<Response, Error>;
type FnQuery<Query> = dyn Fn(Deps, Env, Query) -> StdResult<Binary>;
type FnSudo<Sudo, Error> = dyn Fn(DepsMut, Env, MessageInfo, Sudo) -> Result<Response, Error>;
type FnReply<Error> = dyn Fn(DepsMut, Env, Reply) -> Result<Response, Error>;
type FnMigrate<Migrate, Error> = dyn Fn(DepsMut, Env, Migrate) -> Result<Response, Error>;

pub trait ContractBase {}

pub struct Contract<Init, Exec, Error, Query, Sudo, Migrate> {
    pub init_fn: Box<FnInit<Init, Error>>,
    pub exec_fn: Box<FnExec<Exec, Error>>,
    pub query_fn: Option<Box<FnQuery<Query>>>,
    pub sudo_fn: Option<Box<FnSudo<Sudo, Error>>>,
    pub reply_fn: Option<Box<FnReply<Error>>>,
    pub migrate_fn: Option<Box<FnMigrate<Migrate, Error>>>,
}

impl<Init, Exec, Error, Query, Sudo, Migrate> ContractBase
    for Contract<Init, Exec, Error, Query, Sudo, Migrate>
{
}

pub trait ContractLogic: ContractBase {
    fn instantiate(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Vec<u8>,
    ) -> AnyResult<Response>;

    fn execute(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Vec<u8>,
    ) -> AnyResult<Response>;

    fn query(&self, deps: Deps, env: Env, msg: Vec<u8>) -> AnyResult<Binary>;

    fn migrate(&self, deps: DepsMut, env: Env, msg: Vec<u8>) -> AnyResult<Response>;

    fn reply(&self, deps: DepsMut, env: Env, data: Reply) -> AnyResult<Response>;
}

impl<Init, Exec, Error, Query, Sudo, Migrate> Default
    for Contract<Init, Exec, Error, Query, Sudo, Migrate>
{
    fn default() -> Self {
        Self {
            init_fn: Box::new(|_, _, _, _| -> Result<Response, Error> { Ok(Response::default()) }),
            exec_fn: Box::new(|_, _, _, _| -> Result<Response, Error> { Ok(Response::default()) }),
            query_fn: Some(Box::new(|_, _, _| -> StdResult<Binary> {
                Ok(to_binary(&Empty {})?)
            })),
            sudo_fn: Some(Box::new(|_, _, _, _| -> Result<Response, Error> {
                Ok(Response::default())
            })),
            reply_fn: Some(Box::new(|_, _, _| -> Result<Response, Error> {
                Ok(Response::default())
            })),
            migrate_fn: Some(Box::new(|_, _, _| -> Result<Response, Error> {
                Ok(Response::default())
            })),
        }
    }
}

impl<Init, Exec, Error, Query, Sudo, Migrate> Contract<Init, Exec, Error, Query, Sudo, Migrate> {
    pub fn with_query(
        init: Box<FnInit<Init, Error>>,
        exec: Box<FnExec<Exec, Error>>,
        query: Box<FnQuery<Query>>,
    ) -> Self {
        let mut contract = Contract::default();

        contract.init_fn = init;
        contract.exec_fn = exec;
        contract.query_fn = Some(query);

        contract
    }
}

impl<Init, Exec, Error, Query, Sudo, Migrate> ContractLogic
    for Contract<Init, Exec, Error, Query, Sudo, Migrate>
where
    Init: DeserializeOwned + Debug,
    Exec: DeserializeOwned,
    Query: DeserializeOwned,
    Sudo: DeserializeOwned,
    Migrate: DeserializeOwned,
    Error: Display + Debug + Send + Sync + 'static,
{
    fn instantiate(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Vec<u8>,
    ) -> AnyResult<Response> {
        let msg = from_slice(&msg)?;
        (self.init_fn)(deps, env, info, msg).map_err(|err| anyhow!(err))
    }

    fn execute(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Vec<u8>,
    ) -> AnyResult<Response> {
        let msg = from_slice(&msg)?;
        (self.exec_fn)(deps, env, info, msg).map_err(|err| anyhow!(err))
    }

    fn query(&self, deps: Deps, env: Env, msg: Vec<u8>) -> AnyResult<Binary> {
        let Some(query) = &self.query_fn else {
            bail!("query not implemented for contract")
        };
        let msg = from_slice(&msg)?;
        (query)(deps, env, msg).map_err(|err| anyhow!(err))
    }

    fn migrate(&self, deps: DepsMut, env: Env, msg: Vec<u8>) -> AnyResult<Response> {
        let Some(migrate) = &self.migrate_fn else {
            bail!("migrate not implemented for contract")
        };
        let msg = from_slice(&msg)?;
        migrate(deps, env, msg).map_err(|err| anyhow!(err))
    }

    fn reply(&self, deps: DepsMut, env: Env, data: Reply) -> AnyResult<Response> {
        let Some(reply) = &self.reply_fn else {
            bail!("reply not implemented for contract")
        };
        reply(deps, env, data).map_err(|err| anyhow!(err))
    }
}

pub type ContractLogicWrapper = Box<dyn ContractLogic + 'static>;
