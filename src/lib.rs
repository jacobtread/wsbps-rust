pub mod packets;
pub mod io;

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use crate::{packet_data, packets};
    use crate::io::{Readable, Writable};
    #[test]
    fn it_works() {
        struct a {
            a: u8,
            b: u8,
        }

        packet_data! {
            enum TestEnum [read,write] (u8) {
                A = 0,
                B = 151
            }
        }



        packets! {
            BiPackets [read,write] {
                0x02: TestA {
                    user: u8,
                    test: TestEnum
                }
            }
        }


        let mut p = TestA {
            user: 12,
            test: TestEnum::B
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


