# Roboto

***Domo arigato Mr. Roboto***

## **Note**

This is mostly a learning project.

Documentation wrote by GPT 😎, totally the best use case for it.
I only added the general example...

**DO NOT USE FOR ANYTHING** 😏

```
gpt4 is writing my docs 😎
is a blessed day on earth for all devs
```

## Description

This code defines a Roboto struct and its associated methods, which help simplify the testing of CosmWasm contracts by providing a convenient wrapper around the cw_multi_test library.

The `Roboto` struct has the following fields:

* **app**: An instance of the App struct from the cw_multi_test library.
* **sender**: A string representing the sender's address.
* **contracts**: A HashMap containing information about the known contracts, indexed by a string label.
* **funds**: An optional reference to a slice of Coin structs, which represent the funds available for an operation.

```rust
pub struct Roboto<'a> {
  pub app: App,
  pub sender: String,
  pub contracts: HashMap<String, RobotoKnownContract>,
  pub funds: Option<&'a [Coin]>,
}
```

* **RobotoContractData**: A generic struct that contains the contract initializer function and message.
* **RobotoKnownContract**: A struct that stores an optional code_id and contract addr (address).

The Roboto struct provides methods for:

* Creating a new instance of the Roboto struct.
* Setting the block information.
* Setting the sender's address.
* Setting the available funds for an operation.
* Adding an initial balance for a recipient.
* Initializing a contract.
* Executing a contract.
* Querying a contract.
* Handling the result of a query.
* Stepping through a series of operations on the Roboto struct.

The purpose of this code is to facilitate testing of CosmWasm contracts. It provides a way to easily manage contract instances, set up the environment, execute contracts, and query their state, all while maintaining a clean and organized code structure. The main advantage of using the Roboto struct is that it simplifies the process of writing tests for CosmWasm contracts by abstracting away some of the complexity of managing and executing contracts within the CosmWasm environment.

### Roboto instance methods

#### roboto.new
Constructs a new Roboto instance with the provided App and sender parameters.

```rust
pub fn new(app: App, sender: String) -> Self
```

#### roboto.set_block
Updates the block height, time, and chain ID of the Roboto instance.

```rust
pub fn set_block(&mut self, height: fn(&mut BlockInfo) -> &mut BlockInfo) -> &mut Self
```

#### roboto.set_sender
Updates the sender address of the Roboto instance.

```rust
pub fn set_sender(&mut self, sender: String) -> &mut Self
```

#### roboto.set_funds
Sets the funds to be used for the next operation.

```rust
pub fn set_funds(&mut self, funds: Option<&'a [Coin]>) -> &mut Self
```

#### roboto.add_balance
Adds an initial balance to the specified recipient.

```rust
pub fn add_balance(&mut self, recipient: impl Into<String>, coins: Vec<Coin>) -> &mut Self
```
#### roboto.init
Initializes and deploys a new contract with the provided label and contract data.

```rust
pub fn init<T>(&mut self, label: &str, contract: RobotoContractData<T>) -> &mut Self
where
    T: Serialize
```

#### roboto.exec
Executes a contract message on a deployed contract and processes the response using an optional handler.

```rust
pub fn exec<T, B>(&mut self, label: &str, msg: T, handler: Option<fn(res: AnyResult<AppResponse, B>)>) -> &mut Self
where
    T: Serialize + Debug,
    B: Debug + Display + Sync + Send + 'static
```

#### roboto.query
Queries a deployed contract using a specified query message and returns the result in a deserialized format.

```rust
pub fn query<T, B>(&mut self, label: &str, msg: B) -> Result<T, cosmwasm_std::StdError>
where
    T: DeserializeOwned,
    B: Serialize + Debug
```

#### roboto.queryr
Queries a deployed contract using a specified query message and processes the response using the provided handler.

```rust
pub fn queryr<T, B>(&mut self, label: &str, msg: B, handler: fn(Result<T, cosmwasm_std::StdError>)) -> &mut Self
where
    T: DeserializeOwned,
    B: Serialize + Debug
```

#### roboto.step
Takes a closure that accepts a mutable reference to the Roboto instance and allows users to perform additional operations on it.

```rust
pub fn step(&mut self, processor: &mut dyn for<'r> FnMut(&'r mut Self)) -> &mut Self
```

---

## General example

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

## Query example

Here's an example of querying a contract using the Roboto struct:

Assume that we have a CosmWasm contract called counter that stores an integer value. The contract exposes the following query message to get the current value of the counter:

```Rust
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum CounterQueryMsg {
    GetCount {},
}
```

Now, let's use the Roboto struct to query the current value of the counter:

