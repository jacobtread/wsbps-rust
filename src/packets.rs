use crate::io::VarInt;

/// ## Writable Type Macro
/// A macro used internally to convert struct and packet field types
/// into writable types
#[macro_export]
macro_rules! writable_type {
    // Match VarInts
    (VarInt, $e:expr) => {VarInt(*$e as u32)};
    // Match VarLongs
    (VarLong, $e:expr) => {VarInt(*$e as u64)};
    // Match vectors
    (Vec<$inner:ident>, $e:expr) => {Vec::from($e.as_slice())};
    // Match all other types
    ($typ:ty, $e:expr) => {$e};
}

/// ## Impl Struct Mode Macro
/// This is the underlying backing macro which is used by the impl_packet_data macro which is used by the
/// packet_data macro to generic the specific struct trait implementations for the desired packet mode
#[macro_export]
macro_rules! impl_struct_mode {
    (
        (<-) $Name:ident {
            $($Field:ident, $FieldType:ty),*
        }
    ) => {
        // Implement the io::Readable trait so this struct can be read
        impl $crate::io::Readable for $Name {
            fn read<_ReadX: std::io::Read>(i: &mut _ReadX) -> anyhow::Result<Self> where Self: Sized {
                use anyhow::Context; // Use context from anyhow so .context can be used
                $(
                    // Create a variable for each field and read with the field type
                    let $Field = <$FieldType>::read(i)
                        // Add additional context to error messages
                        .context(concat!(
                                "failed to read field `",
                                stringify!($Field),
                                "` of struct `",
                                stringify!($Name), "`"))?.into();
                )*
                // Provide all the fields to a new struct of self
                Ok(Self {
                    $($Field,)*
                })
            }
        }
    };
    (
        (->) $Name:ident {
            $($Field:ident, $FieldType:ty),*
        }
    ) => {
        // Implement the io::Writable trait so the enum can be written
        #[allow(unused_imports, unused_variables)]
        impl $crate::io::Writable for $Name {
            fn write<_ReadX: std::io::Write>(&mut self, o: &mut _ReadX) -> anyhow::Result<()> {
                // Create a write call for all of the fields using their type
                $($crate::writable_type!($FieldType, &mut self.$Field).write(o)?;)*
                Ok(())
            }
        }
    };
   (
       (<->) $Name:ident {
           $($Field:ident, $FieldType:ty),*
       }
   ) => {
        // Pass the parameters onto the read implementation
        $crate::impl_struct_mode!(
            (<-) $Name {
                $($Field, $FieldType),*
            }
        );
        // Pass the parameters onto the write implementation
        $crate::impl_struct_mode!(
            (->) $Name {
                $($Field, $FieldType),*
            }
        );
    };
}

/// ## Impl Enum Mode Macro
/// This is the underlying backing macro which is used by the impl_packet_data macro which is used by the
/// packet_data macro to generate the specific enum trait implementations for the desired packet mode
#[macro_export]
macro_rules! impl_enum_mode {
    (
        (<-) $Name:ident $Type:ty {
            $($Field:ident, $Value:expr),*
        }
    ) => {
        // Implement the io::Readable trait so this enum can be read
        impl $crate::io::Readable for $Name {
            fn read<B: std::io::Read>(i: &mut B) -> anyhow::Result<Self> where Self: Sized {
                use anyhow::Context; // Use context from anyhow so .context can be used
                // Use the io::Readable for the type parameter to encode it
                let value = <$Type>::read(i)
                    // Add additional context to error messages
                    .context(concat!("failed to read value for enum `", stringify!($Name), "`"))?;
                match value { // Match the value that was read
                    // Match for all the enum fields. Matches will return the enum field
                    $(v if v == $Value => Ok($Name::$Field),)*
                    // Errors are used if none match
                    _ => Err(anyhow::anyhow!("invalid enum value ({})", value)),
                }
            }
        }
    };
    (
        (->) $Name:ident $Type:ty {
            $($Field:ident, $Value:expr),*
        }
    ) => {
        // Implement the io::Writable trait so the enum can be written
        impl $crate::io::Writable for $Name {
            fn write<B: std::io::Write>(&mut self, o: &mut B) -> anyhow::Result<()> {
                match self { // Match self
                    // For each of the fields map them to a write call for the type
                    // and the value for that type
                    $($Name::$Field => <$Type>::write(&mut $Value, o)?,)*
                };
                Ok(())
            }
        }
    };
    (
        (<->) $Name:ident $Type:ty {
            $($Field:ident, $Value:expr),*
        }
    ) => {
        // Pass the parameters onto the read implementation
        $crate::impl_enum_mode!(
            (<-) $Name $Type {
                $($Field, $Value),*
            }
        );
        // Pass the parameters onto the write implementation
        $crate::impl_enum_mode!(
            (->) $Name $Type {
                $($Field, $Value),*
            }
        );
    };
}

