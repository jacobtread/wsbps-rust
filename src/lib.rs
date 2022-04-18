pub mod packets;
pub mod io;
pub mod error;

pub use io::*;
pub use error::*;

#[cfg(test)]
mod tests {
    use std::io::{Cursor};

    use crate::{Writable, Readable, packet_data, packets, VarInt};

    #[test]
    fn it_works() {
        packet_data! {
            enum Test (<->) (VarInt) {
                X: 1,
                B: 999
            }

            struct TestStruct (->) {
                Name: String
            }
        }


        packets! {
            BiPackets (<->) {
                TestA (0x01) {
                    b: VarInt,
                    a: Vec<u8>,
                }
                TestB (0x02) {}
            }
        }


        let mut p = BiPackets::TestA {
            b: VarInt(4294967295),
            a: vec![1,2,5]
        };
        println!("{:?}", p);


        let mut o = Vec::new();
        (p).write(&mut o);
        match p.write(&mut o) {
            Err(_) => println!("Failed to encode"),
            Ok(_) => {
                println!("{:?}", o);
                let mut s = Cursor::new(o);
                match BiPackets::read(&mut s) {
                    Err(e) => println!("{:?}",e),
                    Ok(p) => {
                        println!("{:?}",p);
                        match p {
                            BiPackets::TestA {b, a} => {
                                print!("{:?} {:?}", b, a)
                            }
                            _ => {}
                        }
                    }
                };
            }
        };
    }
}


