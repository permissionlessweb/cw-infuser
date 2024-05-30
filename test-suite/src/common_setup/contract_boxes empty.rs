

pub fn contract_base_factory() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw_infuser::contract::execute,
        cw_infuser::contract::instantiate,
        cw_infuser::contract::query,
    );
    // .with_sudo(cw_infuser::contract::sudo);
    Box::new(contract)
}