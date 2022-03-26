pub use io::{Readable, Writeable};

pub mod packet;
pub mod io;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::io::Cursor;

    use crate::{define_packets, Readable, Writeable};
    use crate::io::VarInt;
    use crate::packet::VariantOf;

    #[test]
    fn it_works() {
        define_packets! {
            Packets {
                TestPacket (0x05) { test: VarInt, a: u8 }

                ExamplePacket (0x06) {
                    test: u8
                }
            }
        }
        let mut t = TestPacket {
            test: VarInt(2),
            a: 1,
        };
        let mut out: Vec<u8> = Vec::new();
        t.write(&mut out);
        println!("{:?}", out);
        let mut s = Cursor::new(out);
        let a = Packets::read(&mut s);
        println!("{:?}", a);
    }
}



