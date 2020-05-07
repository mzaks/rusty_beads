# rusty_beads
Implementation of Beads data serialisation format in Rust.

Beads data serialisation format is designed to encode and decode sequences of heterogeneous values in a very compact way.

The values can be of type:
- fixed size unsigned integer: `U8, U16, U32, U64, U128`
- fixed size signed integer: `I8, I16, I32, I64, I128`
- fixed size floating point numbers: `F16, F32, F64`
- variable size unsigned integer: `Vlq`
- variable size signed integer: `VlqZ`
- bool values: `true, false`
- absence of value: `None`
- utf8 encoded string: `Utf8`
- array of bytes: `Bytes`

## Building and encoding a Beads sequence

In order to build a beads sequence we need to create an instance of TypedBeadsBuilder:

```rust
let mut builder = TypedBeadsBuilder::new(
    &BeadTypeSet::new(&[BeadType::Utf8])
).ok().unwrap();
```

`BeadTypeSet` is an indicator which type of elements we plan to add to the sequence. A Beads sequence may contain 1 to 16 types. 
So as a creator you need to pick, which of the above mentioned types you want/need to add to the sequence.

In the example above, we define that we want to build up a sequence of strings.

`TypedBeadsBuilder` struct has a number of `push_XXX`, which let us push values into Beads sequence:
- push_none
- push_bool
- push_int
- push_unit
- push_double
- push_double_with_accuracy
- push_string
- push_bytes

Every push method returns a boolean which indicates if the value could be pushed to the sequence. 
This is important, because we limit the types, which can be added to the given builder instance.

So if we write:
```rust
let mut builder = TypedBeadsBuilder::new(
    &BeadTypeSet::new(&[BeadType::Utf8])
).ok().unwrap();
let success = builder.push_string("Maxim");
```

The value of `success` will be `true`.

```rust
let mut builder = TypedBeadsBuilder::new(
    &BeadTypeSet::new(&[BeadType::Utf8])
).ok().unwrap();
let success = builder.push_int("45");
```

The value of `success` will be `false`, because `builder` is configured to store strings only.

### Why do we limit the number of types?

Beads sequence stores the type information for every element which is pushed to it. However if we configure the sequence to have only one type, then the type is implicit and it occupies 0 bits.
If we configure the sequence to store 2 types, like for example:
```rust
let mut builder = TypedBeadsBuilder::new(
    &BeadTypeSet::new(&[BeadType::Utf8, BeadType::None])
).ok().unwrap();
```
Then the type information for 8 elements is stored in 1 byte, meaning that the overhead of storing the type of the element is just one bit.
If we configure the sequence to store 3 or 4 different type elements. The type information for 4 elements is stored in one byte.
Overhead for every element being 2 bits. And if the number of configured types is between 5 and 16 the overhead is 4 bits per element.

Other formats like CBOR, MessagePack etc... have a fix overhead of 1 byte for every elements. 
They do employ some tricks like combining the type information with a small value in order to represent both in one byte.
But those tricks are more limited than what you can do with Beads.

For example, in Beads we can store a sequence of boolean values as a bit set:

```rust
let mut builder = TypedBeadsBuilder::new(
    &BeadTypeSet::new(&[BeadType::TrueFlag, BeadType::FalseFlag])
).ok().unwrap();

builder.push_bool(true);
builder.push_bool(true);
builder.push_bool(false);
builder.push_bool(false);
builder.push_bool(true);
builder.push_bool(true);
builder.push_bool(false);
builder.push_bool(true);

let mut buffer: Vec<u8> = vec![];
builder.encode(&mut buffer);
println!("{:?}", buffer);
```

The `true` and `false` values are stored just as type information. The resulting `buffer` is just 2 bytes long:
`[8, 76]`

The first byte indicates the length of the sequence and the second stores the type information for the 8 values we added.

In the snippet above we can also see, how we can encode the Beads sequence. The `encode` method of the `builder` receives an instance of `io::Write` trait and pushes the bytes to it.

### But how do we know which types are in the Beads sequence?
If we use the `encode` method to create a buffer, it does not store the type information.
In this case reading and writing parties need to agree on the types explicitly.
There is another way though:
```rust
let mut buffer: Vec<u8> = vec![];
builder.encode_with_types(&mut buffer);
println!("{:?}", buffer);
``` 

We use `encode_with_types` instead of `encode`. The result is same as with `encode`, just prefixed with 4 bytes which represent the types: `[6, 0, 0, 0, 8, 76]`

## Decoding and reading the values
In order to decode and encoded beads sequence, we need to create an instance of TypedBeads:
```rust
let beads = TypedBeads::new(
    buffer.as_slice(),
    &BeadTypeSet::new(&[BeadType::TrueFlag, BeadType::FalseFlag])
);
println!("Number of elements: {}", beads.len());
```
The `new` function receives a slice of the buffer which was produced with `encode` method and the information about the types. 
If we encoded the sequence with `encode_with_types`, we should instantiate the TypedBeads as following:
```rust
let beads = TypedBeads::new_types_included(
    buffer.as_slice()
);
println!("Number of elements: {}", beads.len());
```
The buffer includes the type information, so we should not provide it explicitly.

