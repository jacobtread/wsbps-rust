pub mod packets;
pub mod io;
pub use io::{Readable, Writable};

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::{packet_data, packets};
    use crate::io::{Readable, Writable};

    #[test]
    fn it_works() {
        packet_data! {
            struct TestStruct (<->) {
                a: u8,
                b: u8,
                c: u16
            }

            enum Test (->) (u8) {
                X: 1
            }
        }

        packets! {
            BiPackets (<->) {
                TestA (0x01) {
                    user: u8,
                    test: TestStruct
                }
                TestB (0x02) {}
            }
        }


        let mut p = TestA {
            user: 12,
            test: TestStruct {
                a: 8,
                b: 12,
                c:400
            },
        };
        println!("{:?}", p);

        let mut o = Vec::new();
        match p.write(&mut o) {
            Err(_) => println!("Failed to encode"),
            Ok(_) => {
                println!("{:?}", o);
                let mut s = Cursor::new(o);
                match BiPackets::read(&mut s) {
                    Err(_) => println!("Failed to decode"),
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


