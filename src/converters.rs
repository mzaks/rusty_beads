use std::io;
use crate::builder::{BeadsSequenceBuilder, IndexedBeadsBuilder, FixedSizeBeadsIncrementalUintBuilder, BeadsBuilder};
use crate::bead_type::{BeadTypeSet, BeadType};
use crate::sequence::BeadsSequence;
use std::collections::{HashMap};

pub fn csv_to_indexed_string_beads<W>(csv: &str, writer: &mut W) where W: io::Write {
    let csv = csv.as_bytes();
    let mut offset = 0;
    let quote = "\"".as_bytes()[0];
    let separator = ",".as_bytes()[0];
    let n = "\n".as_bytes()[0];
    let r = "\r".as_bytes()[0];
    let mut is_in_double_quotes = false;
    let mut bytes: Vec<u8> = vec![];
    let mut column_index = 0;
    let mut builders: Vec<Box<BeadsSequenceBuilder>> = vec![];

    fn add_bytes(builders: &mut Vec<Box<BeadsSequenceBuilder>>, bytes: &mut Vec<u8>, column_index: &mut usize) {
        if builders.len() <= *column_index {
            builders.push(Box::new(BeadsSequenceBuilder::new(&BeadTypeSet::new(&[BeadType::Utf8]))));
        }
        let builder = &mut builders[*column_index];
        builder.push_string(std::str::from_utf8(&bytes).unwrap());
        bytes.clear();
    }

    while offset < csv.len() {
        let char = csv[offset];
        if char == quote {
            if is_in_double_quotes == false {
                is_in_double_quotes = true;
                offset += 1;
            } else {
                if csv.len() > offset + 1 && csv[offset+1] == quote {
                    bytes.push(quote);
                    offset += 2;
                } else {
                    is_in_double_quotes = false;
                    offset += 1;
                }
            }
        } else if char == separator && is_in_double_quotes == false {

            add_bytes(&mut builders, &mut bytes, &mut column_index);

            offset += 1;
            column_index += 1;
        } else if char == n && is_in_double_quotes == false {

            add_bytes(&mut builders, &mut bytes, &mut column_index);

            offset += 1;
            column_index = 0;
        } else if char == r && csv.len() > offset + 1 && csv[offset+1] == n && is_in_double_quotes == false {

            add_bytes(&mut builders, &mut bytes, &mut column_index);

            offset += 2;
            column_index = 0;
        } else {
            bytes.push(char);
            offset += 1;
        }
    }

    if bytes.is_empty() == false {
        add_bytes(&mut builders, &mut bytes, &mut column_index);
    }

    let mut boxed_builders: Vec<Box<dyn BeadsBuilder>> = vec![];
    for b in builders {
        boxed_builders.push(b);
    }

    IndexedBeadsBuilder::encode_from_beads_builders(writer, boxed_builders);
}

pub fn string_beads_to_int_beads<W>(buffer: &[u8], type_set: &BeadTypeSet, writer: &mut W)  -> Result<(), String> where W: io::Write {
    let string_beads = BeadsSequence::new(buffer, &BeadTypeSet::new(&[BeadType::Utf8]));
    let mut builder = BeadsSequenceBuilder::new(type_set);
    for sb in string_beads.iter() {
        let s = sb.to_str();
        let v = match s.parse::<i128>() {
            Ok(v) => v,
            Err(_) => {
                if builder.push_none() == false {
                    return Err(format!("Could not parse value {}", s).to_string());
                } else {
                    continue;
                }
            }
        };
        if builder.push_int(v) == false {
            if builder.push_none() == false {
                return Err(format!("Could not push value {}", v).to_string())
            }
        }
    }
    builder.encode(writer);
    Ok(())
}

pub fn string_beads_to_double_beads<W>(buffer: &[u8], type_set: &BeadTypeSet, accuracy: f64, writer: &mut W)  -> Result<(), String> where W: io::Write {
    let string_beads = BeadsSequence::new(buffer, &BeadTypeSet::new(&[BeadType::Utf8]));
    let mut builder = BeadsSequenceBuilder::new(type_set);
    for sb in string_beads.iter() {
        let s = sb.to_str();
        let v = match s.parse::<f64>() {
            Ok(v) => v,
            Err(_) => {
                if builder.push_none() == false {
                    return Err(format!("Could not parse value '{}'", s).to_string());
                } else {
                    continue;
                }
            }
        };
        if builder.push_double_with_accuracy(v, accuracy) == false {
            if builder.push_none() == false {
                return Err(format!("Could not push value {}", v).to_string())
            }
        }
    }
    builder.encode(writer);
    Ok(())
}

