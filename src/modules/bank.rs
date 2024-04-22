use std::collections::HashMap;

use crate::{
    app::AppResponse, contract::ContractLogicWrapper, env::RobotoEnv, module::ModuleLogic,
};
use anyhow::bail;
use cosmwasm_std::{
    testing::{MockApi, MockQuerier, MockStorage},
    BankMsg, BankQuery, Coin, ContractInfoResponse, DepsMut, Empty, Event, MessageInfo, Uint128,
};
use schemars::JsonSchema;

#[derive(Clone, std::fmt::Debug, PartialEq, Eq, JsonSchema)]
pub enum BankSudo {
    Mint {
        to_address: String,
        amount: Vec<Coin>,
    },
}

#[derive(Default)]
pub struct Bank {
    pub balances: HashMap<String, HashMap<String, Coin>>,
}

pub enum OPMODE {
    INC,
    DEC
}

impl Bank {
    fn add_balance(coin: &mut Coin, amount: Uint128) -> &Coin {
        coin.amount += amount;
        coin
    }

    fn update_balance(&mut self, address: String, amount: Vec<Coin>) {
        match self.balances.get_mut(&address) {
            Some(wallet) => {
                let mut coins = amount.clone();
                while let Some(coin) = coins.pop() {
                    match wallet.get_mut(&coin.denom) {
                        Some(exists) => {
                            exists.amount += coin.amount;
                            Bank::add_balance(exists, coin.amount);
                        },
                        None => {
                            wallet.insert(coin.clone().denom, coin);
                        }
                    }
                }
            }
            None => {
                self
                    .balances
                    .insert(address.clone(), HashMap::default());
                self.update_balance(address, amount);
            }
        }
    }
}

impl ModuleLogic for Bank {
    type ExecM = BankMsg;
    type QueryM = BankQuery;
    type SudoM = BankSudo;

    fn execute(
        &mut self,
        api: &MockApi,
        storage: &mut MockStorage,
        querier: &MockQuerier,
        env: &mut RobotoEnv,
        info: &MessageInfo,
        msg: Self::ExecM,
    ) -> anyhow::Result<AppResponse> {
        match msg {
            BankMsg::Send { to_address, amount } => todo!(),
            BankMsg::Burn { amount } => todo!(),
            _ => bail!("bank execute unsupported"),
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
            BankQuery::Balance { address, denom } => todo!(),
            BankQuery::AllBalances { address } => todo!(),
            _ => todo!(),
        }
    }

    fn sudo(
        &mut self,
        api: &MockApi,
        storage: &mut MockStorage,
        querier: &MockQuerier,
        block: &cosmwasm_std::BlockInfo,
        msg: Self::SudoM,
    ) -> anyhow::Result<AppResponse> {
        match msg {
            BankSudo::Mint { to_address, amount } => {
                self.update_balance(to_address, amount);
                Ok(AppResponse::default())
            }
        }
    }
}
