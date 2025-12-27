#[cfg(test)]
mod bank_test {
    use crate::{app::App, module::ModuleLogic, modules::bank::BankSudo};
    use cosmwasm_std::{
        coin, coins, from_json, Addr, BankMsg, BankQuery, BalanceResponse, AllBalanceResponse, QueryRequest,
    };

    fn assert_balance(app: &mut App, address: &Addr, denom: &str, expected_amount: u128) {
        let res = app
            .query(QueryRequest::Bank(BankQuery::Balance {
                address: address.to_string(),
                denom: denom.to_string(),
            }))
            .unwrap();
        let balance: BalanceResponse = from_json(&res).unwrap();
        assert_eq!(balance.amount, coin(expected_amount, denom));
    }

    #[test]
    fn bank_operations() {
        let mut app = App::new();

        let sender = Addr::unchecked("sender");
        let receiver = Addr::unchecked("receiver");
        let denom = "uatom";

        // 1. Mint funds to sender
        app.router.bank.sudo(
            &app.api,
            &mut app.storage,
            &app.querier,
            &app.env.env.block,
            BankSudo::Mint {
                to_address: sender.to_string(),
                amount: coins(100, denom),
            },
        ).expect("sudo mint should not fail");

        // 2. Check sender balance
        assert_balance(&mut app, &sender, denom, 100);

        // 3. Send funds from sender to receiver
        // App::new() defaults sender to "sender"
        app.execute(cosmwasm_std::CosmosMsg::Bank(BankMsg::Send {
            to_address: receiver.to_string(),
            amount: coins(40, denom),
        })).into_iter().for_each(|r| { r.unwrap(); });

        // 4. Check sender balance (should be 60)
        assert_balance(&mut app, &sender, denom, 60);

        // 5. Check receiver balance (should be 40)
        assert_balance(&mut app, &receiver, denom, 40);

        // 6. Burn funds from sender
        app.execute(cosmwasm_std::CosmosMsg::Bank(BankMsg::Burn {
            amount: coins(10, denom),
        })).into_iter().for_each(|r| { r.unwrap(); });

        // 7. Check sender balance (should be 50)
        assert_balance(&mut app, &sender, denom, 50);

        // 8. Test AllBalances
        let res = app.query(QueryRequest::Bank(BankQuery::AllBalances {
            address: sender.to_string(),
        })).unwrap();
        let all_balances: AllBalanceResponse = from_json(&res).unwrap();
        assert_eq!(all_balances.amount.len(), 1);
        assert_eq!(all_balances.amount[0], coin(50, denom));
    }

    #[test]
    fn ibc_routing() {
        let mut app = App::new();
        // Just check if we can query IBC without panic
        let res = app.query(QueryRequest::Ibc(cosmwasm_std::IbcQuery::ListChannels { port_id: None }));
        assert_eq!(res.unwrap_err().to_string(), "ibc query unsupported");
    }
}
