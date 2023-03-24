# Roboto

***Domo arigato Mr. Roboto***

## Example

```Rust
use cw_multi_test::{
  App
};

use crate::{
  msg::{
    InstantiateMsg,
    QueryMsg,
    ChildInitMsg,
    ListKnownContract
  }
};

const CREATOR: &str = &"creator";
const NFT_CUSTOM: &str = &"nft_custom";
const ORQUESTER: &str = &"orquester";

// .exec(crate::msg::ExecuteMsg::Pause(), None, Some(|res: AppResponse| {
//   let data: Config = from_binary(&res.data.unwrap()).unwrap();
// }));

use roboto::roboto::{
  Roboto,
  RobotoContractData,
};

#[cfg(test)]
mod general {
  use crate::msg::ExecuteMsg;
  use super::*;

  #[test]
  fn init() {
    let init_nft_custom = cw721_custom::msg::InstantiateMsg::new(String::from(CREATOR));

    let mut roboto: Roboto = Roboto::new(App::default());

    let mut orquester_init = RobotoContractData::<InstantiateMsg>::new(
      crate::tests::helpers::nft_orquester_contract,
      InstantiateMsg::new(0u64)
    );

    let custom_init = RobotoContractData::<cw721_custom::msg::InstantiateMsg>::new(
      crate::tests::helpers::nft_custom_contract,
      init_nft_custom.clone()
    );

    let init_child_msg = crate::msg::ExecuteMsg::ChildInit(ChildInitMsg {
      name: String::from(NFT_CUSTOM),
      msg: init_nft_custom,
      send_remote: false,
    });

    let mut set_child_id = |r: &mut Roboto| {
      let c = r.contracts.get(NFT_CUSTOM).unwrap();
      orquester_init.msg.child = c.code_id.unwrap();
    };

    roboto
      .init(NFT_CUSTOM, custom_init)
      .step(&mut set_child_id)
      .init(ORQUESTER, orquester_init)
      .exec::<ExecuteMsg>(ORQUESTER, init_child_msg, None, None);

    let res = roboto.query::<ListKnownContract, QueryMsg>(ORQUESTER, QueryMsg::GetContractKeys{}).unwrap();

    println!("QueryMsg::ListKnownContract: {:#?}", res);
    println!("roboto.contracts: {:#?}", roboto.contracts);

    assert!(false)
  }
}
```

