use std::io::{Read, Write};

use crate::io::{Readable, VarInt, Writable};

#[macro_export]
macro_rules! rw_type {
    (VarInt, $e:expr) => {VarInt(*$e as u64)};
    (Vec<$inner:ident>, $e:expr) => {Vec::from($e.as_slice())};
    ($typ:ty, $e:expr) => {$e};
}

#[macro_export]
macro_rules! impl_packet_mode {
    ([read,write], $ID:literal, $Name:ident, $($Field:ident, $Type:ty)*) => {
          $crate::impl_packet_mode!([read], $ID, $Name, $($Field, $Type)*);
          $crate::impl_packet_mode!([write], $ID, $Name, $($Field, $Type)*);
    };
    ([read], $ID:literal, $Name:ident, $($Field:ident, $Type:ty)*) => {
        #[allow(unused_imports, unused_variables)]
        impl $crate::io::Readable for $Name {
            fn read<_ReadX: std::io::Read>(i: &mut _ReadX) -> anyhow::Result<Self> where Self: Sized {
                use anyhow::Context;
                $(
                    let $Field = <$Type>::read(i)
                      .context(concat!("failed to read field `", stringify!($Field), "` of packet `", stringify!($Name), "`"))?
                      .into();
                )*
                Ok(Self { $($Field,)* })
            }
        }
    };
    ([write], $ID:literal, $Name:ident, $($Field:ident, $Type:ty)*) => {
        #[allow(unused_imports, unused_variables)]
        impl $crate::io::Writable for $Name {
            fn write<_ReadX: std::io::Write>(&mut self, o: &mut _ReadX) -> anyhow::Result<()> {
                $crate::io::VarInt($ID as u64).write(o)?;
                $($crate::rw_type!($Type, &mut self.$Field).write(o)?;)*
                Ok(())
            }
        }
    };
}

#[macro_export]
macro_rules! impl_group_mode {
    ([read, write],$Group:ident, $($ID:literal, $Name:ident)*) => {
        $crate::impl_group_mode!([read], $Group, $($ID, $Name)*);
    };
    ([read],$Group:ident, $($ID:literal, $Name:ident)*) => {
        impl $crate::io::Readable for $Group {
            fn read<_ReadX: std::io::Read>(i: &mut _ReadX) -> anyhow::Result<Self> {
                let p_id = $crate::io::VarInt::read(i)?.0;
                match p_id {
                    $(id if id == $ID => Ok($Group::$Name($Name::read(i)?)),)*
                    _ => Err(anyhow::anyhow!("unknown packet id ({})", p_id)),
                }
            }
        }

        $(
            impl From<$Name> for $Group { fn from(p: $Name) -> Self { $Group::$Name(p) }}

            impl $crate::packets::PacketVariant<$Group> for $Name {
                fn id() -> $crate::io::VarInt { $crate::io::VarInt($ID as u64) }
                fn destructure(e: $Group) -> Option<Self> where Self: Sized {
                    match e {
                        $Group::$Name(p) => Some(p),
                        _ => None,
                    }
                }
            }
        )*
    };
    ([write],$Group:ident, $($ID:literal, $Name:ident)*) => {};
}

#[macro_export]
macro_rules! impl_struct_mode {
    ([read,write], $StructName:ident, $($StructField:ident, $StructFieldType:ty)*) => {
        $crate::impl_struct_mode!([read], $StructName, $($StructField, $StructFieldType)*);
        $crate::impl_struct_mode!([write], $StructName, $($StructField, $StructFieldType)*);
    };
    ([read], $StructName:ident, $($StructField:ident, $StructFieldType:ty)*) => {
        impl $crate::io::Readable for $StructName {
            fn read<_ReadX: std::io::Read>(i: &mut _ReadX) -> anyhow::Result<Self> where Self: Sized {
                use anyhow::Context;
                $(
                    let $StructField = <$StructFieldType>::read(i)
                        .context(concat!(
                                "failed to read field `",
                                stringify!($StructField),
                                "` of struct `",
                                stringify!($StructName), "`")
                    )?.into();
                )*
                Ok(Self { $($StructField,)* })
            }
        }
    };
    ([write], $StructName:ident, $($StructField:ident, $StructFieldType:ty)*) => {
        #[allow(unused_imports, unused_variables)]
        impl $crate::io::Writable for $StructName {
            fn write<_ReadX: std::io::Write>(&mut self, o: &mut _ReadX) -> anyhow::Result<()> {
                $($crate::rw_type!($StructFieldType, &mut self.$StructField).write(o)?;)*
                Ok(())
            }
        }
    };
}

