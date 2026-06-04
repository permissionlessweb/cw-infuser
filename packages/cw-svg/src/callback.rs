#[cosmwasm_schema::cw_serde]
pub struct SvgMintCallbackAction {
    /// contract addr of cw721-svg (self-contained minter).
    ///  MUST in same token a given shitstrap instance shits out.
    pub c: String,
    pub msg: MintMsg,
}

#[cosmwasm_schema::cw_serde]
pub struct MintMsg {
    /// amount of tokens to mint. Must send atleast `t` funds, where t >= amnt * mint-fee
    pub amnt: u64,
    /// Merkle proof hashes for whitelist verification (bypasses mint fees)
    pub proof_hashes: Vec<String>,
    /// Per-address mint allocation encoded in the merkle leaf.
    /// When set, the leaf is hash(sender || allocation) and this value
    /// caps how many tokens the address can mint via whitelist.
    pub alloc: u32,
}
 