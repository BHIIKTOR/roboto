#[cfg(test)]
mod bank_test {
    use crate::{app::App, module::ModuleLogic, modules::bank::BankSudo};
    use cosmwasm_std::{
        coin, coins, Addr, BankMsg, BankQuery, BalanceResponse, AllBalanceResponse, QueryRequest,
    };

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
        ).unwrap();

        // 2. Check sender balance
        let res = app.query(QueryRequest::Bank(BankQuery::Balance {
            address: sender.to_string(),
            denom: denom.to_string(),
        })).unwrap();
        let balance: BalanceResponse = cosmwasm_std::from_binary(&res).unwrap();
        assert_eq!(balance.amount, coin(100, denom));

        // 3. Send funds from sender to receiver
        // App::new() defaults sender to "sender"
        app.execute(cosmwasm_std::CosmosMsg::Bank(BankMsg::Send {
            to_address: receiver.to_string(),
            amount: coins(40, denom),
        }));

        // 4. Check sender balance (should be 60)
        let res = app.query(QueryRequest::Bank(BankQuery::Balance {
            address: sender.to_string(),
            denom: denom.to_string(),
        })).unwrap();
        let balance: BalanceResponse = cosmwasm_std::from_binary(&res).unwrap();
        assert_eq!(balance.amount, coin(60, denom));

        // 5. Check receiver balance (should be 40)
        let res = app.query(QueryRequest::Bank(BankQuery::Balance {
            address: receiver.to_string(),
            denom: denom.to_string(),
        })).unwrap();
        let balance: BalanceResponse = cosmwasm_std::from_binary(&res).unwrap();
        assert_eq!(balance.amount, coin(40, denom));

        // 6. Burn funds from sender
        app.execute(cosmwasm_std::CosmosMsg::Bank(BankMsg::Burn {
            amount: coins(10, denom),
        })).into_iter().for_each(|r| r.unwrap());

        // 7. Check sender balance (should be 50)
        let res = app.query(QueryRequest::Bank(BankQuery::Balance {
            address: sender.to_string(),
            denom: denom.to_string(),
        })).unwrap();
        let balance: BalanceResponse = cosmwasm_std::from_binary(&res).unwrap();
        assert_eq!(balance.amount, coin(50, denom));

        // 8. Test AllBalances
        let res = app.query(QueryRequest::Bank(BankQuery::AllBalances {
            address: sender.to_string(),
        })).unwrap();
        let all_balances: AllBalanceResponse = cosmwasm_std::from_binary(&res).unwrap();
        assert_eq!(all_balances.amount.len(), 1);
        assert_eq!(all_balances.amount[0], coin(50, denom));
    }

    #[test]
    fn ibc_routing() {
        let mut app = App::new();
        // Just check if we can query IBC without panic
        let res = app.query(QueryRequest::Ibc(cosmwasm_std::IbcQuery::ListChannels { port_id: None }));
        assert_eq!(res.unwrap_err().to_string(), "Generic error: ibc query unsupported");
    }
}
