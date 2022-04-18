use std::io;
use std::string::FromUtf8Error;
use thiserror::Error;
use crate::Readable;

trait ValueTrait: Readable + Sized {}

#[derive(Error, Debug)]
pub enum PacketError {
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error("failed to convert string bytes to utf-8 string {0:?}")]
    BadEncoding(#[from] FromUtf8Error),
    #[error("string length ({0}) was greater than max string length size ({1})")]
    InvalidStringLength(usize, usize),
    #[error("unexpected value. expected {0}")]
    UnexpectedValue(&'static str),
    #[error("var-{0} exceeded maximum length of {1} bytes")]
    VarOverflow(&'static str, usize),
    #[error("packet with unknown id of {0} received")]
    UnknownPacket(u32),
    #[error("unknown enum value")]
    UnknownEnumValue
}