```Rust
use cosmwasm_std::testing::mock_dependencies;
use cw_multi_test::AppBuilder;
use crate::counter::{CounterContract, CounterInitMsg, CounterQueryMsg};

fn main() {
    // Initialize the app and Roboto instance
    let app = AppBuilder::new().build();
    let sender = "sender_address".to_string();
    let mut roboto = Roboto::new(app, sender);

    // Initialize the counter contract
    let init_msg = CounterInitMsg { count: 0 };
    let contract = RobotoContractData::new(CounterContract::new, init_msg);
    roboto.init("counter", contract);

    // Query the counter contract
    let query_msg = CounterQueryMsg::GetCount {};
    let result: Result<u32, cosmwasm_std::StdError> = roboto.query("counter", query_msg);

    match result {
        Ok(count) => println!("The current value of the counter is: {}", count),
        Err(err) => println!("Error querying the counter: {:?}", err),
    }
}
```
In this example, we first create a new App instance and a Roboto instance with a sender address. We then initialize the counter contract using `Roboto::init` with a `RobotoContractData` instance containing the contract's initializer function and initialization message.

To query the current value of the counter, we create a `CounterQueryMsg::GetCount` message and use the Roboto::query method. The query method returns a `Result` type, which we then match to either print the current value of the counter or an error message if the query fails.

## Exec + Step example

In this example, we'll use a simple counter contract that allows incrementing and decrementing an integer value. We will use the Roboto struct to execute the Increment and Decrement messages and demonstrate the use of the exec and step methods.

First, let's define the contract messages and contract implementation:

```Rust
// counter.rs
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw2::set_contract_version;
use serde::{Deserialize, Serialize};

const CONTRACT_NAME: &str = "crates.io:counter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CounterInitMsg {
    pub count: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum CounterHandleMsg {
    Increment {},
    Decrement {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum CounterQueryMsg {
    GetCount {},
}

pub fn instantiate(
    deps: DepsMut, _env: Env, _info: MessageInfo, msg: CounterInitMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    deps.storage.set(b"count", &msg.count.to_be_bytes());
    Ok(Response::default())
}

pub fn execute(
    deps: DepsMut, _env: Env, _info: MessageInfo, msg: CounterHandleMsg,
) -> StdResult<Response> {
    match msg {
        CounterHandleMsg::Increment {} => increment(deps),
        CounterHandleMsg::Decrement {} => decrement(deps),
    }
}

fn increment(deps: DepsMut) -> StdResult<Response> {
    let count = get_count(deps.storage)?;
    let new_count = count + 1;
    deps.storage.set(b"count", &new_count.to_be_bytes());
    Ok(Response::default())
}

fn decrement(deps: DepsMut) -> StdResult<Response> {
    let count = get_count(deps.storage)?;
    let new_count = count - 1;
    deps.storage.set(b"count", &new_count.to_be_bytes());
    Ok(Response::default())
}

fn get_count(storage: &dyn cosmwasm_std::Storage) -> StdResult<i32> {
    let data = storage.get(b"count").ok_or(cosmwasm_std::StdError::not_found("count"))?;
    let count = i32::from_be_bytes(data.try_into().unwrap());
    Ok(count)
}
```

Now, let's create a test that uses the Roboto struct to execute the Increment and Decrement messages using the exec and step methods:

```rust
use cosmwasm_std::testing::mock_dependencies;
use cw_multi_test::AppBuilder;
use crate::counter::{instantiate, execute, CounterInitMsg, CounterHandleMsg, CounterQueryMsg};

fn main() {
    // Initialize the app and Roboto instance
    let app = AppBuilder::new().build();
    let sender = "sender_address".to_string();
    let mut roboto = Roboto::new(app, sender);

    // Initialize the counter contract
    let init_msg = CounterInitMsg { count: 0 };
    let contract = RobotoContractData::new(|| Box::new(ContractWrapper(instantiate, execute)), init_msg);
    roboto.init("counter", contract);

    // Define a handler to process the response after executing a message
    fn handle_response(res: AnyResult<AppResponse, cosmwasm_std::StdError>) {
        match res {
            Ok(response) => println!("Execution successful: {:?}", response),
            Err(err) => println!("Error executing the message: {:?}", err),
        }
    }

    // Execute Increment message
    let increment_msg = CounterHandleMsg::Increment {};
    roboto.exec("counter", increment_msg, Some(handle_response));

    // Execute Decrement message using the step method
    roboto.step(&mut |roboto| {
        let decrement_msg = CounterHandleMsg::Decrement {};
        roboto.exec("counter", decrement_msg, Some(handle_response));
        roboto
    });

    // Query the counter contract
    let query_msg = CounterQueryMsg::GetCount {};
    let result: Result<i32, cosmwasm_std::StdError> = roboto.query("counter", query_msg);

    match result {
        Ok(count) => println!("The final value of the counter is: {}", count),
        Err(err) => println!("Error querying the counter: {:?}", err),
    }
}
```

In this example, we first execute an Increment message using the `Roboto::exec` method. The handle_response function is passed as an argument to process the response from the execution.

Then, we execute a Decrement message using the `Roboto::step` method. The step method takes a closure that allows us to perform additional operations on the Roboto instance within the closure. In this case, we use the closure to call the exec method with the Decrement message and the handle_response function.

Finally, we query the counter contract to get the final value of the counter using the `Roboto::query` method and print the result.

***Note***
This is just an example, there are more usage cases than this

## Methods

