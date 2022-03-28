# WSBPS-rust

An implementation of my websocket binary packet in rust this only implements the serialization and deserialization of packets leaving you open to using whichever websocket system you would like.

## Directions

Directions are used within packet macros to define which kind of traits should be implemented for each object. This is used to reduce the number of traits that are implemented when they don't need to be.

Directions are represented as arrows. The following are the meanings for each arrow

> Arrows are represented in code as <-, ->, and  <->
> the fancy arrow is only used in this documentation

### &larr; Left Arrow

`<-` The left arrow is used for data that is read-only, this will only implement the readable trait for this item.

### &rarr; Right Arrow

`->` The right arrow is used for data that is write-only, this will only implement the writable trait for this item

### &harr; Double Headed Arrow

`<->` The double-headed arrow is used for data the is both readable and writable this implements both the readable and writable traits

## Data Types

The following is a table of Data Types that can be transmitted through this packet system along with their respective types in other languages and some other custom data types.

### Standard Number Types

| Rust Type | Range                     | Javascript (wsbps-js) | Length (bytes) |
|:---------:|---------------------------|-----------------------|----------------|
|    i8     | -128 to 127               | number (i8)           | 1              |
|    i16    | -32768 to 32767           | number (i16)          | 2              |
|    i32    | -2147483648 to 2147483647 | number (i32)          | 4              |
|    u8     | 0 to 255                  | number (u8)           | 1              |
|    u16    | 0 to 65535                | number (u16)          | 2              |
|    u32    | 0 to 4294967295           | number (u32)          | 4              |
|    f32    | -3.4e+38 to 3.4e+38       | number (f32)          | 4              |
|    f64    | -1.7e+308 to +1.7e+308    | number (f64)          | 4              |

> All number types listed in the table above are encoded using Big-Endian

### Variable Length Numbers

#### VarInt

VarInts is a number that can range in size anywhere from 0 to 4294967295 and
can be sent as binary data ranging from the length of 1byte to the length of 4 bytes

> VarInts / VarLongs are serialized 7 bits at a time starting with the least significant
> bits the most significant bit (msb) in each output byte indicates if there is
> a continuation byte (msb = 1)

#### Example

| VarInt  | Binary                              | Byte Format      |
|---------|-------------------------------------|------------------|
| 1       | 00000001                            | 1                |
| 127     | 01111111                            | 127              |
| 128     | 10000000 00000001                   | 128, 1           |
| 255     | 11111111 00000001                   | 255, 1           |
| 300     | 10101100 00000010                   | 172, 2           |
| 16384   | 10000000 10000000 00000001          | 128, 128, 1      |
| 2097152 | 10000000 10000000 10000000 00000001 | 128, 128, 128, 1 |

As you can see this format is far more efficient for storing data of varying length
however the VarInt has the same maximum length as the u32 (Unsigned 32-bit integer)

#### VarLong

The VarInt data type can only shift up to 5 offsets which restricts it to only handling
u32 numbers. The VarLong on the other hand can shift up to 10 offsets meaning that it can
encode and handle all numbers between 0 and 18446744073709551615 (u64)

> VarLongs are encoded in the same way as VarInts just they are allowed a greater number 
> of shifts when being read these are seperated in order to reduce memory allocations for
> VarInts so that they don't need to be allocated as u64 unless necessary (VarLong)

### Boolean
Booleans are encoded as a singular byte 1 representing a true value and 0 representing a false value.

### String
Strings are encoded using a VarInt for the length of the string followed by a sequence of the UTF-8
encoded bytes with the length being equal to the length VarInt - 1

```
Length VarInt
Contents [u8; Length]
```

### Arrays 
Array data types use Vectors these are encoded in the same way that strings are with a VarInt for the
length of the array and then all the respective values for that array are encoded in sequence after
the VarInt 

```
Length VarInt
For Length {
    Item Any
}
```

You can represent these types in packet structs using the ``Vec<Type>`` a common implementation
of this would be a ByteArray which is represented as a ``Vec<u8>``

## Packet Groups

To create packets you use the packets macro. Inside the macro you must specify packet "Groups" these 
groups are used to handle the differences between client and server packet IDs. The syntax for defining 
a group is as follows:

```rust
use wsbps::*;
packets! {
    GroupName (Direction) {
        //... Packets
    }
}
```

> In this example you would replace the "GroupName" with the name of the packet group
> and "Direction" with a direction arrow for this packet group

This macro will then generate an enum with the provided Group name which can be used to read packets if the 
read direction is implemented.

## Packets
You can then define packets with the following structure. This structure should be used inside group blocks

```rust
Name (ID) {
    example: u8
    // Normal struct field:type
}
```

> In this example you should replace "Name" with the name of this packet. The name you provide is also the 
> name that the generated struct will have. "ID" should be replaced with a unique identifier for this packet IDs
> are encoded using VarInts, so they can range anywhere from 0x00000000 - 0xffffffff (0 - 4294967295).

The following is an example of both packet groups and packet implementations put together

```rust
use wsbps::*;

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

## Structs & Enums

If you want to use custom structs or enums within your packets there is two options.

### Option 1

packet_data macro

You can easily create enum and structs for use within packets using the packet_data macro. 
This macro will automatically generate the required read and write traits for the enums / structs
you provide

```rust
use wsbps::*;

packet_data! {
    enum Test (<->) (VarInt) {
        X: 1,
        B: 999
    }
    
    struct TestStruct (->) {
        Name: String
    }
}
```

> The first set of brackets contains the "Direction" for this enum /struct type which tells it 
> which traits it needs to implement and the second set of brackets on enums contains the data
> type for this enum in this case the VarInt data type is used. Any integer data type is acceptable

### Option 2
If your data requires a custom encoding or is too complex to describe within a struct or enum you can 
manually implement the Readable and Writable traits from the io module

```rust
impl Writable for SomeType {
    fn write<B: Write>(&mut self, o: &mut B) -> Result<()> {
       // Your writing logic
        Ok(())
    }
}

impl Readable for SomeType {
    fn read<B: Read>(i: &mut B) -> Result<Self> where Self: Sized {
        // Your reading logic
    }
}
```