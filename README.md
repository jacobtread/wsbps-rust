# WSBPS-rust

An implementation of my websocket binary packet in rust this only implements the serialization
and deserialization of packets leaving you open to using whichever websocket system you would like.


## Create Packets
```rust
use wsbps::packets;

packets! {
    BiPackets (<->) {
        APacket (0x01) {
            user: u8
        }
    }
    
    ServerPackets (->) {
        BPacket (0x00) {
            name: u8
        }
    }
    
    ClientPackets (<-) {
        CPacket (0x00) {
            test: u8,
            test2: u8
        }
    }
}


```