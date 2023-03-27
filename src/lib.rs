// Inspired by @javiermendonca StakingRobot
// https://github.com/javierjmc/dao-contracts/commit/dd9cf0ae8a21e7a02fa9a38ad6892bc1a960c8f4#diff-8ba21c3b286ab510ffe18ca0f505731adbb3e7ca4064036a7e3e5fd6f7acd6da

use anyhow::Result as AnyResult;

use std::{
  collections::HashMap,
  fmt::{Debug, Display},
};

use cosmwasm_std::{
  Addr, Coin, BlockInfo,
};

use cw_multi_test::{
  App,
  Contract,
  Executor,
  AppResponse,
};

use serde::{
  de::DeserializeOwned,
  Serialize
};

pub struct RobotoContractDataStruct<T> {
  pub init: fn() -> Box<(dyn Contract<cosmwasm_std::Empty> + 'static)>,
  pub msg: T
}

pub type RobotoContractData<T> = RobotoContractDataStruct<T>;

impl<T> RobotoContractData<T> {
  pub fn new(
    init: fn() -> Box<(dyn Contract<cosmwasm_std::Empty> + 'static)>,
    msg: T
  ) -> Self {
    Self {
        init,
        msg,
    }
  }
}

#[derive(Debug)]
pub struct RobotoKnownContract {
  pub code_id: Option<u64>,
  pub addr: Option<Addr>,
}

pub struct Roboto<'a> {
  pub app: App,
  pub sender: String,
  pub contracts: HashMap<String, RobotoKnownContract>,
  // pub error_handler: Option<fn(res: &anyhow::Error)>,
  pub funds: Option<&'a [Coin]>,
}

impl<'a> Roboto<'a> {
  pub fn new(
    app: App,
    sender: String
  ) -> Self {
    Self {
      app,
      sender,
      contracts: Default::default(),
      funds: None,
    }
  }
  pub fn set_block(
    &mut self,
    height: fn(&mut BlockInfo) -> &mut BlockInfo
  ) -> &mut Self {
    self.app.update_block(|mut block| {
      let b = height(block).clone();
      block.chain_id = b.chain_id;
      block.time = b.time;
      block.height = b.height;
    });
    self
  }

  pub fn set_sender(
    &mut self,
    sender: String,
  ) -> &mut Self {
    self.sender = sender;
    self
  }

  pub fn set_funds(
    &mut self,
    funds: Option<&'a [Coin]>,
  ) -> &mut Self {
    self.funds = funds;
    self
  }

  pub fn add_balance(
    &mut self,
    recipient: impl Into<String>,
    coins: Vec<Coin>
  ) -> &mut Self {
    self.app.init_modules(|router, _api, storage| {
      router
      .bank
      .init_balance(
          storage,
          &Addr::unchecked(recipient),
          coins
      )
      .unwrap();
    });
    self
  }

  pub fn init<T>(
    &mut self,
    label: &str,
    contract: RobotoContractData<T>
  ) -> &mut Self
    where
      T: Serialize
  {
    let code_id = self.app.store_code((contract.init)());

    let send_funds = self.funds.unwrap_or(&[]);

    let res = self
      .app
      .instantiate_contract(
        code_id,
        Addr::unchecked(self.sender.clone()),
        &contract.msg,
        send_funds,
        label,
        None
      );

    // clear the funds for next operation
    self.funds = None;

    self.contracts.insert(String::from(label), RobotoKnownContract {
      code_id: Some(code_id),
      addr: Some(res.unwrap()),
    });

    self
  }

  pub fn exec<T, B>(
    &mut self,
    label: &str,
    msg: T,
    handler: Option<fn(res: AnyResult<AppResponse, B>)>,
  ) -> &mut Self
    where
    T: Serialize + Debug,
    B: Debug + Display + Sync + Send + 'static
  {
    let send_funds = self.funds.unwrap_or(&[]);

    let res = self
      .app
      .execute_contract(
        Addr::unchecked(self.sender.clone()),
        self.contracts[label].addr.clone().unwrap(),
        &msg,
        send_funds
      );

    // clear the funds for next operation
    self.funds = None;

    if let Some(handle) = handler {
      match res {
        Ok(o) => handle(Ok(o)),
        Err(err) => handle(Err(err.downcast::<B>().unwrap())),
      }
    }

    self
  }

  pub fn query<T, B>(
    &mut self,
    label: &str,
    msg: B
  ) -> Result<T, cosmwasm_std::StdError>
    where
      T: DeserializeOwned,
      B: Serialize + Debug
  {
    self
      .app
      .wrap()
      .query_wasm_smart::<T>(
        self.contracts[label].addr.clone().unwrap(),
        &msg
      )
  }

  pub fn queryr<T, B>(
    &mut self,
    label: &str,
    msg: B,
    handler: fn(Result<T, cosmwasm_std::StdError>)
  ) -> &mut Self
    where
      T: DeserializeOwned,
      B: Serialize + Debug
  {
    let res = self
      .app
      .wrap()
      .query_wasm_smart::<T>(
        self.contracts[label].addr.clone().unwrap(),
        &msg
      );

    handler(res);

    self
  }

  pub fn step(
    &mut self,
    processor: &mut dyn for<'r> FnMut(&'r mut Self),
  ) -> &mut Self {
    processor(self);
    self
  }
}