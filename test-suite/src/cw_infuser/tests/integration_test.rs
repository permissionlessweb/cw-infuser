

pub fn contract_cw_infuser() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        cw_infuser::contract::execute,
        cw_infuser::contract::instantiate,
        cw_infuser::contract::query,
    );
    Box::new(contract)
}