/*
Macro for creating packet reader writer structs and formats

Example usage:

define_packet! {
    ExamplePacket (0x01) {


    }
}

 */
use std::collections::HashMap;
use std::io::{Read, Write};

use crate::io::{Readable, VarInt, Writeable};

#[macro_export]
macro_rules! define_packets {
    (

        $enum_name:ident {
            $(
                $name:ident ($id:literal) {
                    $(
                        $field:ident: $type:ty
                    )*
                }
            )*
        }
    ) => {

        #[derive(Debug, Clone, PartialEq)]
        enum $enum_name {
            $(
                $name($name),
            )*
        }

        impl $enum_name {
             pub fn id(&self) -> u32 {
                match self {
                    $(
                        $enum_name::$name(_) => $id,
                    )*
                }
            }
        }

        impl $crate::Readable for $enum_name {

            fn read<T: std::io::Read>(buf: &mut T) -> anyhow::Result<Self> where Self: Sized {
                let packet_id = $crate::io::VarInt::read(buf)?.0;
                match packet_id {
                    $(
                        id if id == $id => Ok($enum_name::$name($name::read(buf)?)),
                    )*
                    _ => Err(anyhow::anyhow!("Unknown packet ID of packet: {}", packet_id)),
                }
            }
        }

        $(

            #[derive(Debug, Clone, PartialEq)]
            pub struct $name {
                $(
                    pub $field: $crate::rust_type!($type),
                )*
            }

            impl $crate::packet::VariantOf<$enum_name> for $name{
                fn discriminant_id() -> u32 { $id }

                #[allow(unreachable_patterns)]
                fn destructure(e: $enum_name) -> Option<Self> {
                    match e {
                        $enum_name::$name(p) => Some(p),
                        _ => None,
                    }
                }
            }

            impl From<$name> for $enum_name {
                fn from(packet: $name) -> Self {
                    $enum_name::$name(packet)
                }
            }

            #[allow(unused_imports, unused_variables)]
            impl $crate::Readable for $name {
                fn read<T: std::io::Read>(buf: &mut T) -> anyhow::Result<Self> where Self: Sized {
                    use anyhow::Context as _;
                    $(
                        let $field = <$type>::read(buf)
                            .context(concat!("Could not read field `", stringify!($field), "` of packet `", stringify!($packet), "`"))?
                            .into();
                    )*

                    Ok(Self {
                        $(
                            $field,
                        )*
                    })
                }
            }

          #[allow(unused_variables)]
            impl $crate::Writeable for $name {
                fn write<T: std::io::Write>(&mut self, buf: &mut T) -> anyhow::Result<()> {
                    $crate::io::VarInt($id as u64).write(buf)?;
                    $(
                        $crate::writeable_type!($type, &mut self.$field).write(buf)?;
                    )*
                    Ok(())
                }
            }
        )*
    };
}


pub trait VariantOf<Enum> {
    fn discriminant_id() -> u32;

    fn destructure(e: Enum) -> Option<Self>
        where
            Self: Sized;
}

#[macro_export]
macro_rules! rust_type {
    (ByteArray) => {
        Vec<u8>
    };
    (String) => {
        & str
    };
    ($typ:ty) => {
        $typ
    };
}

#[macro_export]
macro_rules! writeable_type {
    (VarInt, $e:expr) => {
        VarInt(*$e as u64)
    };
    (Vec<$inner:ident>, $e:expr) => {
        Vec::from($e.as_slice())
    };
    ($typ:ty, $e:expr) => {
        $e
    };
}

define_packets! {
    Packets {
        TestPacket (0x05) {
            test: VarInt
        }

        ExamplePacket (0x06) {
            test: u8
        }
    }
}