The instantiation of `TypedBeads` is very light weight. In fact it is just a wrapper for us to get the length of the sequence and an `Iterator`.

Reading the values can be performed through iteration:
```rust
for bead in beads.iter() {
    println!("{}", bead.to_bool())
}
```

The `iter` method returns a `BeadsIterator` which implements the `Iterator` where `Item = BeadReference`.
So the type of the `bead` variable is `BeadReference`.

`BeadReference` lets us examine the bead and reinterpret it as a specific type.

For examination `BeadReference` includes multiple `is_XXX` methods:
- is_none 
- is_true
- is_false
- is_bool
- is_uint
- is_int
- is_float
- is_bytes
- is_string

The reinterpretation methods are:
- to_bool
- to_int
- to_uint
- to_float
- to_string
- to_bytes

The reinterpretation methods will either convert the stored values to the target type or panic.
The `to_uint` method will work not only for `uint` values, but also for `int` values if they are positive.
`to_int` method will also work for `uint` values, until they are smaller than `i128::MAX`.
The `to_float` method is the safest to use, as it works for `int`, `uint` and `float`. (There could be rounding errors though)

`BeadReference` can also be safely converted to `i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, f32, f64, String` as we implement `TryFrom<BeadReference<'_>>` trait for all those types.

### Can we access values directly, without iterating over the whole sequence?
In some cases we can. If the elements we store are symmetrical, than we can produce a `SymmetricTypedBeads` from the `TypedBeads`:
```rust
let mut builder = TypedBeadsBuilder::new(
    &BeadTypeSet::new(&[BeadType::TrueFlag, BeadType::FalseFlag])
).ok().unwrap();

builder.push_bool(true);
builder.push_bool(true);
builder.push_bool(false);
builder.push_bool(false);
builder.push_bool(true);
builder.push_bool(true);
builder.push_bool(false);
builder.push_bool(true);

let mut buffer: Vec<u8> = vec![];
builder.encode_with_types(&mut buffer);
println!("{:?}", buffer);

let beads = TypedBeads::new_types_included(
    buffer.as_slice()
);
println!("Number of elements: {}", beads.len());

let sym_beads = beads.symmetric().ok().unwrap();
println!("Value at index {} is {}", 3, sym_beads.get(3).to_bool());
```

### What does it mean to have a symmetrical beads sequence?
Symmetrical Beads sequence is a sequence which includes only fixed, same size types.
Types like `utf8, bytes, vlq, vlqz` are not fixed as different values can occupy different number of bytes. 
So if our Beads sequence contains one of this types it is not symmetrical.
If our Beads sequence contains of types `u8` and `u16` it is also not symmetrical, because some values can have size of 1 byte and others 2 bytes.
A Beads sequence which is configured with types: `u32, i32, f32` is symmetrical, as every value is represented in 4 bytes.
Same is true for the `True, False` Beads sequence and even for `True, False, None` Beads sequence.

## Float numbers and accuracy
Beads allows us to pick from three floating numbers representations `f16, f32, f64`. 
But when we push a value into the sequence we represent it as `f64`. 
This means that if we picked `f16` for the bead type and the number is not representable with `f16` the result of the push operation will be `false` and the value will not be stored in the sequence.
This is why we have a possibility to define accuracy when we store floating point numbers:
```rust
#[test]
fn roundtrip_push_double_with_accuracy() {
    let types = BeadTypeSet::new(&[BeadType::F16, BeadType::F32, BeadType::F64]);
    let mut builder = TypedBeadsBuilder::new(
        &types
    ).ok().unwrap();
    builder.push_double(0.1);
    builder.push_double_with_accuracy(0.1, std::f32::EPSILON as f64);
    builder.push_double_with_accuracy(0.1, 0.01);

    let mut buffer: Vec<u8> = vec![];
    builder.encode(&mut buffer);

    assert_eq!(buffer, vec![
        3, 6,
        154, 153, 153, 153, 153, 153, 185, 63,
        205, 204, 204, 61,
        102, 46]);

    let beads = TypedBeads::new(buffer.as_slice(), &types);
    let out_values: Vec<f64> = beads.iter().map(|b|{b.to_float()}).collect();
    assert_eq!(out_values, vec![0.1, 0.10000000149011612, 0.0999755859375]);
}
```

As you can see from this unit test, we store the number `0.1` three times.
First push is just a regular push and the `0.1` is store in 8 bytes (`f64`).
Second push is with accuracy value, which is the smallest number representable in `f32`.
We can also see that, when we convert the value back to `f64` from beads sequence, it has a small deviation from `f64` representation of `0.1`.
Last push is with a much higher accuracy threshold. So the value `0.1` can be stored in just 2 bytes (`f16`).
As you can see in the assertion the value also has quite a high deviation from `0.1`, but is still in bounds of provided accuracy.