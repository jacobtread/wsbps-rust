#[macro_export]
macro_rules! packets {
    (
        // The name of the packet enum
        name: $e_name:ident;

        $(
            $p_name:ident $p_id:literal {
                $(
                    $p_field:ident: $p_field_type:ty
                ),*
            }
        )*
    ) => {

        /// Creating structs for each of the packet's
        $(
            #[derive(Debug, Clone, PartialEq)]
            struct $p_name {
                $(
                    $p_field: $p_field_type
                ),*
            }

            #[allow(unused_imports, unused_variables)]
            impl $crate::io::RW for $p_name {
                fn write<B: std::io::Write>(&mut self, o: &mut B) -> anyhow::Result<()> {
                    $crate::io::VarInt($p_id).write(o)?;
                    $($crate::rw_type!($p_field_type, &mut self.$p_field).write(o)?;)*
                    Ok(())
                }

                fn read<B: std::io::Read>(i: &mut B) -> anyhow::Result<Self> where Self: Sized {
                    use anyhow::Context;
                    $(
                        let $p_field = <$p_field_type>::read(i)
                            .context(concat!("failed to read field `", stringify!($p_field), "` of packet `", stringify!($p_name), "`"))?
                            .into();
                    )*

                    Ok(Self {
                        $(
                            $p_field,
                        )*
                    })
                }
            }

            impl From<$p_name> for $e_name { fn from(p: $p_name) -> Self { $e_name::$p_name(p) }}

            impl $crate::packets::PacketVariant<$e_name> for $p_name {
                fn id() -> $crate::io::VarInt { $crate::io::VarInt($p_id) }
                fn destructure(e: $e_name) -> Option<Self> where Self: Sized {
                    match e {
                        $e_name::$p_name(p) => Some(p),
                        _ => None,
                    }
                }
            }
        )*

        /// Enum containing all the packet names and their implementations
        #[derive(Debug, Clone, PartialEq)]
        #[allow(dead_code)]
        enum $e_name {
            $(
                $p_name($p_name),
            )*
        }

        impl $crate::packets::PacketReader for $e_name {
            fn read<B: std::io::Read>(i: &mut B) -> anyhow::Result<Self> where Self: Sized {
                let p_id = $crate::io::VarInt::read(i)?.0;
                match p_id {
                    $(
                        id if id == $p_id => Ok($e_name::$p_name($p_name::read(i)?)),
                    )*
                    _ => Err(anyhow::anyhow!("unknown packet id ({})", p_id)),
                }
            }

        }
    };
}

/// ## Packet Structs Macro
/// Macro for defining structs that can be serialized for use inside
/// of packets as the field type.
///
/// ## Usage
///
/// ```
/// use wsbps::packet_structs;
///
/// packet_structs! {
///     TestStruct {
///         user: u8
///     }
/// }
/// ```
///
/// This will generate the following struct along with a RW implementation which
/// calls the RW implementation of all of the struct fields.
///
/// ```
/// #[derive(Debug, Clone, PartialEq)]
/// struct TestStruct {
///     user: u8,
/// }
/// ```
#[macro_export]
macro_rules! packet_structs {
    (
        $(
            $name:ident{
                $(
                    $field:ident: $field_type:ty
                ),*
            }
        )*
    ) => {
        $(
        #[derive(Debug, Clone, PartialEq)]
        struct $name {
            $(
                $field: $field_type
            ),*
        }

        #[allow(unused_imports, unused_variables)]
            impl $crate::io::RW for $name {
                fn write<B: std::io::Write>(&mut self, o: &mut B) -> anyhow::Result<()> {
                    $($crate::rw_type!($field_type, &mut self.$field).write(o)?;)*
                    Ok(())
                }

                fn read<B: std::io::Read>(i: &mut B) -> anyhow::Result<Self> where Self: Sized {
                    use anyhow::Context;
                    $(
                        let $field = <$field_type>::read(i)
                            .context(concat!("failed to read field `", stringify!($field), "` of struct `", stringify!($name), "`"))?
                            .into();
                    )*

                    Ok(Self {
                        $(
                            $field,
                        )*
                    })
                }
            }
        )*
    };
}

#[macro_export]
macro_rules! rw_type {
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

pub trait Packet: crate::io::RW {
    fn write<B: std::io::Write>(&mut self, o: &mut B) -> anyhow::Result<()>;
}

pub trait PacketVariant<Enum> {
    fn id() -> crate::io::VarInt;
    fn destructure(e: Enum) -> Option<Self> where Self: Sized;
}

pub trait PacketReader {
    fn read<B: std::io::Read>(i: &mut B) -> anyhow::Result<Self> where Self: Sized;
}