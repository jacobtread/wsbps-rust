pub use io::{Readable, Writable};

pub mod packets;
pub mod io;

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::*;
    use crate::io::VarInt;

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
                    b: VarInt
                }
                TestB (0x02) {}
            }
        }


        let mut p = TestA {
           b: VarInt(4294967295)
        };
        println!("{:?}", p);

        let mut o = Vec::new();
        match p.write(&mut o) {
            Err(_) => println!("Failed to encode"),
            Ok(_) => {
                println!("{:?}", o);
                let mut s = Cursor::new(o);
                match BiPackets::read(&mut s) {
                    Err(e) => println!("{:?}",e),
                    Ok(p) => {
                        match p {
                            BiPackets::TestA(p) => {
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


