// Inspired by @javiermendonca StakingRobot
// https://github.com/javierjmc/dao-contracts/commit/dd9cf0ae8a21e7a02fa9a38ad6892bc1a960c8f4#diff-8ba21c3b286ab510ffe18ca0f505731adbb3e7ca4064036a7e3e5fd6f7acd6da

use std::{
  collections::HashMap,
  fmt::Debug
};

use cosmwasm_std::{
  Addr,
  Coin
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

pub struct Roboto {
  pub app: App,
  pub sender: String,
  pub contracts: HashMap<String, RobotoKnownContract>,
}

impl Roboto {
  pub fn new(
    app: App,
    sender: String,
  ) -> Self {
    Self {
      app,
      sender,
      contracts: Default::default(),
    }
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

    let res = self
      .app
      .instantiate_contract(
        code_id,
        Addr::unchecked(self.sender.clone()),
        &contract.msg,
        &[],
        label,
        None
      );

    self.contracts.insert(String::from(label), RobotoKnownContract {
      code_id: Some(code_id),
      addr: Some(res.unwrap()),
    });

    self
  }

  pub fn exec<T>(
    &mut self,
    label: &str,
    msg: T,
    funds: Option<&[Coin]>,
    processor: Option<fn(res: AppResponse)>,
  ) -> &mut Self
    where
    T: Serialize + Debug
  {
    let send_funds: &[Coin] = if funds.is_none() {
      &[]
    } else {
      funds.unwrap()
    };

    let res = self
      .app
      .execute_contract(
        Addr::unchecked(self.sender.clone()),
        self.contracts[label].addr.clone().unwrap(),
        &msg,
        &send_funds
      ).unwrap();

    processor.map(|proc| proc(res));

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

  pub fn step(
    &mut self,
    processor: &mut dyn for<'r> FnMut(&'r mut Self),
  ) -> &mut Self {
    processor(self);
    self
  }
}