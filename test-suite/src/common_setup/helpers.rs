use cosmwasm_std::{Addr, Coin};
use cw4::Member;
use cw_multi_test::{App, AppBuilder};

pub fn mock_app_builder_init_funds(init_funds: &[Coin]) -> App {
    AppBuilder::new().build(|router, _, storage| {
        router
            .bank
            .init_balance(storage, &Addr::unchecked(OWNER), init_funds.to_vec())
            .unwrap();
    })
}