pub fn string_beads_to_indexed_beads<W>(buffer: &[u8], writer: &mut W) where W: io::Write {
    let string_beads = BeadsSequence::new(buffer, &BeadTypeSet::new(&[BeadType::Utf8]));
    let mut builder = IndexedBeadsBuilder::new();
    for sb in string_beads.iter() {
        builder.push(sb.to_bytes());
    }
    builder.encode(writer);
}

pub fn u128_from_slice(slice: &[u8]) -> u128 {
    let mut tmp: [u8;16] = [0; 16];
    for i in 0..slice.len() {
        tmp[i] = slice[i];
    }
    u128::from_le_bytes(tmp) as u128
}

pub fn beads_to_dedup_beads<W>(buffer: &'_[u8], types: &BeadTypeSet, writer: &mut W) where W: io::Write {
    let beads = BeadsSequence::new(buffer, types);
    let mut lookup = HashMap::new();
    let mut value_builder = IndexedBeadsBuilder::new();
    let mut index_builder = FixedSizeBeadsIncrementalUintBuilder::new();
    let mut index = 0;
    for sb in beads.iter() {
        let bytes = sb.to_bytes();
        if lookup.contains_key(bytes) {
            index_builder.push(lookup[bytes]);
            continue
        }
        lookup.insert(bytes, index);
        value_builder.push(bytes);
        index_builder.push(index);
        index += 1;
    }

    let mut builders: Vec<Box<dyn BeadsBuilder + '_>> = Vec::new();
    let index_box: Box<dyn BeadsBuilder + '_> = Box::new(index_builder);
    let value_box: Box<dyn BeadsBuilder + '_> = Box::new(value_builder);
    builders.push(index_box);
    builders.push(value_box);

    IndexedBeadsBuilder::encode_from_beads_builders(writer, builders);
}

#[cfg(test)]
mod tests {
    use crate::converters::{csv_to_indexed_string_beads, string_beads_to_int_beads, string_beads_to_double_beads, string_beads_to_indexed_beads, u128_from_slice, beads_to_dedup_beads};
    use crate::sequence::{IndexedBeads, BeadsSequence, FixedSizeBeads};
    use crate::bead_type::{BeadTypeSet, BeadType};
    use std::convert::TryFrom;
    use crate::builder::BeadsSequenceBuilder;

    #[test]
    fn empty_string() {
        let mut out: Vec<u8> = vec![];
        csv_to_indexed_string_beads("", &mut out);
        assert_eq!(out, vec![])
    }

    #[test]
    fn one_row() {
        let mut out: Vec<u8> = vec![];
        csv_to_indexed_string_beads("a,b", &mut out);
        assert_eq!(out, vec![16, 3, 6, 1, 1, 97, 1, 1, 98]);
        let ib = IndexedBeads::new(out.as_slice());
        assert_eq!(ib.len(), 2);
        let b1 = BeadsSequence::new(ib[0].as_ref(), &BeadTypeSet::new(&[BeadType::Utf8]));
        let c1: Vec<String> = b1.iter().map(|b| String::try_from(b).unwrap()).collect();
        assert_eq!(c1, vec!["a"]);
        let b1 = BeadsSequence::new(ib[1].as_ref(), &BeadTypeSet::new(&[BeadType::Utf8]));
        let c1: Vec<String> = b1.iter().map(|b| String::try_from(b).unwrap()).collect();
        assert_eq!(c1, vec!["b"]);
    }

