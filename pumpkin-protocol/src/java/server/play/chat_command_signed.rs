use crate::ServerPacket;
use crate::codec::var_int::VarInt;
use crate::ser::{NetworkReadExt, ReadingError};
use pumpkin_data::packet::serverbound::PLAY_CHAT_COMMAND_SIGNED;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;
use std::io::Read;

#[java_packet(PLAY_CHAT_COMMAND_SIGNED)]
pub struct SChatCommandSigned {
    pub command: Box<str>,
    pub timestamp: i64,
    pub salt: i64,
    pub argument_signatures: Vec<ArgumentSignaturesEntry>,
    pub message_count: VarInt,
    pub acknowledged: Box<[u8]>, // Bitset fixed 20 bits
    pub checksum: u8,            // 1.21.5 "fingerprint" checksum
}

#[derive(Debug)]
pub struct ArgumentSignaturesEntry {
    pub name: Box<str>,
    pub signature: Box<[u8]>,
}

impl ServerPacket for SChatCommandSigned {
    fn read(mut read: impl Read, _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            command: read.get_str()?,
            timestamp: read.get_i64_be()?,
            salt: read.get_i64_be()?,
            argument_signatures: read.get_list_bounded(
                |read| {
                    Ok(ArgumentSignaturesEntry {
                        name: read.get_str_bounded(16)?,
                        signature: read.read_boxed_slice(256)?,
                    })
                },
                8,
            )?,
            message_count: read.get_var_int()?,
            acknowledged: read.get_fixed_bitset(20)?,
            checksum: read.get_u8()?,
        })
    }
}
