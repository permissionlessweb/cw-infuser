use blake3::Hash;
use cosmwasm_std::{HexBinary, StdError, StdResult};

pub fn valid_hash_string(hash_string: &String) -> StdResult<()> {
    let hex_res = HexBinary::from_hex(hash_string.as_str());
    if hex_res.is_err() {
        return Err(cosmwasm_std::StdError::invalid_hex(hash_string.to_string()));
    }

    let hex_binary = hex_res.unwrap();

    let decoded = hex_binary.to_array::<32>();

    if decoded.is_err() {
        return Err(cosmwasm_std::StdError::invalid_data_size(
            32,
            hex_binary.len(),
        ));
    }
    Ok(())
}

pub fn verify_merkle_root(merkle_root: &String) -> StdResult<()> {
    valid_hash_string(merkle_root)
}

pub fn string_to_hash(string: &String) -> StdResult<Hash> {
    let mut byte_slice = [0; 32];
    hex::decode_to_slice(string, &mut byte_slice)
        .map_err(|_| StdError::generic_err("Couldn't decode hash string".to_string()))?;
    Ok(Hash::from_bytes(byte_slice))
}
