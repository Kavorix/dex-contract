use blake2b_ref::{Blake2b, Blake2bBuilder};
use ckb_std::{
    ckb_types::{bytes::Bytes, packed::Script, prelude::*},
    high_level::load_script,
};

use crate::error::Error;

pub fn get_blake2b() -> Blake2b {
    Blake2bBuilder::new(32)
        .personal(b"ckb-default-hash")
        .build()
}

pub fn blake2b_160(message: &[u8]) -> [u8; 20] {
    let mut blake2b = get_blake2b();
    blake2b.update(&message);
    let mut result = [0; 32];
    blake2b.finalize(&mut result);
    let mut hash = [0; 20];
    hash.copy_from_slice(&result[0..20]);
    hash
}

pub fn parse_array<const N: usize>(arr: &[u8]) -> Result<[u8; N], Error> {
    arr.try_into().map_err(|_| Error::Encoding)
}

const MIN_ARGS_SIZE: usize = 66;
#[derive(Debug, Clone)]
pub struct DexArgs {
    // the minimum length of serialized lock script is 49bytes
    pub owner_lock:     Script,
    pub setup:          u8,
    pub total_value:    u128,
    pub receiver_lock:  Option<Script>,
    pub unit_type_hash: Option<[u8; 20]>,
}

impl DexArgs {
    pub fn from_script() -> Result<Self, Error> {
        let data: Bytes = load_script()?.args().unpack();
        if data.len() < MIN_ARGS_SIZE {
            return Err(Error::LockArgsInvalid);
        }
        let owner_size = u32::from_le_bytes(parse_array::<4>(&data[0..4])?) as usize;
        let required_size = owner_size + 17;
        if data.len() < (required_size + 17) {
            return Err(Error::LockArgsInvalid);
        }

        let owner_lock = Script::from_slice(&data[..owner_size]).map_err(|_e| Error::Encoding)?;
        let setup = data[owner_size];
        let total_value =
            u128::from_be_bytes(parse_array::<16>(&data[owner_size + 1..required_size])?);

        let option = &data[required_size..];
        let mut receiver_lock = None;
        let mut unit_type_hash = None;
        if option.len() == 20 {
            unit_type_hash = Some(parse_array::<20>(option)?);
        } else if option.len() > required_size + 4 {
            let receiver_size = u32::from_le_bytes(parse_array::<4>(&option[0..4])?) as usize;
            if option.len() < receiver_size {
                return Err(Error::LockArgsInvalid);
            }
            receiver_lock =
                Some(Script::from_slice(&option[..receiver_size]).map_err(|_e| Error::Encoding)?);

            if option.len() == receiver_size + 20 {
                unit_type_hash = Some(parse_array::<20>(&option[option.len() - 20..])?)
            }
        }

        Ok(DexArgs {
            owner_lock,
            setup,
            total_value,
            receiver_lock,
            unit_type_hash,
        })
    }
}