#[macro_export]
macro_rules! packet_struct {
    (
        struct $StructName:ident $StructMode:tt {
            $($StructField:ident: $StructFieldType:ty),*
        }
    ) => {
        #[derive(Debug, Clone, PartialEq)]
        struct $StructName {
            $($StructField: $StructFieldType),*
        }

        $crate::impl_struct_mode!($StructMode, $StructName, $($StructField, $StructFieldType)*);
    };
}



#[macro_export]
macro_rules! impl_enum_mode {
    ([read,write], $EnumName:ident, $EnumType:ty, $($EnumField:ident, $EnumValue:literal)*) => {
        $crate::impl_enum_mode!([read], $EnumName, $EnumType, $($EnumField, $EnumValue)*);
        $crate::impl_enum_mode!([write], $EnumName, $EnumType, $($EnumField, $EnumValue)*);
    };
    ([read], $EnumName:ident, $EnumType:ty, $($EnumField:ident, $EnumValue:literal)*) => {
        impl $crate::io::Readable for $EnumName {
            fn read<B: std::io::Read>(i: &mut B) -> anyhow::Result<Self> where Self: Sized {
                use anyhow::Context;
                let value = <$EnumType>::read(i)
                    .context(concat!("failed to read value for enum `", stringify!($EnumName), "`"))?;
                match value {
                    $(
                        v if v == $EnumValue => Ok($EnumName::$EnumField),
                    )*
                    _ => Err(anyhow::anyhow!("invalid enum value ({})", value)),
                }
            }
        }
    };
    ([write], $EnumName:ident, $EnumType:ty, $($EnumField:ident, $EnumValue:literal)*) => {
        impl $crate::io::Writable for $EnumName {
            fn write<B: std::io::Write>(&mut self, o: &mut B) -> anyhow::Result<()> {
                match self {
                    $(
                        $EnumName::$EnumField => <$EnumType>::write(&mut $EnumValue, o),
                    )*
                };
                Ok(())
            }
        }
    };
}

#[macro_export]
macro_rules! impl_packet_data {
    (
        enum $EnumName:ident $EnumMode:tt ($EnumType:ty) {
            $($EnumField:ident: $EnumValue:literal),*
        }
    ) => {
        #[derive(Debug, Clone, PartialEq)]
        enum $EnumName {
            $($EnumField),*
        }

        $crate::impl_enum_mode!($EnumMode, $EnumName, $EnumType, $($EnumField, $EnumValue),*);
    };
    (
        struct $StructName:ident $StructMode:tt {
            $($StructField:ident: $StructFieldType:ty),*
        }
    ) => {
        #[derive(Debug, Clone, PartialEq)]
        struct $StructName {
            $($StructField: $StructFieldType),*
        }

        $crate::impl_struct_mode!($StructMode, $StructName, $($StructField, $StructFieldType)*);
    };
}


#[macro_export]
macro_rules! packet_data {
    (
        $(
            $Keyword:ident $Name:ident $Mode:tt $( ($TypeT:ident) )? {
                $(
                    $StructField:ident: $($EnumValue:literal)?$($StructFieldType:ty)?
                ),*
            }
        )*
    ) => {
        $(
            $crate::impl_packet_data!(
                $Keyword $Name $Mode $(($TypeT))? {
                    $($StructField: $($StructFieldType)? $($EnumValue)? ),*
                }
            );
        )*
    };
}

#[macro_export]
macro_rules! packets {
    (
        $(
            $Group:ident $Mode:tt {
                 $($ID:literal: $Name:ident {
                        $($Field:ident: $Type:ty),*
                 })*
            }
        )*
    ) => {
        $(
            $(
                #[derive(Debug, Clone, PartialEq)]
                struct $Name {
                    $($Field: $Type),*
                }
                $crate::impl_packet_mode!($Mode, $ID, $Name, $($Field, $Type)*);
            )*

            #[derive(Debug, Clone, PartialEq)]
            #[allow(dead_code)]
            enum $Group {
                $(
                    $Name($Name),
                )*
            }

            $crate::impl_group_mode!($Mode, $Group, $($ID, $Name)*);
        )*
    };
}

pub trait PacketVariant<Enum> {
    fn id() -> VarInt;
    fn destructure(e: Enum) -> Option<Self> where Self: Sized;
}
