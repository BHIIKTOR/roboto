use cosmwasm_std::{
    testing::{mock_info, MockApi, MockQuerier, MockStorage},
    Binary, ContractResult, CosmosMsg, Empty, Event, MessageInfo, QueryRequest, Response,
};

use std::borrow::BorrowMut;

use crate::{env::RobotoEnv, router::Router};

#[derive(Default, Clone, Debug)]
pub struct AppResponse {
    pub responses: Vec<Response>,
    pub events: Vec<Event>,
    pub data: Option<Binary>,
}

impl AppResponse {
    pub fn new() -> Self {
        Self {
            responses: vec![],
            events: vec![],
            data: None,
        }
    }
}

pub struct App {
    pub storage: MockStorage,
    pub api: MockApi,
    pub querier: MockQuerier,

    pub env: RobotoEnv,
    pub info: MessageInfo,

    pub router: Router,
}

impl App {
    pub fn new() -> Self {
        // TODO: Make env, info and deps customizable
        Self {
            storage: MockStorage::default(),
            api: MockApi::default(),
            querier: MockQuerier::default(),

            env: RobotoEnv::new(),
            info: mock_info("sender", &vec![]),

            router: Router::new(),
        }
    }
}

impl App {
    fn handle_response(
        &mut self,
        app_res: &mut Result<AppResponse, anyhow::Error>,
    ) -> Vec<Result<AppResponse, anyhow::Error>> {
        let mut out = vec![];
        let data = app_res.as_mut().unwrap();
        out.push(Ok(data.to_owned()));
        while let Some(mut res) = data.responses.pop() {
            while let Some(msg) = res.messages.pop() {
                out.push(self.router.handle_execute(
                    &self.api,
                    &mut self.storage.borrow_mut(),
                    &self.querier,
                    &mut self.env,
                    &self.info,
                    msg.msg,
                ))
            }
        }
        out
    }

    pub fn execute(&mut self, msg: CosmosMsg) -> Vec<Result<AppResponse, anyhow::Error>> {
        let mut res = self.router.handle_execute(
            &self.api,
            &mut self.storage.borrow_mut(),
            &self.querier,
            &mut self.env,
            &self.info,
            msg,
        );

        self.handle_response(&mut res)
    }

    pub fn query(&mut self, query: QueryRequest<Empty>) -> ContractResult<cosmwasm_std::Binary> {
        self.router
            .handle_query(
                &self.api,
                &mut self.storage.borrow_mut(),
                &self.querier,
                &mut self.env,
                &query,
            )
            .unwrap()
    }
}