    #[test]
    fn two_row() {
        let mut out: Vec<u8> = vec![];
        csv_to_indexed_string_beads("a,b\n1,2", &mut out);
        assert_eq!(out, vec![16, 5, 10, 2, 1, 97, 1, 49, 2, 1, 98, 1, 50]);
        let ib = IndexedBeads::new(out.as_slice());
        assert_eq!(ib.len(), 2);
        let b1 = BeadsSequence::new(ib[0].as_ref(), &BeadTypeSet::new(&[BeadType::Utf8]));
        let c1: Vec<String> = b1.iter().map(|b| String::try_from(b).unwrap()).collect();
        assert_eq!(c1, vec!["a", "1"]);
        let b1 = BeadsSequence::new(ib[1].as_ref(), &BeadTypeSet::new(&[BeadType::Utf8]));
        let c1: Vec<String> = b1.iter().map(|b| String::try_from(b).unwrap()).collect();
        assert_eq!(c1, vec!["b", "2"]);
    }

    #[test]
    fn one_row_with_quotes() {
        let mut out: Vec<u8> = vec![];
        csv_to_indexed_string_beads("a,\"ccc\"\"b,\"\"cccc\",e", &mut out);
        assert_eq!(out, vec![24, 3, 16, 19, 1, 1, 97, 1, 11, 99, 99, 99, 34, 98, 44, 34, 99, 99, 99, 99, 1, 1, 101]);
        let ib = IndexedBeads::new(out.as_slice());
        assert_eq!(ib.len(), 3);
        let b = BeadsSequence::new(ib[0].as_ref(), &BeadTypeSet::new(&[BeadType::Utf8]));
        let c: Vec<String> = b.iter().map(|b| String::try_from(b).unwrap()).collect();
        assert_eq!(c, vec!["a"]);
        let b = BeadsSequence::new(ib[1].as_ref(), &BeadTypeSet::new(&[BeadType::Utf8]));
        let c: Vec<String> = b.iter().map(|b| String::try_from(b).unwrap()).collect();
        assert_eq!(c, vec!["ccc\"b,\"cccc"]);

        let b = BeadsSequence::new(ib[2].as_ref(), &BeadTypeSet::new(&[BeadType::Utf8]));
        let c: Vec<String> = b.iter().map(|b| String::try_from(b).unwrap()).collect();
        assert_eq!(c, vec!["e"]);
    }

    #[test]
    fn convert_string_beads_to_i32() {
        let mut builder = BeadsSequenceBuilder::new(&BeadTypeSet::new(&[BeadType::Utf8]));
        builder.push_string("1");
        builder.push_string("-1");
        builder.push_string("0");
        builder.push_string("1535340");

        let mut buffer: Vec<u8> = vec![];
        builder.encode(&mut buffer);

        let mut out: Vec<u8> = vec![];
        string_beads_to_int_beads(buffer.as_slice(), &BeadTypeSet::new(&[BeadType::I32]), &mut out).unwrap();

        assert_eq!(out, vec![4, 1, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 108, 109, 23, 0]);

        let beads = BeadsSequence::new(out.as_slice(), &BeadTypeSet::new(&[BeadType::I32]));
        let sym_b = beads.symmetric().ok().unwrap();
        assert_eq!(sym_b.get(0).to_int(), 1);
        assert_eq!(sym_b.get(1).to_int(), -1);
        assert_eq!(sym_b.get(2).to_int(), 0);
        assert_eq!(sym_b.get(3).to_int(), 1_535_340);
    }

    #[test]
    fn convert_string_beads_to_u8() {
        let mut builder = BeadsSequenceBuilder::new(&BeadTypeSet::new(&[BeadType::Utf8]));
        builder.push_string("1");
        builder.push_string("100");
        builder.push_string("0");
        builder.push_string("255");

        let mut buffer: Vec<u8> = vec![];
        builder.encode(&mut buffer);

        let mut out: Vec<u8> = vec![];
        string_beads_to_int_beads(buffer.as_slice(), &BeadTypeSet::new(&[BeadType::U8]), &mut out).unwrap();

        assert_eq!(out, vec![4, 1, 100, 0, 255]);

        let beads = BeadsSequence::new(out.as_slice(), &BeadTypeSet::new(&[BeadType::U8]));
        let sym_b = beads.symmetric().ok().unwrap();
        assert_eq!(sym_b.get(0).to_int(), 1);
        assert_eq!(sym_b.get(1).to_int(), 100);
        assert_eq!(sym_b.get(2).to_int(), 0);
        assert_eq!(sym_b.get(3).to_int(), 255);
    }