/// ## Impl Packet Data
/// This is the underlying backing macro for packet_data which handles which type should be
/// implemented and for which mode (enum / struct) this is used to speed up parsing and reduce
/// the complexity of the packet_data macro
#[macro_export]
macro_rules! impl_packet_data {
    // Matching enums
    (
        enum $Name:ident $Mode:tt $Type:ty {
            $($Field:ident, $Value:expr),*
        }
    ) => {
        // Create the backing enum
        #[derive(Debug, Clone, PartialEq)]
        #[allow(dead_code)]
        enum $Name {
            $($Field),*
        }

        // Implement the traits for the provided mode
        $crate::impl_enum_mode!(
            $Mode $Name $Type {
                $($Field, $Value),*
            }
        );
    };
    // Matching structs
    (
        struct $Name:ident $Mode:tt {
            $($Field:ident, $FieldType:ty),*
        }
    ) => {
        // Create the backing struct
        #[derive(Debug, Clone, PartialEq)]
        struct $Name {
            $($Field: $FieldType),*
        }

        // Implement the traits for the provided mode
        $crate::impl_struct_mode!(
            $Mode $Name {
                $($Field, $FieldType),*
            }
        );
    };
}

/// ## Packet Data
/// This macro is used to implement read and write traits for enums so they can be used within
/// packets as packet fields. This is a block and you should use it to implement all of your
/// structs and enums at once.
///
/// ## Directions
/// (<->) Bi-Direction: This implements both readers and writers for this data. This should
/// be used in structs and enums that are shared between readable and writable packets.
///
/// (->) Write-Only: This implements only the writers for this data. This should be used if
/// the struct/enum is only going to be sent and not received.
///
/// (<-) Read-Only: This implements only the readers for this data. This should be used if
/// the struct/enum is only going to be received and not send.
///
/// ## Example
///
/// ```
/// use wsbps::packet_data;
/// packet_data! {
///     struct ExampleBiStruct (<->) {
///         Field: u8,
///         Name: String
///     }
///
///     enum TestWriteEnum (->) (u8) {
///         A: 1,
///         B: 2
///     }
/// }
/// ```
///
#[macro_export]
macro_rules! packet_data {
    (
        $(
            $Keyword:ident $Name:ident $Mode:tt $(($Type:ty))? {
                $(
                    $Field:ident:$($EnumValue:literal)?$($FieldType:ty)?
                ),* $(,)?
            }
        )*
    ) => {
        $(
            // Implement the underlying types for each matched value
            $crate::impl_packet_data!(
                $Keyword $Name $Mode $($Type)? {
                    $($Field, $($EnumValue)? $($FieldType)?),*
                }
            );
        )*
    };
}


