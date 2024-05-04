use std::collections::HashMap;

use anyhow::{bail, Result as AnyResult};

use crate::{app::AppResponse, env::RobotoEnv, module::ModuleLogic};
use cosmwasm_std::{
    testing::{MockApi, MockQuerier, MockStorage},
    to_json_binary, AllBalanceResponse, AllDenomMetadataResponse, BalanceResponse, BankMsg,
    BankQuery, Coin, DenomMetadata, DenomMetadataResponse, Empty, Event, MessageInfo,
    SupplyResponse, Uint128,
};

use schemars::JsonSchema;

fn coins_to_string(coins: &[Coin]) -> String {
    coins
        .iter()
        .map(|c| format!("{}{}", c.amount, c.denom))
        .collect::<Vec<String>>()
        .join(",")
}

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
    pub supply: HashMap<String, Uint128>,
    pub metadata: HashMap<String, DenomMetadata>,
}

pub enum OPMODE {
    INC,
    DEC,
}

impl Bank {
    pub fn new() -> Self {
        Self::default()
    }

    fn increase_supply(&mut self, coin: Coin) -> AnyResult<()> {
        match self.supply.get_mut(&coin.denom) {
            Some(supply) => *supply += coin.amount,
            None => {
                let mut supply = Uint128::zero();
                supply += coin.amount;
                self.supply.insert(coin.denom, supply);
            }
        }
        Ok(())
    }

    fn get_balance(&mut self, address: &String, denom: &String) -> AnyResult<&mut Coin> {
        let wallet: &mut HashMap<String, Coin>;

        if self.balances.contains_key(address) {
            wallet = self.balances.get_mut(address).unwrap();
        } else {
            self.balances.insert(address.clone(), HashMap::new());
            wallet = self.balances.get_mut(address).unwrap();
        }

        let coin = wallet.get_mut(denom);

        if coin.is_none() {
            let coin = &mut Coin {
                denom: denom.to_string(),
                amount: Uint128::zero(),
            };
            wallet.insert(denom.clone(), coin.clone());
        }

        Ok(wallet.get_mut(denom).unwrap())
    }

    fn add_balance(coin: &mut Coin, amount: Uint128) -> AnyResult<&Coin> {
        coin.amount = coin.amount.checked_add(amount).unwrap();
        Ok(coin)
    }

    fn sub_balance(coin: &mut Coin, amount: Uint128) -> AnyResult<&Coin> {
        coin.amount = coin.amount.checked_sub(amount).unwrap();
        Ok(coin)
    }

    fn update_balance(
        &mut self,
        mode: OPMODE,
        address: String,
        amount: Vec<Coin>,
    ) -> AnyResult<()> {
        match self.balances.get_mut(&address) {
            Some(wallet) => {
                let mut coins = amount.clone();
                while let Some(coin) = coins.pop() {
                    match wallet.get_mut(&coin.denom) {
                        Some(exists) => {
                            match mode {
                                OPMODE::INC => Bank::add_balance(exists, coin.amount).unwrap(),
                                OPMODE::DEC => Bank::sub_balance(exists, coin.amount).unwrap(),
                            };
                        }
                        None => {
                            wallet.insert(coin.clone().denom, coin);
                        }
                    }
                }
                Ok(())
            }
            None => {
                self.balances.insert(address.clone(), HashMap::default());
                self.update_balance(OPMODE::INC, address, amount).unwrap();
                Ok(())
            }
        }
    }

    fn check_balance(&mut self, from: &String, amount: &Coin) -> AnyResult<()> {
        if self.balances.is_empty() {
            bail!("no balances")
        }

        let from_balance = self.get_balance(from, &amount.denom).unwrap();
        if from_balance.amount.lt(&amount.amount) {
            bail!("sender does not have enough balance")
        }

        Ok(())
    }

    fn send(&mut self, from: String, to: String, amount: Coin) -> AnyResult<()> {
        self.check_balance(&from, &amount).unwrap();
        self.update_balance(OPMODE::DEC, from.clone(), vec![amount.clone()])
            .unwrap();
        self.update_balance(OPMODE::INC, to, vec![amount]).unwrap();
        Ok(())
    }

