# Roboto

***Domo arigato Mr. Roboto***

## Examples

```Rust
#[cfg(test)]
mod general {
    use cw_multi_test::App;
    use roboto::roboto::{Roboto, RobotoContractData};
    use cw721_base::MintMsg;
    use crate::msg::{StoreConfMsg, StoreConf, InstantiateMsg};
    use crate::tests::test_helpers::tests_helpers::nft_custom_contract;
    use crate::{
        msg::ExecuteMsg,
        tests::test_helpers::tests_helpers::{
            get_store_batch_msg,
            get_init_msg,
        }
    };

    const ADMIN: &str = "admin";
    const NFT_CUSTOM: &str = &"nft_custom";

    #[test]
    fn store() {
        let init_msg = get_init_msg(0, 900);

        let mut roboto = Roboto::new(App::default(), ADMIN.to_string());

        let init = RobotoContractData::<InstantiateMsg>::new(
            nft_custom_contract,
            init_msg
        );

        let store_one = ExecuteMsg::Store(MintMsg {
            token_id: String::from("0"),
            owner: ADMIN.to_string(),
            token_uri: None,
            extension: None,
        });

        let store_batch = ExecuteMsg::StoreBatch(get_store_batch_msg(20));

        let store_conf = ExecuteMsg::StoreConf(StoreConfMsg {
            conf: Some(StoreConf {
                name: String::from("nft"),
                desc: String::from("nft"),
                ipfs: String::from("nft"),
                attributes: vec![String::from("value"), String::from("something")],
            }),
            attributes: vec![
                vec![String::from("value"), String::from("something")],
                vec![String::from("value"), String::from("something")]
            ],
        });

        roboto
            .init(NFT_CUSTOM, init)
            .exec::<ExecuteMsg>(NFT_CUSTOM, store_one, Some(|res| {
                assert_eq!(res.events[1].attributes[1].value, "store");
            }))
            .exec::<ExecuteMsg>(NFT_CUSTOM, store_batch, Some(|res| {
                assert_eq!(res.events[1].attributes[1].value, "store_batch");
            }))
            .exec::<ExecuteMsg>(NFT_CUSTOM, store_conf, Some(|res| {
                assert_eq!(res.events[1].attributes[1].value, "store_conf");
            }));
    }
}
```

```Rust
#[test]
fn roboto_mint() {
    let mut init_msg = get_init_msg(0, 12000000000);
    init_msg.max_mint_batch = Some(Uint128::from(10u128));

    let mut roboto = Roboto::new(App::default(), ADMIN.to_string());

    let init_custom = RobotoContractData::<InstantiateMsg>::new(
        nft_custom_contract,
        init_msg
    );

    let store_batch = ExecuteMsg::StoreBatch(get_store_batch_msg(40));

    let exec_mint = ExecuteMsg::Mint();

    let exec_mint_batch = ExecuteMsg::MintBatch(MintBatchMsg {
        amount: Uint128::from(10u32)
    });

    let exec_mint_incorrect_funds = ExecuteMsg::MintBatch(MintBatchMsg {
        amount: Uint128::from(10u32)
    });

    let exec_mint_too_large = ExecuteMsg::MintBatch(MintBatchMsg {
        amount: Uint128::from(11u32)
    });

    roboto
        .set_sender(ADMIN.to_string())
        .add_balance(MINTER, vec![Coin::new(1000000000u128, DENOM.to_string())])
        .init(NFT_CUSTOM, init_custom)
        .exec::<ExecuteMsg, ContractError>(NFT_CUSTOM, store_batch, Some(|res| {
            assert_eq!(res.unwrap().events[1].attributes[1].value, "store_batch");
        }))
        .set_block(|_block| 100000000u64)
        .set_funds(Some(&vec![Coin::new(4000000u128, DENOM.to_string())]))
        .set_sender(MINTER.to_string())
        .exec::<ExecuteMsg, ContractError>(NFT_CUSTOM, exec_mint, Some(|res| {
            assert_eq!(res.unwrap().events[1].attributes[1].value, "mint");
        }))
        .set_funds(Some(&vec![Coin::new(40000000u128, DENOM.to_string())]))
        .exec::<ExecuteMsg, ContractError>(NFT_CUSTOM, exec_mint_batch, Some(|res| {
            assert_eq!(res.unwrap().events[1].attributes[1].value, "mint_batch");
        }))
        .set_funds(Some(&vec![Coin::new(40000000u128, DENOM.to_string())]))
        .exec::<ExecuteMsg, ContractError>(NFT_CUSTOM, exec_mint_too_large, Some(|res| {
            assert!(res.unwrap_err().eq(&ContractError::MintAmountLargerThanAllowed{}))
        }))
        .set_funds(Some(&vec![Coin::new(80000000u128, DENOM.to_string())]))
        .exec::<ExecuteMsg, ContractError>(NFT_CUSTOM, exec_mint_incorrect_funds, Some(|res| {
            assert!(res.unwrap_err().eq(&ContractError::IncorrectFunds{}))
        }));
}
```

## set_block
changes the block in env
```Rust
    robot.set_block(|_block| 100000000u64)
```

## add_balance
add native toknes to a wallet.

```Rust
    robot.add_balance(MINTER, vec![coin(100, "ucosm")])
```

## set_funds
get set to `None` after each init and exec.

```Rust
  roboto.set_funds(Some(&vec![coin(100, "ucosm")]))
```
```