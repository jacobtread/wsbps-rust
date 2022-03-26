# WSBPS-rust

An implementation of my websocket binary packet in rust this only implements the serialization
and deserialization of packets leaving you open to using whichever websocket system you would like.


## Create Packets
```rust
use wsbps::{packets, io::VarInt, };
use wsbps::io::VarInt;


// Packet generator macro
packets! {
    name: Packets; // This is the name of the generated packets enum

    // TestPacket is the name of the struct that will be generated
    // 0x01 is the unique ID for this packet which is used to identify it
    TestPacket 0x01 {
        field: VarInt, // VarInt field
        arr: Vec<u8>, // Vector of u8 (This is decoded as a byte array in JS)
    }
    Test2Packet 0x02 {}
}

```