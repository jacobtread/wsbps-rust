/// ## Writable Type Macro
/// A macro used internally to convert struct and packet field types
/// into writable types
#[macro_export]
macro_rules! writable_type {
    // Match VarInts
    (VarInt, $e:expr) => { *$e };
    // Match VarLongs
    (VarLong, $e:expr) => { *$e } ;
    // Match vectors
    (Vec<$inner:ident>, $e:expr) => { *$e };
    // Match all other types
    ($typ:ty, $e:expr) => { $e };
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
        impl $crate::Readable for $Name {
            fn read<_ReadX: std::io::Read>(i: &mut _ReadX) -> $crate::ReadResult<Self> where Self: Sized {
                // Provide all the fields to a new struct of self
                Ok(Self {
                    // Read all the fields for the struct
                    $(
                        $Field: <$FieldType>::read(i)?.into(),
                    )*
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
        impl $crate::Writable for $Name {
            fn write<_ReadX: std::io::Write>(&mut self, o: &mut _ReadX) -> $crate::WriteResult {
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


#[macro_export]
macro_rules! discriminant_to_literal {
    (String, $discriminant:expr) => {
        &*$discriminant
    };
    ($discriminant_type:ty, $discriminant:expr) => {
        $discriminant.into()
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
        impl $crate::Readable for $Name {
            fn read<B: std::io::Read>(i: &mut B) -> $crate::ReadResult<Self> where Self: Sized {
                // Use the io::Readable for the type parameter to encode it
                let value = $crate::discriminant_to_literal!($Type, <$Type>::read(i)?);
                match value { // Match the value that was read
                    // Match for all the enum fields. Matches will return the enum field
                    $($Value => Ok($Name::$Field),)*
                    // Errors are used if none match
                    _ => Err($crate::PacketError::UnknownEnumValue),
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
        impl $crate::Writable for $Name {
            fn write<B: std::io::Write>(&mut self, o: &mut B) -> $crate::WriteResult {
                match self { // Match self
                    // For each of the fields map them to a write call for the type
                    // and the value for that type
                    $($Name::$Field => <$Type>::from($Value).write(o)?,)*
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
        pub enum $Name {
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
        pub struct $Name {
            $(pub $Field: $FieldType),*
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

/// # Impl Group Mode Macro
/// This macro implements the specific read/write mode for the group. This also implements the traits
/// for each specific mode.
#[macro_export]
macro_rules! impl_group_mode {
    (
        (<-) $Group:ident {
            $(
                $Name:ident, $ID:literal {
                    $($Field:ident, $Type:ty),*
                }
            );*
        }
    ) => {
        // Implement the io::Readable trait so this enum can be read this must be
        // implemented here so we can read the packet ID first then read the
        // respective packet
        impl $crate::Readable for $Group {
            fn read<_ReadX: std::io::Read>(i: &mut _ReadX) -> $crate::ReadResult<Self> {
                let p_id = $crate::VarInt::read(i)?.0;
                match p_id {
                    // Match for all the packet IDS and read the packet struct and return
                    // the enum value with the struct as the value
                    $(
                        $ID => Ok($Group::$Name {
                            $(
                                $Field: <$Type>::read(i)?.into(),
                            )*
                        }),
                    )*
                    _ => Err($crate::PacketError::UnknownPacket(p_id))
                }
            }
        }
    };
    (
        (->) $Group:ident {
            $(
                $Name:ident, $ID:literal {
                    $($Field:ident, $Type:ty),*
                }
            );*
        }
    ) => {
        impl $crate::Writable for $Group {
            fn write<_WriteX: std::io::Write>(&mut self, o: &mut _WriteX) -> $crate::WriteResult {
                match self {
                    $(
                        $Group::$Name {
                            $($Field),*
                        } => {
                            $crate::VarInt($ID as u32).write(o)?;
                            $($crate::writable_type!($Type, $Field).write(o)?;)*
                        },
                    )*
                }
                Ok(())
            }
        }
    };
    (
        (<->) $Group:ident {
            $(
                $Name:ident, $ID:literal {
                    $($Field:ident, $Type:ty),*
                }
            );*
        }
    ) => {
        $crate::impl_group_mode!(
            (<-) $Group {
                $(
                    $Name, $ID {
                        $($Field, $Type),*
                    }
                );*
            }
        );
        $crate::impl_group_mode!(
           (->) $Group {
                $(
                    $Name, $ID {
                        $($Field, $Type),*
                    }
                );*
            }
        );
    };
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
            // Implement the group enum
            #[derive(Debug, Clone, PartialEq)]
            #[allow(dead_code)]
            pub enum $Group {
                $(
                    $Name {
                        $(
                            $Field: $Type,
                        )*
                    }
                ),*
            }

            // Implement the specified group mode
            $crate::impl_group_mode!(
                $Mode $Group {
                    $(
                        $Name, $ID {
                            $($Field, $Type),*
                        }
                    );*
                }
            );

            // Implement packet variant ID for each packet enum value
            impl $Group {
                // Packet id function to allow retrieval of the packet ID on the packet
                fn id(&self) -> $crate::VarInt {
                    $crate::VarInt(match self {
                        $($Group::$Name { .. } => $ID as u32,)*
                    })
                }
            }
        )*
    };
}