    fn send_many(&mut self, from: String, to: String, amount: Vec<Coin>) -> AnyResult<()> {
        amount
            .into_iter()
            .for_each(|x| self.send(from.clone(), to.clone(), x).unwrap());
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
        _env: &mut RobotoEnv,
        info: &MessageInfo,
        msg: Self::ExecM,
    ) -> anyhow::Result<AppResponse> {
        match msg {
            BankMsg::Send { to_address, amount } => {
                let _ = self
                    .send_many(info.sender.to_string(), to_address.clone(), amount.clone())
                    .unwrap();
                let events = vec![Event::new("transfer")
                    .add_attribute("recipient", &to_address)
                    .add_attribute("sender", &info.sender)
                    .add_attribute("amount", coins_to_string(&amount))];
                Ok(AppResponse {
                    events,
                    data: None,
                    responses: vec![],
                })
            }
            BankMsg::Burn { amount } => {
                amount.into_iter().for_each(|coin| {
                    self.check_balance(&info.sender.to_string(), &coin).unwrap();
                });
                Ok(AppResponse::default())
            }
            _ => bail!("wrong BankMsg msg"),
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
                let balance = self.balances.get(&address).unwrap().get(&denom).unwrap();
                Ok(to_json_binary(&BalanceResponse::new(balance.clone())).unwrap())
            }
            BankQuery::AllBalances { address } => {
                let balance = self.balances.get(&address).unwrap();
                Ok(to_json_binary(&AllBalanceResponse::new(
                    balance
                        .into_iter()
                        .filter(|x| !x.1.amount.is_zero())
                        .map(|x| x.1.clone())
                        .collect::<Vec<Coin>>(),
                ))
                .unwrap())
            }
            #[cfg(feature = "cosmwasm_1_1")]
            BankQuery::Supply { denom } => Ok(to_json_binary(&SupplyResponse::new(Coin {
                denom: denom.clone(),
                amount: self.supply.get(&denom).unwrap().clone(),
            }))
            .unwrap()),
            #[cfg(feature = "cosmwasm_1_3")]
            BankQuery::DenomMetadata { denom } => Ok(to_json_binary(&DenomMetadataResponse::new(
                self.metadata.get(&denom).unwrap().clone(),
            ))
            .unwrap()),
            #[cfg(feature = "cosmwasm_1_3")]
            BankQuery::AllDenomMetadata { pagination: _ } => {
                // if let Some(page) = pagination {}
                let data = self
                    .metadata
                    .iter()
                    .map(|item| item.1.clone())
                    .collect::<Vec<DenomMetadata>>();
                Ok(to_json_binary(&AllDenomMetadataResponse::new(data, None)).unwrap())
            }
            _ => panic!("wrong BankQuery msg"),
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
                self.update_balance(OPMODE::INC, to_address, amount.clone())
                    .unwrap();
                self.increase_supply(amount.first().unwrap().clone())
                    .unwrap();
                Ok(AppResponse::default())
            }
        }
    }
}

#[cfg(test)]
mod bank {
    use cosmwasm_std::{
        from_json,
        testing::{mock_env, mock_info, MockApi, MockQuerier, MockStorage},
        AllBalanceResponse, BalanceResponse, BankMsg, BankQuery, Coin, Env,
        SupplyResponse, Uint128,
    };

    use super::{Bank, BankSudo};
    use crate::{env::RobotoEnv, module::ModuleLogic};

    const DENOM: &str = "utaco";
    const AMOUNT: u128 = 1_000_000u128;

    fn start_mocks() -> (MockApi, MockStorage, MockQuerier, Env) {
        (
            MockApi::default(),
            MockStorage::default(),
            MockQuerier::default(),
            mock_env(),
        )
    }

    fn mint() -> (Bank, String, Uint128, String) {
        let (api, mut storage, querier, env) = start_mocks();

        let denom = DENOM.to_string();
        let amount = Uint128::from(AMOUNT);
        let address = api.addr_make("reciever").to_string();

        let mut bank = Bank::new();

        let mint = bank.sudo(
            &api,
            &mut storage,
            &querier,
            &env.block,
            BankSudo::Mint {
                to_address: address.clone(),
                amount: vec![Coin {
                    denom: denom.clone(),
                    amount,
                }],
            },
        );

        assert!(mint.is_ok());

        (bank, denom, amount, address)
    }

    #[test]
    fn balance() {
        let (api, mut storage, querier, _) = start_mocks();

        let (bank, denom, amount, address) = mint();

        let res = bank.query(
            &api,
            &mut storage,
            &querier,
            &mut RobotoEnv::new(),
            BankQuery::Balance { address, denom },
        );

        let res = from_json::<BalanceResponse>(&res.unwrap()).unwrap();

        assert_eq!(res.amount.amount, amount);
    }

    #[test]
    fn supply() {
        let (api, mut storage, querier, _) = start_mocks();

        let (bank, _, amount, _) = mint();

        let res = bank.query(
            &api,
            &mut storage,
            &querier,
            &mut RobotoEnv::new(),
            BankQuery::Supply {
                denom: DENOM.to_string(),
            },
        );

        let res = from_json::<SupplyResponse>(&res.unwrap()).unwrap();

        assert_eq!(res.amount.amount, amount);
    }

    #[test]
    fn all_balances() {
        let (api, mut storage, querier, _) = start_mocks();

        let (bank, _, amount, address) = mint();

        let res = bank.query(
            &api,
            &mut storage,
            &querier,
            &mut RobotoEnv::new(),
            BankQuery::AllBalances { address },
        );

        let res = from_json::<AllBalanceResponse>(&res.unwrap()).unwrap();
        assert_eq!(res.amount.first().unwrap().amount, amount);
    }

    #[test]
    fn send() {
        let (api, mut storage, querier, _) = start_mocks();

        let to_address = api.addr_make("to_address").to_string();

        let (mut bank, denom, amount, from_address) = mint();

        let info = mock_info(&from_address, &vec![]);

        let res = bank
            .execute(
                &api,
                &mut storage,
                &querier,
                &mut RobotoEnv::new(),
                &info,
                BankMsg::Send {
                    to_address: to_address.clone(),
                    amount: vec![Coin { denom, amount }],
                },
            )
            .unwrap();

        let event = res.events.first().unwrap();
        assert_eq!(event.attributes[0].value, to_address.clone());
        assert_eq!(event.attributes[1].value, from_address.clone());

        let res = bank.query(
            &api,
            &mut storage,
            &querier,
            &mut RobotoEnv::new(),
            BankQuery::AllBalances {
                address: from_address,
            },
        );

        let res = from_json::<AllBalanceResponse>(&res.unwrap()).unwrap();
        assert!(res.amount.is_empty());

        let res = bank.query(
            &api,
            &mut storage,
            &querier,
            &mut RobotoEnv::new(),
            BankQuery::AllBalances {
                address: to_address,
            },
        );

        let res = from_json::<AllBalanceResponse>(&res.unwrap()).unwrap();
        assert_eq!(res.amount.first().unwrap().amount, amount);
    }
}