    #[test]
    fn convert_string_beads_to_u8_and_i8() {
        let mut builder = BeadsSequenceBuilder::new(&BeadTypeSet::new(&[BeadType::Utf8]));
        builder.push_string("1");
        builder.push_string("100");
        builder.push_string("0");
        builder.push_string("-25");

        let mut buffer: Vec<u8> = vec![];
        builder.encode(&mut buffer);

        let mut out: Vec<u8> = vec![];
        string_beads_to_int_beads(buffer.as_slice(), &BeadTypeSet::new(&[BeadType::U8, BeadType::I8]), &mut out).unwrap();

        assert_eq!(out, vec![4, 15, 1, 100, 0, 231]);

        let beads = BeadsSequence::new(out.as_slice(), &BeadTypeSet::new(&[BeadType::U8, BeadType::I8]));
        let sym_b = beads.symmetric().ok().unwrap();
        assert_eq!(sym_b.get(0).to_int(), 1);
        assert_eq!(sym_b.get(1).to_int(), 100);
        assert_eq!(sym_b.get(2).to_int(), 0);
        assert_eq!(sym_b.get(3).to_int(), -25);
    }

    #[test]
    fn convert_string_beads_to_u8_and_none() {
        let mut builder = BeadsSequenceBuilder::new(&BeadTypeSet::new(&[BeadType::Utf8]));
        builder.push_string("1");
        builder.push_string("100");
        builder.push_string("0");
        builder.push_string("");
        builder.push_string("-25");

        let mut buffer: Vec<u8> = vec![];
        builder.encode(&mut buffer);

        let mut out: Vec<u8> = vec![];
        string_beads_to_int_beads(buffer.as_slice(), &BeadTypeSet::new(&[BeadType::U8, BeadType::None]), &mut out).unwrap();

        assert_eq!(out, vec![5, 7, 1, 100, 0]);

        let beads = BeadsSequence::new(out.as_slice(), &BeadTypeSet::new(&[BeadType::U8, BeadType::None]));
        assert_eq!(beads.is_symmetrical(), false);
        let values = [1, 100, 0];
        for (i, b) in beads.iter().enumerate() {
            if i < 3 {
                assert_eq!(b.to_int(), values[i]);
            } else {
                assert_eq!(b.is_none(), true);
            }
        }
    }

    #[test]
    fn convert_string_double_beads_to_u8_and_i8() {
        let mut builder = BeadsSequenceBuilder::new(&BeadTypeSet::new(&[BeadType::Utf8]));
        builder.push_string("1.0");
        builder.push_string("100.0");
        builder.push_string("0.0");
        builder.push_string("-25.0");

        let mut buffer: Vec<u8> = vec![];
        builder.encode(&mut buffer);

        let mut out: Vec<u8> = vec![];
        string_beads_to_double_beads(buffer.as_slice(), &BeadTypeSet::new(&[BeadType::U8, BeadType::I8]), 0.0, &mut out).unwrap();

        assert_eq!(out, vec![4, 8, 1, 100, 0, 231]);

        let beads = BeadsSequence::new(out.as_slice(), &BeadTypeSet::new(&[BeadType::U8, BeadType::I8]));
        let sym_b = beads.symmetric().ok().unwrap();
        assert_eq!(sym_b.get(0).to_int(), 1);
        assert_eq!(sym_b.get(1).to_int(), 100);
        assert_eq!(sym_b.get(2).to_int(), 0);
        assert_eq!(sym_b.get(3).to_int(), -25);
    }

    #[test]
    fn convert_string_double_beads_to_f32() {
        let mut builder = BeadsSequenceBuilder::new(&BeadTypeSet::new(&[BeadType::Utf8]));
        builder.push_string("1.1");
        builder.push_string("100.5");
        builder.push_string("0.0");
        builder.push_string("-25.0");

        let mut buffer: Vec<u8> = vec![];
        builder.encode(&mut buffer);

        let mut out: Vec<u8> = vec![];
        string_beads_to_double_beads(buffer.as_slice(), &BeadTypeSet::new(&[BeadType::F32]), std::f32::EPSILON as f64, &mut out).unwrap();

        assert_eq!(out, vec![4, 205, 204, 140, 63, 0, 0, 201, 66, 0, 0, 0, 0, 0, 0, 200, 193]);

        let beads = BeadsSequence::new(out.as_slice(), &BeadTypeSet::new(&[BeadType::F32]));
        let sym_b = beads.symmetric().ok().unwrap();
        assert_eq!(sym_b.get(0).to_float(), 1.100000023841858);
        assert_eq!(sym_b.get(1).to_float(), 100.5);
        assert_eq!(sym_b.get(2).to_float(), 0.0);
        assert_eq!(sym_b.get(3).to_float(), -25.0);
    }

