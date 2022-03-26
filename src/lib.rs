pub mod packets;
pub mod io;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::io::Cursor;

    use crate::io::RW;
    use crate::io::VarInt;
    use crate::{packet_structs, packets};
    use crate::packets::{ PacketReader};

    #[test]
    fn it_works() {
        packet_structs! {
            TestStruct {
                users: HashMap<u8, String>
            }
        }

        packets! {
            name: Packets; // This is the name of the generated packets enum

            // TestPacket is the name of the struct that will be generated
            // 0x01 is the unique ID for this packet which is used to identify it
            TestPacket 0x01 {
                field: VarInt, // VarInts require
                arr: Vec<u8>,
                test: TestStruct // Usage of a packet struct defined by the packet_structs! macro
            }

            Test2Packet 0x02 {}
        }

        let mut m: HashMap<u8, String> = HashMap::new();
        m.insert(25, String::from("Hello world"));

        let mut p = TestPacket {
            field: VarInt(12),
            arr: vec![0, 2],
            test: TestStruct {
                users: m
            }
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


