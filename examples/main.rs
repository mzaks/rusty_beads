use rusty_beads::builder::BeadsSequenceBuilder;
use rusty_beads::bead_type::{BeadTypeSet, BeadType};
use rusty_beads::sequence::BeadsSequence;
use std::convert::TryFrom;

fn main() {
    bit_set();

    run_numbers();
    run_strings();
}

fn bit_set() {
    let mut builder = BeadsSequenceBuilder::new(
        &BeadTypeSet::new(&[BeadType::TrueFlag, BeadType::FalseFlag])
    );

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

    let beads = BeadsSequence::new(
        buffer.as_slice(),
        &BeadTypeSet::new(&[BeadType::TrueFlag, BeadType::FalseFlag])
    );
    println!("Number of elements: {}", beads.len());

    for bead in beads.iter() {
        println!("{}", bead.to_bool())
    }

    let mut buffer: Vec<u8> = vec![];
    builder.encode_with_types(&mut buffer);
    println!("{:?}", buffer);

    let beads = BeadsSequence::new_types_included(
        buffer.as_slice()
    );
    println!("Number of elements: {}", beads.len());

    let sym_beads = beads.symmetric().ok().unwrap();
    println!("Value at index {} is {}", 3, sym_beads.get(3).to_bool());
}

fn run_numbers() {
    let types = BeadTypeSet::new(&[BeadType::Vlq, BeadType::VlqZ, BeadType::U8, BeadType::I8]);
    let mut builder = BeadsSequenceBuilder::new(
        &types
    );
    let data = [145, -145, 4502345, -4502345, 0, 3945873459];
    for v in data.iter() {
        builder.push_int(*v);
    }

    let mut out: Vec<u8>  = vec![];
    builder.encode(&mut out);
    println!("Data as beads: {:?}, len: {}", out, out.len());

    let beads = BeadsSequence::new(out.as_slice(), &types);

    println!("Beads count: {}", beads.len());
    for b in beads.iter() {
        println!("{}", b.to_int());
    }
    let numbers:Vec<i64> = beads.iter().map(|b| i64::try_from(b.to_int()).unwrap()).collect();
    println!("Numbers: {:?}", numbers);

    let numbers:Vec<f64> = beads.iter().map(|b| b.to_float()).collect();
    println!("Numbers: {:?}", numbers);

    for b in beads.iter() {
        println!("{:?}", b.to_bytes());
    }
}

fn run_strings() {
    let types = BeadTypeSet::new(&[BeadType::Utf8, BeadType::None]);
    let mut builder = BeadsSequenceBuilder::new(
        &types
    );
    let data: Vec<Option<&str>> = vec![Some("hello"), None, Some("this is something"), None, None, Some("special"), Some("")];
    for v in data.iter() {
        match v {
            Some(s) => builder.push_string(s),
            None => builder.push_none()
        };
    }

    let mut out: Vec<u8>  = vec![];
    builder.encode(&mut out);
    println!("Data as beads: {:?}, len: {}", out, out.len());

    let beads = BeadsSequence::new(out.as_slice(), &types);

    println!("Beads count: {}", beads.len());
    for b in beads.iter() {
        println!("{:?}", String::try_from(b));
    }
    let strings:Vec<Option<String>> = beads.iter().map(|b| {
        String::try_from(b).ok()
    }).collect();
    println!("Strings: {:?}", strings);

    for b in beads.iter() {
        println!("{:?}", b.to_bytes());
    }
}