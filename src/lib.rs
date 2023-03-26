// Inspired by @javiermendonca StakingRobot
// https://github.com/javierjmc/dao-contracts/commit/dd9cf0ae8a21e7a02fa9a38ad6892bc1a960c8f4#diff-8ba21c3b286ab510ffe18ca0f505731adbb3e7ca4064036a7e3e5fd6f7acd6da

use std::{
  collections::HashMap,
  fmt::Debug, error::Error
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

pub struct Roboto<'a> {
  pub app: App,
  pub sender: String,
  pub contracts: HashMap<String, RobotoKnownContract>,
  pub error_handler: Option<fn(res: &anyhow::Error)>,
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
      error_handler: None
    }
  }

  pub fn set_block(
    &mut self,
    height: fn() -> u64
  ) -> &mut Self {
    self.app.update_block(|mut block| block.height = height());
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

  pub fn set_error_handler(
    &mut self,
    error_handler: Option<fn(res: &anyhow::Error)>,
  ) -> &mut Self {
    self.error_handler = error_handler;
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

    let send_funds = match self.funds {
        Some(f) => f,
        None => &[],
    };

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
    ok_handler: Option<fn(res: AppResponse)>,
  ) -> &mut Self
    where
    T: Serialize + Debug
  {
    let send_funds = match self.funds {
        Some(f) => f,
        None => &[],
    };

    let res = self
      .app
      .execute_contract(
        Addr::unchecked(self.sender.clone()),
        self.contracts[label].addr.clone().unwrap(),
        &msg,
        send_funds
      );

    match res {
      Ok(res) => {
        if let Some(ok) = ok_handler {
          ok(res)
        }
      },
      Err(err) => {
        if let Some(error) = self.error_handler {
          error(&err);
          panic!("{}", &err.to_string())
        }
      },
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