### Roboto::exec
The exec method is a part of the Roboto struct and is responsible for executing a contract message on a deployed contract. It sends the message to the specified contract and processes the response using an optional handler.

#### Method Signature

```Rust
pub fn exec<T, B>(
    &mut self,
    label: &str,
    msg: T,
    handler: Option<fn(res: AnyResult<AppResponse, B>)>,
) -> &mut Self
where
    T: Serialize + Debug,
    B: Debug + Display + Sync + Send + 'static
```

#### Parameters

* **label**: `&str`: The label of the deployed contract. This label should match the one used when initializing the contract with Roboto::init.
* **msg**: `T`: The message to be sent to the contract. The message should implement Serialize and Debug traits.
* **handler**: `Option<fn(res: AnyResult<AppResponse, B>)>`: An optional function to process the response from the contract execution. The handler function takes a single parameter res of type `AnyResult<AppResponse, B>` where B is the error type.

#### Return Value

The method returns a mutable reference to Self, allowing for method chaining.

#### Usage example

```Rust
// Define a handler to process the response after executing a message
fn handle_response(res: AnyResult<AppResponse, cosmwasm_std::StdError>) {
    match res {
        Ok(response) => println!("Execution successful: {:?}", response),
        Err(err) => println!("Error executing the message: {:?}", err),
    }
}

// Assume `roboto` is an instance of the Roboto struct
let increment_msg = CounterHandleMsg::Increment {};
roboto.exec("counter", increment_msg, Some(handle_response));
```

In this example, we define a handle_response function to process the response from the contract execution. We then execute an Increment message on the counter contract using the `Roboto::exec` method and pass the `handle_response` function as an argument.

---

### Roboto::query

The query method is a part of the Roboto struct and is responsible for querying a deployed contract using a specified query message. It sends the query message to the specified contract and returns the result in a deserialized format.

#### Method Signature

```Rust
pub fn query<T, B>(
    &mut self,
    label: &str,
    msg: B
) -> Result<T, cosmwasm_std::StdError>
where
    T: DeserializeOwned,
    B: Serialize + Debug
```

#### Parameters
* **label**: &str: The label of the deployed contract. This label should match the one used when initializing the contract with `Roboto::init`.
* **msg**: B: The query message to be sent to the contract. The message should implement Serialize and Debug traits.

#### Return Value
The method returns a `Result` wrapping the deserialized response `T` on success, or a `cosmwasm_std::StdError` on failure.

#### Usage Example

```rust
// Assume `roboto` is an instance of the Roboto struct
let query_msg = CounterQueryMsg::GetCount {};
let result: Result<i32, cosmwasm_std::StdError> = roboto.query("counter", query_msg);

match result {
    Ok(count) => println!("The value of the counter is: {}", count),
    Err(err) => println!("Error querying the counter: {:?}", err),
}
```

In this example, we create a `CounterQueryMsg::GetCount` query message to query the counter contract. We use the `Roboto::query` method to send the query message and retrieve the deserialized result as an `i32`. We then print the value of the counter or the error if the query failed.

---

### Roboto::step
The step method is a part of the Roboto struct and is responsible for enabling custom processing steps with the Roboto instance. It takes a closure that accepts a mutable reference to the Roboto instance and allows users to perform additional operations on it.

#### Method Signature
```Rust
pub fn step(
    &mut self,
    processor: &mut dyn for<'r> FnMut(&'r mut Self),
) -> &mut Self
```
#### Parameters
processor: &mut dyn for<'r> FnMut(&'r mut Self): A mutable reference to a closure that accepts a mutable reference to the Roboto instance. The closure should define the additional operations or processing steps to perform on the Roboto instance.

#### Return Value
The method returns a mutable reference to Self, allowing for method chaining.

#### Usage Example
```rust
// Define a handler to process the response after executing a message
fn handle_response(res: AnyResult<AppResponse, cosmwasm_std::StdError>) {
    match res {
        Ok(response) => println!("Execution successful: {:?}", response),
        Err(err) => println!("Error executing the message: {:?}", err),
    }
}

// Assume `roboto` is an instance of the Roboto struct
// Execute Decrement message using the step method
roboto.step(&mut |roboto| {
    let decrement_msg = CounterHandleMsg::Decrement {};
    roboto.exec("counter", decrement_msg, Some(handle_response));
    roboto
});

// Query the counter contract
let query_msg = CounterQueryMsg::GetCount {};
let result: Result<i32, cosmwasm_std::StdError> = roboto.query("counter", query_msg);

match result {
    Ok(count) => println!("The final value of the counter is: {}", count),
    Err(err) => println!("Error querying the counter: {:?}", err),
}
```

In this example, we first define a `handle_response` function to process the response from the contract execution. Then, we execute a `Decrement` message using the `Roboto::step` method. The step method takes a closure that allows us to perform additional operations on the `Roboto` instance within the closure. In this case, we use the closure to call the exec method with the `Decrement` message and the `handle_response` function. Finally, we query the counter contract to get the final value of the counter using the `Roboto::query` method and print the result.

---