    #[test]
    fn convert_string_beads_to_indexed() {
        let mut builder = BeadsSequenceBuilder::new(&BeadTypeSet::new(&[BeadType::Utf8]));
        builder.push_string("1.1");
        builder.push_string("100.5");
        builder.push_string("0.0");
        builder.push_string("-25.0");
        builder.push_string("🤪");

        let mut buffer: Vec<u8> = vec![];
        builder.encode(&mut buffer);

        let mut out: Vec<u8> = vec![];
        string_beads_to_indexed_beads(&buffer, &mut out);

        assert_eq!(out, vec![40, 3, 8, 11, 16, 20, 49, 46, 49, 49, 48, 48, 46, 53, 48, 46, 48, 45, 50, 53, 46, 48, 240, 159, 164, 170]);

        let beads = IndexedBeads::new(out.as_slice());
        assert_eq!(beads.len(), 5);
        assert_eq!(std::str::from_utf8(&beads[0]).unwrap(), "1.1");
        assert_eq!(std::str::from_utf8(&beads[1]).unwrap(), "100.5");
        assert_eq!(std::str::from_utf8(&beads[2]).unwrap(), "0.0");
        assert_eq!(std::str::from_utf8(&beads[3]).unwrap(), "-25.0");
        assert_eq!(std::str::from_utf8(&beads[4]).unwrap(), "🤪");
    }

    #[test]
    fn convert_slice_to_u128() {
        assert_eq!(1, u128_from_slice(&[1]));
        assert_eq!(513, u128_from_slice(&[1, 2]));
        assert_eq!(65793, u128_from_slice(&[1, 1, 1]));
    }

    #[test]
    fn convert_string_beads_to_dedup_beads() {
        let mut builder = BeadsSequenceBuilder::new(&BeadTypeSet::new(&[BeadType::Utf8]));
        builder.push_string("Max");
        builder.push_string("Maxim");
        builder.push_string("Alex");
        builder.push_string("Max");
        builder.push_string("🤪");
        builder.push_string("Maxim");
        builder.push_string("Max");
        builder.push_string("Alex");
        builder.push_string("🤪");

        let mut buffer: Vec<u8> = vec![];
        builder.encode(&mut buffer);

        let mut out: Vec<u8> = vec![];
        beads_to_dedup_beads(&buffer, &BeadTypeSet::new(&[BeadType::Utf8]), &mut out);

        assert_eq!(out, vec![
            16, 10, 31,
            1, 0, 1, 2, 0, 3, 1, 0, 2, 3,
            32, 3, 8, 12, 16,
            77, 97, 120, 77, 97, 120, 105, 109, 65, 108, 101, 120, 240, 159, 164, 170
        ]);

        let beads = IndexedBeads::new(out.as_slice());
        assert_eq!(beads.len(), 2);
        let index = FixedSizeBeads::new(&beads[0]);
        let values = IndexedBeads::new(&beads[1]);
        assert_eq!(index.len(), 9);
        assert_eq!(values.len(), 4);
        assert_eq!(&values[u128_from_slice(&index[0]) as usize], "Max".as_bytes());
        assert_eq!(&values[u128_from_slice(&index[1]) as usize], "Maxim".as_bytes());
        assert_eq!(&values[u128_from_slice(&index[2]) as usize], "Alex".as_bytes());
        assert_eq!(&values[u128_from_slice(&index[3]) as usize], "Max".as_bytes());
        assert_eq!(&values[u128_from_slice(&index[4]) as usize], "🤪".as_bytes());
        assert_eq!(&values[u128_from_slice(&index[5]) as usize], "Maxim".as_bytes());
        assert_eq!(&values[u128_from_slice(&index[6]) as usize], "Max".as_bytes());
        assert_eq!(&values[u128_from_slice(&index[7]) as usize], "Alex".as_bytes());
        assert_eq!(&values[u128_from_slice(&index[8]) as usize], "🤪".as_bytes());
    }
}