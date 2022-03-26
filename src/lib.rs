pub mod packets;
pub mod io;

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::io::RW;
    use crate::io::VarInt;
    use crate::packets;
    use crate::packets::{ PacketReader};

    #[test]
    fn it_works() {
        packets! {
            name: Packets; // This is the name of the generated packets enum

            // TestPacket is the name of the struct that will be generated
            // 0x01 is the unique ID for this packet which is used to identify it
            TestPacket 0x01 {
                field: VarInt, // VarInts require
                arr: Vec<u8>
            }

            Test2Packet 0x02 {}
        }

        let mut p = TestPacket {
            field: VarInt(12),
            arr: vec![0, 2],
        };
        println!("{:?}", p);

        let mut o = Vec::new();
        match p.write(&mut o) {
            Err(_) => println!("Failed to encode"),
            Ok(_) => {
                println!("{:?}", o);
                let mut s = Cursor::new(o);
                match Packets::read(&mut s) {
                    Err(_) => println!("Failed to decode"),
                    Ok(p) => {
                        match p {
                            Packets::TestPacket(p) => {
                                print!("{:?}", p)
                            }
                            _ => {}
                        }
                    }
                };
            }
        };
    }
}


