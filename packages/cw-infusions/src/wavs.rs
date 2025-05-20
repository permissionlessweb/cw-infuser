#[cosmwasm_schema::cw_serde]
pub struct WavsBundle {
    pub infuser: String,
    pub nft_addr: String,
    pub infused_ids: Vec<String>,
}

#[cosmwasm_schema::cw_serde]
pub struct WavsRecordResponse {
    pub addr: String,
    pub count: u64,
}

#[cosmwasm_schema::cw_serde]
pub struct WavsMintCountResponse {
    pub to_mint: u64,
    pub remaining: u64,
}
