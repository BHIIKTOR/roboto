use std::collections::HashMap;

use crate::{
    app::AppResponse, env::RobotoEnv, module::ModuleLogic,
};
use anyhow::bail;
use cosmwasm_std::{
    testing::{MockApi, MockQuerier, MockStorage},
    to_binary, BankMsg, BankQuery, Coin, MessageInfo,
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

impl Bank {
    fn update_balance(&mut self, address: String, amount: Vec<Coin>) {
        match self.balances.get_mut(&address) {
            Some(wallet) => {
                let mut coins = amount.clone();
                while let Some(coin) = coins.pop() {
                    match wallet.get_mut(&coin.denom) {
                        Some(exists) => {
                            exists.amount += coin.amount;
                        }
                        None => {
                            wallet.insert(coin.clone().denom, coin);
                        }
                    }
                }
            }
            None => {
                self.balances.insert(address.clone(), HashMap::default());
                self.update_balance(address, amount);
            }
        }
    }

    fn sub_balance(
        &mut self,
        address: &str,
        amount: &[Coin],
    ) -> anyhow::Result<()> {
        let Some(wallet) = self.balances.get_mut(address) else {
            bail!("Wallet not found for address: {}", address);
        };

        for coin in amount {
            let Some(exists) = wallet.get_mut(&coin.denom) else {
                bail!("Coin not found: {}", coin.denom);
            };

            if exists.amount < coin.amount {
                 bail!("Insufficient funds: {} < {}", exists.amount, coin.amount);
            }
            exists.amount -= coin.amount;
        }
        Ok(())
    }
}

impl ModuleLogic for Bank {
    type ExecM = BankMsg;
    type QueryM = BankQuery;
    type SudoM = BankSudo;

    fn execute(
        &mut self,
        _api: &MockApi,
        _storage: &mut MockStorage,
        _querier: &MockQuerier,
        env: &mut RobotoEnv,
        info: &MessageInfo,
        msg: Self::ExecM,
    ) -> anyhow::Result<AppResponse> {
        match msg {
            BankMsg::Send { to_address, amount } => {
                self.sub_balance(info.sender.as_str(), amount.clone())?;
                self.update_balance(to_address, amount);
                env.increase_tx();
                Ok(AppResponse::default())
            }
            BankMsg::Burn { amount } => {
                self.sub_balance(info.sender.as_str(), amount)?;
                env.increase_tx();
                Ok(AppResponse::default())
            }
            _ => bail!("bank execute unsupported"),
        }
    }

    fn query(
        &self,
        _api: &MockApi,
        _storage: &mut MockStorage,
        _querier: &MockQuerier,
        _env: &mut RobotoEnv,
        msg: Self::QueryM,
    ) -> anyhow::Result<cosmwasm_std::Binary> {
        match msg {
            BankQuery::Balance { address, denom } => {
                let amount = if let Some(wallet) = self.balances.get(&address) {
                    if let Some(coin) = wallet.get(&denom) {
                         coin.clone()
                    } else {
                         Coin::new(0, denom)
                    }
                } else {
                    Coin::new(0, denom)
                };
                to_binary(&cosmwasm_std::BalanceResponse { amount }).map_err(Into::into)
            }
            BankQuery::AllBalances { address } => {
                let amount = if let Some(wallet) = self.balances.get(&address) {
                    wallet.values().cloned().collect()
                } else {
                    vec![]
                };
                to_binary(&cosmwasm_std::AllBalanceResponse { amount }).map_err(Into::into)
            }
            _ => bail!("bank query unsupported"),
        }
    }

    fn sudo(
        &mut self,
        _api: &MockApi,
        _storage: &mut MockStorage,
        _querier: &MockQuerier,
        _block: &cosmwasm_std::BlockInfo,
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