/// # Impl Packet Mode Macro
/// This is the underlying backing macro for the packets macro this implements the specific packet
/// mode for each individual packets
#[macro_export]
macro_rules! impl_packet_mode {
    (
        (<-) $Name:ident $ID:literal {
            $($Field:ident, $Type:ty),*
        }
    ) => {
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
    (
        (->) $Name:ident $ID:literal {
            $($Field:ident, $Type:ty),*
        }
    ) => {
        #[allow(unused_imports, unused_variables)]
        impl $crate::io::Writable for $Name {
            fn write<_ReadX: std::io::Write>(&mut self, o: &mut _ReadX) -> anyhow::Result<()> {
                $crate::io::VarInt($ID as u32).write(o)?;
                $($crate::writable_type!($Type, &mut self.$Field).write(o)?;)*
                Ok(())
            }
        }
    };
    (
        (<->) $Name:ident $ID:literal {
            $($Field:ident, $Type:ty),*
        }
    ) => {
        // Pass the parameters onto the read implementation
        $crate::impl_packet_mode!(
            (<-) $Name $ID {
                $($Field, $Type),*
            }
        );
        // Pass the parameters onto the write implementation
        $crate::impl_packet_mode!(
            (->) $Name $ID {
                $($Field, $Type),*
            }
        );
    };
}


/// # Impl Group Mode Macro
/// This macro implements the specific read/write mode for the group. This also implements the traits
/// for each specific mode.
#[macro_export]
macro_rules! impl_group_mode {
    (
        (<-) $Group:ident {
            $($Name:ident, $ID:literal),*
        }
    ) => {
        // Implement the io::Readable trait so this enum can be read this must be
        // implemented here so we can read the packet ID first then read the
        // respective packet
        impl $crate::io::Readable for $Group {
            fn read<_ReadX: std::io::Read>(i: &mut _ReadX) -> anyhow::Result<Self> {
                let p_id = $crate::io::VarInt::read(i)?.0;
                match p_id {
                    // Match for all the packet IDS and read the packet struct and return
                    // the enum value with the struct as the value
                    $(id if id == $ID => Ok($Group::$Name($Name::read(i)?)),)*
                    _ => Err(anyhow::anyhow!("unknown packet id ({})", p_id)),
                }
            }
        }

        $(
            // Implement conversion of packets from the group for each packet name
            // to allow conversion between packets and the enum representation.
            impl From<$Name> for $Group { fn from(p: $Name) -> Self { $Group::$Name(p) }}

            // Implement packet variant for the packet name of this current group
            impl $crate::packets::PacketVariant<$Group> for $Name {
                // Packet id function to allow retrieval of the packet ID on the packet
                fn id() -> $crate::io::VarInt { $crate::io::VarInt($ID as u32) }
                // Implement destructure function
                fn destructure(e: $Group) -> Option<Self> where Self: Sized {
                    match e {
                        // Match the enum name and return some with that value
                        $Group::$Name(p) => Some(p),
                        _ => None,
                    }
                }
            }
        )*
    };
    (
        (<->) $Group:ident {
            $($Name:ident, $ID:literal),*
        }
    ) => {
        // Read write only needs to implement reading because writing is not
        // done from the group enum layer
        $crate::impl_group_mode!(
            (<-) $Group {
                $($Name, $ID),*
            }
        );
    };
    ((->)) => { /* Write implementations are matched but ignored */ };
}

/// # Packets Macro
/// This macro is used to define packet groups. It implements the structs for each packet along
/// with their readers and writers (if they require them) and an enum for the packet group to
/// read packets.
///
/// ## Directions
/// (<->) Bi-Direction: This implements both readers and writers for this data. This should
/// be used in structs and enums that are shared between readable and writable packets.
///
/// (->) Write-Only: This implements only the writers for this data. This should be used if
/// the struct/enum is only going to be sent and not received.
///
/// (<-) Read-Only: This implements only the readers for this data. This should be used if
/// the struct/enum is only going to be received and not send.
///
/// ## Example
/// ```
///
/// use wsbps::packets;
///
/// packets! {
///     BiPackets (<->) {
///         APacket (0x02) {
///             User: u8,
///             Name: String
///         }
///         BPacket (0x05) {
///             Name: String
///         }
///     }
///
///     ServerPackets (->) {
///         CPacket (0x02) {
///             User: u8,
///             Name: String
///         }
///         DPacket (0x05) {
///             Name: String
///         }
///     }
/// }
/// ```
#[macro_export]
macro_rules! packets {
    (
        $(
            $Group:ident $Mode:tt {
                 $(
                     $Name:ident ($ID:literal) {
                            $($Field:ident: $Type:ty),* $(,)?
                     }
                 )*
            }
        )*
    ) => {
        $(
            $(
                // Implement a struct for each packet
                #[derive(Debug, Clone, PartialEq)]
                struct $Name {
                    $($Field: $Type),*
                }

                // Implement the specified packet mode
                $crate::impl_packet_mode!(
                    $Mode $Name $ID {
                        $($Field, $Type),*
                    }
                );
            )*

            // Implement the group enum
            #[derive(Debug, Clone, PartialEq)]
            #[allow(dead_code)]
            enum $Group {
                $($Name($Name)),*
            }

            // Implement the specified group mode
            $crate::impl_group_mode!(
                $Mode $Group {
                    $($Name, $ID),*
                }
            );
        )*
    };
}


pub trait PacketVariant<Enum> {
    fn id() -> VarInt;
    fn destructure(e: Enum) -> Option<Self> where Self: Sized;
}
