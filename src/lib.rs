pub mod packets;
pub mod io;

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use crate::{packet_data, packets};
    use crate::io::{Readable, Writable};
    #[test]
    fn it_works() {

        packet_data! {
            enum Test [read,write] (u8) {
                X = 1
            }
        }

        packets! {
            BiPackets [read,write] {
                0x02: TestA {
                    user: u8,
                    test: TestStruct
                }
            }
        }


        let mut p = TestA {
            user: 12,
            test: TestStruct {
                a: 8,
                b: 7
            }
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


