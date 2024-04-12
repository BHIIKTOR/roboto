use cosmwasm_std::{Addr, BlockInfo, ContractInfo, Env, Timestamp, TransactionInfo};

use crate::app::App;

pub struct RobotoEnv {
    pub env: Env,
}

impl RobotoEnv {
    pub fn new() -> Self {
        Self {
            env: Env {
                block: BlockInfo {
                    height: 12_345,
                    time: Timestamp::from_nanos(1_571_797_419_879_305_533),
                    chain_id: "cosmos-testnet-14002".to_string(),
                },
                transaction: Some(TransactionInfo { index: 0 }),
                contract: ContractInfo {
                    address: Addr::unchecked(""),
                },
            },
        }
    }

    pub fn increase_tx(&mut self) {
        if let Some(tx) = &self.env.transaction {
            self.env.transaction = Some(TransactionInfo {
                index: tx.index.checked_add(1).unwrap(),
            })
        }
    }

    pub fn set_time(&mut self, ts: Timestamp) {
        self.env.block.time = ts;
    }

    pub fn set_height(&mut self, height: u64) {
        self.env.block.height = height;
    }

    pub fn set_contract(&mut self, address: &String) {
        self.env.contract.address = Addr::unchecked(address);
    }

    pub fn set_chain_id(&mut self, chain_id: String) {
        self.env.block.chain_id = chain_id;
    }

    pub fn inner(&self) -> Env {
        self.env.clone()
    }
}
