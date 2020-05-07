use crate::bead_type::{BeadType, BeadTypeSet};
use crate::builder::{BeadsSequenceBuilder, IndexedBeadsBuilder, FixedSizeBeadsBuilder, FixedSizeBeadsIncrementalUintBuilder};
use crate::sequence::{BeadsSequence, IndexedBeads, FixedSizeBeads, DedupBeads};
use crate::converters::beads_to_dedup_beads;
use std::f64;

#[test]
fn bead_type_set() {
    let set = BeadTypeSet::new(&[BeadType::None, BeadType::F32, BeadType::U8]);
    assert!(set.contains(&BeadType::None));
    assert!(set.contains(&BeadType::F32));
    assert!(set.contains(&BeadType::U8));
    assert!(!set.contains(&BeadType::U16));
    assert_eq!(set.size(), 3)
}

#[test]
fn push_bool_beads_sequence() {
    let mut builder = BeadsSequenceBuilder::new(
        &BeadTypeSet::new(&[BeadType::TrueFlag, BeadType::FalseFlag])
    );
    builder.push_bool(true);
    builder.push_bool(true);
    builder.push_bool(false);
    builder.push_bool(true);
    builder.push_bool(false);
    builder.push_bool(false);
    builder.push_bool(false);
    builder.push_bool(true);
    builder.push_bool(false);
    builder.push_bool(false);
    builder.push_bool(true);
    builder.push_bool(false);

    let mut out = Vec::new();
    builder.encode(&mut out);
    assert_eq!(out, vec![12, 116, 11]);

    out.clear();
    builder.encode_with_types(&mut out);
    assert_eq!(out, vec![6, 0, 0, 0, 12, 116, 11]);
}

#[test]
fn push_bool_beads_and_none_sequence() {
    let mut builder = BeadsSequenceBuilder::new(
        &BeadTypeSet::new(&[BeadType::TrueFlag, BeadType::FalseFlag, BeadType::None])
    );
    builder.push_bool(true);
    builder.push_bool(true);
    builder.push_none();
    builder.push_bool(false);
    builder.push_bool(true);
    builder.push_bool(false);
    builder.push_bool(false);
    builder.push_bool(false);
    builder.push_bool(true);
    builder.push_bool(false);
    builder.push_bool(false);
    builder.push_bool(true);
    builder.push_bool(false);

    let mut out = Vec::new();
    builder.encode(&mut out);
    assert_eq!(out, vec![13, 133, 169, 105, 2]);

    out.clear();
    builder.encode_with_types(&mut out);
    assert_eq!(out, vec![7, 0, 0, 0, 13, 133, 169, 105, 2]);
}

#[test]
fn push_u8_beads_sequence() {
    let mut builder = BeadsSequenceBuilder::new(
        &BeadTypeSet::new(&[BeadType::U8, BeadType::None])
    );

    builder.push_uint(20);
    builder.push_none();
    builder.push_uint(21);
    builder.push_int(22);

    let mut out = Vec::new();
    builder.encode(&mut out);
    assert_eq!(out, vec![4, 13, 20, 21, 22]);

    out.clear();
    builder.encode_with_types(&mut out);
    assert_eq!(out, vec![9, 0, 0, 0, 4, 13, 20, 21, 22]);
}

#[test]
fn push_u16_beads_sequence() {
    let mut builder = BeadsSequenceBuilder::new(
        &BeadTypeSet::new(&[BeadType::U8, BeadType::U16])
    );

    builder.push_uint(20);
    builder.push_uint(261);
    builder.push_int(22);

    let mut out = Vec::new();
    builder.encode(&mut out);
    assert_eq!(out, vec![3, 2, 20, 5, 1, 22]);
}

#[test]
fn push_u32_beads_sequence() {
    let mut builder = BeadsSequenceBuilder::new(
        &BeadTypeSet::new(&[BeadType::U8, BeadType::U32])
    );

    builder.push_uint(20);
    builder.push_uint(261000);
    builder.push_int(22);

    let mut out = Vec::new();
    builder.encode(&mut out);
    assert_eq!(out, vec![3, 2, 20, 136, 251, 3, 0, 22]);
}

#[test]
fn push_vlq_beads_sequence() {
    let mut builder = BeadsSequenceBuilder::new(
        &BeadTypeSet::new(&[BeadType::Vlq, BeadType::U32])
    );

    builder.push_uint(20);
    builder.push_uint(261000);
    builder.push_int(22);

    let mut out = Vec::new();
    builder.encode(&mut out);
    assert_eq!(out, vec![3, 7, 20, 136, 247, 15, 22]);
}
#[test]
fn push_vlqz_beads_sequence() {
    let mut builder = BeadsSequenceBuilder::new(
        &BeadTypeSet::new(&[BeadType::VlqZ, BeadType::U32])
    );

    builder.push_uint(20);
    builder.push_uint(261000);
    builder.push_uint(22);
    builder.push_int(-22);

    let mut out = Vec::new();
    builder.encode(&mut out);
    assert_eq!(out, vec![4, 15, 40, 144, 238, 31, 44, 43]);
}

#[test]
fn push_f32_beads_sequence() {
    let mut builder = BeadsSequenceBuilder::new(
        &BeadTypeSet::new(&[BeadType::F32, BeadType::U128])
    );

    builder.push_uint(20);
    builder.push_uint(261000);
    builder.push_uint(22);
    builder.push_int(-22);

    let mut out = Vec::new();
    builder.encode(&mut out);
    assert_eq!(out, vec![4, 15, 0, 0, 160, 65, 0, 226, 126, 72, 0, 0, 176, 65, 0, 0, 176, 193]);
}

#[test]
fn push_f64_beads_sequence() {
    let mut builder = BeadsSequenceBuilder::new(
        &BeadTypeSet::new(&[BeadType::F64, BeadType::U128])
    );

    builder.push_uint(20);
    builder.push_uint(261000);
    builder.push_uint(22);
    builder.push_int(-22);

    let mut out = Vec::new();
    builder.encode(&mut out);
    assert_eq!(out, vec![4, 15, 0, 0, 0, 0, 0, 0, 52, 64, 0, 0, 0, 0, 64, 220, 15, 65, 0, 0, 0, 0, 0, 0, 54, 64, 0, 0, 0, 0, 0, 0, 54, 192]);
}

#[test]
fn push_f16_beads_sequence() {
    let mut builder = BeadsSequenceBuilder::new(
        &BeadTypeSet::new(&[BeadType::F16, BeadType::F32])
    );

    builder.push_uint(20);
    builder.push_uint(261000);
    builder.push_uint(22);
    builder.push_int(-22);

    let mut out = Vec::new();
    builder.encode(&mut out);
    assert_eq!(out, vec![4, 2, 0, 77, 0, 226, 126, 72, 128, 77, 128, 205]);

    out.clear();
    builder.encode_with_types(&mut out);
    assert_eq!(out, vec![0, 96, 0, 0, 4, 2, 0, 77, 0, 226, 126, 72, 128, 77, 128, 205]);
}

#[test]
fn push_i16_beads_sequence() {
    let mut builder = BeadsSequenceBuilder::new(
        &BeadTypeSet::new(&[BeadType::I8, BeadType::I16])
    );

    builder.push_uint(20);
    builder.push_int(-261);
    builder.push_int(-22);

    let mut out = Vec::new();
    builder.encode(&mut out);
    assert_eq!(out, vec![3, 2, 20, 251, 254, 234]);
}


#[test]
fn push_many_ints() {
    let mut builder = BeadsSequenceBuilder::new(
        &BeadTypeSet::new(&[BeadType::I8, BeadType::None])
    );
    for i in 1..14 {
        builder.push_int(i);
        if i == 7 {
            builder.push_none();
        }
    }

    let mut out = Vec::new();
    builder.encode(&mut out);
    assert_eq!(out, vec![14, 127, 1, 2, 3, 4, 5, 6, 7, 63, 8, 9, 10, 11, 12, 13]);
}

#[test]
fn push_many_ints_four_types() {
    let mut builder = BeadsSequenceBuilder::new(
        &BeadTypeSet::new(&[BeadType::I8, BeadType::None, BeadType::Vlq, BeadType::VlqZ])
    );
    for i in 1..14 {
        builder.push_int(i);
        if i == 7 {
            builder.push_none();
        }
    }

    let mut out = Vec::new();
    builder.encode(&mut out);
    assert_eq!(out, vec![14, 85, 1, 2, 3, 4, 21, 5, 6, 7, 85, 8, 9, 10, 11, 5, 12, 13]);

    out.clear();
    builder.encode_with_types(&mut out);
    assert_eq!(out, vec![1, 1, 3, 0, 14, 85, 1, 2, 3, 4, 21, 5, 6, 7, 85, 8, 9, 10, 11, 5, 12, 13]);
}

#[test]
fn push_many_ints_sixteen_types() {
    let mut builder = BeadsSequenceBuilder::new(
        &BeadTypeSet::new(&[BeadType::I8, BeadType::None, BeadType::Vlq, BeadType::VlqZ, BeadType::TrueFlag])
    );
    for i in 1..14 {
        builder.push_int(i);
        if i == 7 {
            builder.push_none();
        }
    }

    let mut out = Vec::new();
    builder.encode(&mut out);
    assert_eq!(out, vec![14, 34, 1, 2, 34, 3, 4, 34, 5, 6, 2, 7, 34, 8, 9, 34, 10, 11, 34, 12, 13]);

    out.clear();
    builder.encode_with_types(&mut out);
    assert_eq!(out, vec![3, 1, 3, 0, 14, 34, 1, 2, 34, 3, 4, 34, 5, 6, 2, 7, 34, 8, 9, 34, 10, 11, 34, 12, 13]);
}

#[test]
fn push_string() {
    let mut builder = BeadsSequenceBuilder::new(
        &BeadTypeSet::new(&[BeadType::Utf8, BeadType::None])
    );
    builder.push_string("Maxim");

    let mut out = Vec::new();
    builder.encode(&mut out);
    assert_eq!(out, vec![1, 1, 5, 77, 97, 120, 105, 109]);

    out.clear();
    builder.push_string("Hello 🤪");
    builder.encode(&mut out);
    assert_eq!(out, vec![2, 3, 5, 77, 97, 120, 105, 109, 10, 72, 101, 108, 108, 111, 32, 240, 159, 164, 170]);

    out.clear();
    builder.push_none();
    builder.encode(&mut out);
    assert_eq!(out, vec![3, 3, 5, 77, 97, 120, 105, 109, 10, 72, 101, 108, 108, 111, 32, 240, 159, 164, 170]);

    out.clear();
    builder.push_string("Aha!");
    builder.encode(&mut out);
    assert_eq!(out, vec![4, 11, 5, 77, 97, 120, 105, 109, 10, 72, 101, 108, 108, 111, 32, 240, 159, 164, 170, 4, 65, 104, 97, 33]);
}

#[test]
fn push_string_and_bytes() {
    let mut builder = BeadsSequenceBuilder::new(
        &BeadTypeSet::new(&[BeadType::Utf8, BeadType::Bytes])
    );
    builder.push_string("Maxim");

    let mut out = Vec::new();
    builder.encode(&mut out);
    assert_eq!(out, vec![1, 0, 5, 77, 97, 120, 105, 109]);

    out.clear();
    builder.push_bytes("Hello 🤪".as_bytes());
    builder.encode(&mut out);
    assert_eq!(out, vec![2, 2, 5, 77, 97, 120, 105, 109, 10, 72, 101, 108, 108, 111, 32, 240, 159, 164, 170]);

    out.clear();
    builder.push_bytes("Aha!".as_bytes());
    builder.encode(&mut out);
    assert_eq!(out, vec![3, 6, 5, 77, 97, 120, 105, 109, 10, 72, 101, 108, 108, 111, 32, 240, 159, 164, 170, 4, 65, 104, 97, 33]);
}

#[test]
fn push_long_string() {
    let mut builder = BeadsSequenceBuilder::new(
        &BeadTypeSet::new(&[BeadType::Utf8, BeadType::None])
    );
    builder.push_string(r#"
Lorem ipsum, or lipsum as it is sometimes known,
is dummy text used in laying out print,
graphic or web designs.
The passage is attributed to an unknown typesetter in the 15th century
who is thought to have scrambled parts of Cicero's De Finibus Bonorum et Malorum
for use in a type specimen book."#);

    let mut out = Vec::new();
    builder.encode(&mut out);
    assert_eq!(out, vec![1, 1, 170, 2,
                         10, 76, 111, 114, 101, 109, 32, 105, 112, 115, 117, 109, 44, 32,
                         111, 114, 32, 108, 105, 112, 115, 117, 109, 32,
                         97, 115, 32, 105, 116, 32, 105, 115, 32,
                         115, 111, 109, 101, 116, 105, 109, 101, 115, 32,
                         107, 110, 111, 119, 110, 44, 10, 105, 115, 32,
                         100, 117, 109, 109, 121, 32, 116, 101, 120, 116, 32,
                         117, 115, 101, 100, 32, 105, 110, 32, 108, 97, 121, 105, 110, 103, 32,
                         111, 117, 116, 32, 112, 114, 105, 110, 116, 44, 10,
                         103, 114, 97, 112, 104, 105, 99, 32, 111, 114, 32, 119, 101, 98, 32,
                         100, 101, 115, 105, 103, 110, 115, 46, 10, 84, 104, 101, 32,
                         112, 97, 115, 115, 97, 103, 101, 32, 105, 115, 32,
                         97, 116, 116, 114, 105, 98, 117, 116, 101, 100, 32, 116, 111, 32,
                         97, 110, 32, 117, 110, 107, 110, 111, 119, 110, 32,
                         116, 121, 112, 101, 115, 101, 116, 116, 101, 114, 32, 105, 110, 32,
                         116, 104, 101, 32, 49, 53, 116, 104, 32,
                         99, 101, 110, 116, 117, 114, 121, 10,
                         119, 104, 111, 32, 105, 115, 32, 116, 104, 111, 117, 103, 104, 116, 32,
                         116, 111, 32, 104, 97, 118, 101, 32,
                         115, 99, 114, 97, 109, 98, 108, 101, 100, 32, 112, 97, 114, 116, 115, 32,
                         111, 102, 32, 67, 105, 99, 101, 114, 111, 39, 115, 32, 68, 101, 32,
                         70, 105, 110, 105, 98, 117, 115, 32, 66, 111, 110, 111, 114, 117, 109, 32,
                         101, 116, 32, 77, 97, 108, 111, 114, 117, 109, 10,
                         102, 111, 114, 32, 117, 115, 101, 32, 105, 110, 32, 97, 32,
                         116, 121, 112, 101, 32, 115, 112, 101, 99, 105, 109, 101, 110, 32,
                         98, 111, 111, 107, 46]);
}

#[test]
fn create_beads_sequence() {
    let beads = BeadsSequence::new(
        &[12, 116, 11],
        &BeadTypeSet::new(&[BeadType::TrueFlag, BeadType::FalseFlag])
    );
    assert_eq!(beads.len(), 12);

    let refs: Vec<bool> =  beads.iter().map(|x| x.to_bool()).collect();
    assert_eq!(refs.len(), 12);
    assert_eq!(refs, vec![true, true, false, true, false, false, false, true, false, false, true, false])
}

#[test]
fn roundtrip_strings() {
    let types = BeadTypeSet::new(&[BeadType::Utf8, BeadType::None]);
    let mut builder = BeadsSequenceBuilder::new(&types);
    let strings = vec!["Hello", "my name is Maxim", "what about you? 🤨"];
    for s in strings.iter() {
        builder.push_string(s);
    }
    let mut out = Vec::new();
    builder.encode(&mut out);

    let beads = BeadsSequence::new(out.as_slice(), &types);
    for (index, b) in beads.iter().enumerate() {
        assert_eq!(b.to_str(), strings[index])
    }
    let values: Vec<&str> = beads.iter().map(|b| b.to_str()).collect();
    assert_eq!(strings, values);
}

#[test]
fn roundtrip_strings_types_included() {
    let types = BeadTypeSet::new(&[BeadType::Utf8, BeadType::None]);
    let mut builder = BeadsSequenceBuilder::new(&types);
    let strings = vec!["Hello", "my name is Maxim", "what about you? 🤨"];
    for s in strings.iter() {
        builder.push_string(s);
    }
    let mut out = Vec::new();
    builder.encode_with_types(&mut out);

    let beads = BeadsSequence::new_types_included(out.as_slice());
    for (index, b) in beads.iter().enumerate() {
        assert_eq!(b.to_str(), strings[index])
    }
}

#[test]
fn roundtrip_strings_3_types() {
    let types = BeadTypeSet::new(&[BeadType::Utf8, BeadType::None, BeadType::Bytes]);
    let mut builder = BeadsSequenceBuilder::new(&types);
    let strings = vec!["Hello", "my name is Maxim", "what about you? 🤨", "$$$", "2343"];
    for s in strings.iter() {
        builder.push_string(s);
    }
    let mut out = Vec::new();
    builder.encode(&mut out);

    let beads = BeadsSequence::new(out.as_slice(), &types);
    for (index, b) in beads.iter().enumerate() {
        assert_eq!(b.to_str(), strings[index])
    }
}

#[test]
fn roundtrip_strings_5_types() {
    let types = BeadTypeSet::new(&[BeadType::Utf8, BeadType::None, BeadType::Bytes, BeadType::VlqZ, BeadType::Vlq]);
    let mut builder = BeadsSequenceBuilder::new(&types);
    let strings = vec!["Hello", "my name is Maxim", "what about you? 🤨"];
    for s in strings.iter() {
        builder.push_string(s);
    }
    let mut out = Vec::new();
    builder.encode(&mut out);

    let beads = BeadsSequence::new(out.as_slice(), &types);
    for (index, b) in beads.iter().enumerate() {
        assert_eq!(b.to_str(), strings[index])
    }
}

#[test]
fn roundtrip_strings_5_types_types_included() {
    let types = BeadTypeSet::new(&[BeadType::Utf8, BeadType::None, BeadType::Bytes, BeadType::VlqZ, BeadType::Vlq]);
    let mut builder = BeadsSequenceBuilder::new(&types);
    let strings = vec!["Hello", "my name is Maxim", "what about you? 🤨"];
    for s in strings.iter() {
        builder.push_string(s);
    }
    let mut out = Vec::new();
    builder.encode_with_types(&mut out);

    let beads = BeadsSequence::new_types_included(out.as_slice());
    for (index, b) in beads.iter().enumerate() {
        assert_eq!(b.to_str(), strings[index])
    }
}

#[test]
fn non_symmetric_sequence() {
    let types = BeadTypeSet::new(&[BeadType::Utf8, BeadType::None, BeadType::Bytes, BeadType::VlqZ, BeadType::Vlq]);
    let mut builder = BeadsSequenceBuilder::new(&types);
    let strings = vec!["Hello", "my name is Maxim", "what about you? 🤨"];
    for s in strings.iter() {
        builder.push_string(s);
    }
    let mut out = Vec::new();
    builder.encode(&mut out);

    let beads = BeadsSequence::new(out.as_slice(), &types);
    assert_eq!(beads.is_symmetrical(), false);
}

#[test]
fn symmetric_sequence_bool() {
    let types = BeadTypeSet::new(&[BeadType::TrueFlag, BeadType::FalseFlag]);
    let mut builder = BeadsSequenceBuilder::new(&types);
    let bools = vec![true, true, false, true];
    for b in bools.iter() {
        builder.push_bool(*b);
    }
    let mut out = Vec::new();
    builder.encode(&mut out);

    let beads = BeadsSequence::new(out.as_slice(), &types);
    assert_eq!(beads.is_symmetrical(), true);

    let symb = beads.symmetric().ok().unwrap();
    assert_eq!(symb.len(), 4);
    assert_eq!(symb.get(0).to_bool(), true);
    assert_eq!(symb.get(1).to_bool(), true);
    assert_eq!(symb.get(2).to_bool(), false);
    assert_eq!(symb.get(3).to_bool(), true);
}

#[test]
fn symmetric_sequence_bool_and_none() {
    let types = BeadTypeSet::new(&[BeadType::TrueFlag, BeadType::FalseFlag, BeadType::None]);
    let mut builder = BeadsSequenceBuilder::new(&types);
    let bools = vec![true, true, false, true];
    for b in bools.iter() {
        builder.push_bool(*b);
    }
    builder.push_none();
    let mut out = Vec::new();
    builder.encode(&mut out);

    let beads = BeadsSequence::new(out.as_slice(), &types);
    assert_eq!(beads.is_symmetrical(), true);
    assert_eq!(beads.len(), 5);

    let symb = beads.symmetric().ok().unwrap();
    assert_eq!(symb.len(), 5);
    assert_eq!(symb.get(0).to_bool(), true);
    assert_eq!(symb.get(1).to_bool(), true);
    assert_eq!(symb.get(2).to_bool(), false);
    assert_eq!(symb.get(3).to_bool(), true);
    assert_eq!(symb.get(4).is_none(), true);
}

#[test]
fn symmetric_sequence_one_byte_numbers() {
    let types = BeadTypeSet::new(&[BeadType::U8, BeadType::I8]);
    let mut builder = BeadsSequenceBuilder::new(&types);
    let values = vec![1, -4, 6, 0, -9];
    for v in values.iter() {
        builder.push_int(*v);
    }
    let mut out = Vec::new();
    builder.encode(&mut out);

    let beads = BeadsSequence::new(out.as_slice(), &types);
    assert_eq!(beads.is_symmetrical(), true);
    assert_eq!(beads.len(), 5);

    let symb = beads.symmetric().ok().unwrap();
    assert_eq!(symb.len(), 5);
    assert_eq!(symb.get(0).to_int(), 1);
    assert_eq!(symb.get(1).to_int(), -4);
    assert_eq!(symb.get(2).to_int(), 6);
    assert_eq!(symb.get(3).to_int(), 0);
    assert_eq!(symb.get(4).to_int(), -9);
}

#[test]
fn symmetric_sequence_two_byte_numbers_3_types() {
    let types = BeadTypeSet::new(&[BeadType::U16, BeadType::I16, BeadType::F16]);
    let mut builder = BeadsSequenceBuilder::new(&types);
    let values = vec![1, -4, 6, 0, -9];
    for v in values.iter() {
        builder.push_int(*v);
    }
    let mut out = Vec::new();
    builder.encode(&mut out);

    let beads = BeadsSequence::new(out.as_slice(), &types);
    assert_eq!(beads.is_symmetrical(), true);
    assert_eq!(beads.len(), 5);

    let symb = beads.symmetric().ok().unwrap();
    assert_eq!(symb.len(), 5);
    assert_eq!(symb.get(0).to_int(), 1);
    assert_eq!(symb.get(1).to_int(), -4);
    assert_eq!(symb.get(2).to_int(), 6);
    assert_eq!(symb.get(3).to_int(), 0);
    assert_eq!(symb.get(4).to_int(), -9);
}

#[test]
fn symmetric_sequence_four_byte_numbers_3_types() {
    let types = BeadTypeSet::new(&[BeadType::U32, BeadType::I32, BeadType::F32]);
    let mut builder = BeadsSequenceBuilder::new(&types);
    let values = vec![1, -4, 6, 0, -9];
    for v in values.iter() {
        builder.push_int(*v);
    }
    let mut out = Vec::new();
    builder.encode(&mut out);

    let beads = BeadsSequence::new(out.as_slice(), &types);
    assert_eq!(beads.is_symmetrical(), true);
    assert_eq!(beads.len(), 5);

    let symb = beads.symmetric().ok().unwrap();
    assert_eq!(symb.len(), 5);
    assert_eq!(symb.get(0).to_int(), 1);
    assert_eq!(symb.get(1).to_int(), -4);
    assert_eq!(symb.get(2).to_int(), 6);
    assert_eq!(symb.get(3).to_int(), 0);
    assert_eq!(symb.get(4).to_int(), -9);
}

#[test]
fn symmetric_sequence_eight_byte_numbers_3_types() {
    let types = BeadTypeSet::new(&[BeadType::U64, BeadType::I64, BeadType::F64]);
    let mut builder = BeadsSequenceBuilder::new(&types);
    let values = vec![1, -4, 6, 0, -9];
    for v in values.iter() {
        builder.push_int(*v);
    }
    let mut out = Vec::new();
    builder.encode(&mut out);

    let beads = BeadsSequence::new(out.as_slice(), &types);
    assert_eq!(beads.is_symmetrical(), true);
    assert_eq!(beads.len(), 5);

    let symb = beads.symmetric().ok().unwrap();
    assert_eq!(symb.len(), 5);
    assert_eq!(symb.get(0).to_int(), 1);
    assert_eq!(symb.get(1).to_int(), -4);
    assert_eq!(symb.get(2).to_int(), 6);
    assert_eq!(symb.get(3).to_int(), 0);
    assert_eq!(symb.get(4).to_int(), -9);
}

#[test]
fn symmetric_sequence_sixteen_byte_numbers() {
    let types = BeadTypeSet::new(&[BeadType::U128, BeadType::I128]);
    let mut builder = BeadsSequenceBuilder::new(&types);
    let values = vec![1, -4, 6, 0, -9];
    for v in values.iter() {
        builder.push_int(*v);
    }
    let mut out = Vec::new();
    builder.encode(&mut out);

    let beads = BeadsSequence::new(out.as_slice(), &types);
    assert_eq!(beads.is_symmetrical(), true);
    assert_eq!(beads.len(), 5);

    let symb = beads.symmetric().ok().unwrap();
    assert_eq!(symb.len(), 5);
    assert_eq!(symb.get(0).to_int(), 1);
    assert_eq!(symb.get(1).to_int(), -4);
    assert_eq!(symb.get(2).to_int(), 6);
    assert_eq!(symb.get(3).to_int(), 0);
    assert_eq!(symb.get(4).to_int(), -9);
}

#[test]
fn symmetric_sequence_four_byte_numbers_3_types_100_values() {
    let types = BeadTypeSet::new(&[BeadType::U32, BeadType::I32, BeadType::F32]);
    let mut builder = BeadsSequenceBuilder::new(&types);
    for v in 0..100 {
        builder.push_int(v - 50);
    }
    let mut out = Vec::new();
    builder.encode(&mut out);

    let beads = BeadsSequence::new(out.as_slice(), &types);
    assert_eq!(beads.is_symmetrical(), true);
    assert_eq!(beads.len(), 100);

    let symb = beads.symmetric().ok().unwrap();
    assert_eq!(symb.len(), 100);
    for v in 0..100 {
        assert_eq!(symb.get(v).to_float(), (v as f64) - 50.0);
    }
}

#[test]
fn non_symmetric_sequence_one_byte_numbers() {
    let types = BeadTypeSet::new(&[BeadType::U8, BeadType::I8, BeadType::I16]);
    let mut builder = BeadsSequenceBuilder::new(&types);
    let values = vec![1, -4, 6, 0, -9];
    for v in values.iter() {
        builder.push_int(*v);
    }
    let mut out = Vec::new();
    builder.encode(&mut out);

    let beads = BeadsSequence::new(out.as_slice(), &types);
    assert_eq!(beads.is_symmetrical(), false);
    assert_eq!(beads.len(), 5);
}

#[test]
fn push_single_type_sequence() {
    let mut builder = BeadsSequenceBuilder::new(
        &BeadTypeSet::new(&[BeadType::U8])
    );

    builder.push_uint(20);
    builder.push_uint(21);
    builder.push_int(22);

    let mut out = Vec::new();
    builder.encode(&mut out);
    assert_eq!(out, vec![3, 20, 21, 22]);

    out.clear();
    builder.encode_with_types(&mut out);
    assert_eq!(out, vec![8, 0, 0, 0, 3, 20, 21, 22]);
}

#[test]
fn roundtrip_single_type_string_sequence() {
    let types = BeadTypeSet::new(&[BeadType::Utf8]);
    let mut builder = BeadsSequenceBuilder::new(&types);
    let values = vec!["Max", "Maxim", "Alex"];
    for v in values.iter() {
        builder.push_string(*v);
    }
    let mut out = Vec::new();
    builder.encode(&mut out);

    let beads = BeadsSequence::new(out.as_slice(), &types);
    assert_eq!(beads.is_symmetrical(), false);
    assert_eq!(beads.len(), 3);

    for (index, b) in beads.iter().enumerate() {
        assert_eq!(b.to_str(), values[index]);
    }
}

#[test]
fn roundtrip_single_type_int_sequence() {
    let types = BeadTypeSet::new(&[BeadType::I16]);
    let mut builder = BeadsSequenceBuilder::new(&types);
    for v in 0..100 {
        builder.push_int(v);
    }
    let mut out = Vec::new();
    builder.encode_with_types(&mut out);

    let beads = BeadsSequence::new_types_included(out.as_slice());
    assert_eq!(beads.is_symmetrical(), true);
    assert_eq!(beads.len(), 100);
    let symb = beads.symmetric().ok().unwrap();

    for index in 0..100 {
        assert_eq!(symb.get(index).to_int(), index as i128)
    }
}

#[test]
fn roundtrip_type_priority() {
    let types = BeadTypeSet::new(&[BeadType::U8, BeadType::I8]);
    let mut builder = BeadsSequenceBuilder::new(
        &types
    );
    let values = vec![-1, 20, 203, 204];

    for v in values.iter() {
        builder.push_int(*v);
    }

    let mut buffer: Vec<u8> = vec![];
    builder.encode(&mut buffer);

    let beads = BeadsSequence::new(buffer.as_slice(), &types);
    let out_values: Vec<i128> = beads.iter().map(|b|{b.to_int()}).collect();
    assert_eq!(out_values, values);
}

#[test]
fn roundtrip_type_priority_3_type() {
    let types = BeadTypeSet::new(&[BeadType::U8, BeadType::I8, BeadType::None]);
    let mut builder = BeadsSequenceBuilder::new(
        &types
    );
    let values = vec![-1, 20, 203, 204];

    for v in values.iter() {
        builder.push_int(*v);
    }

    let mut buffer: Vec<u8> = vec![];
    builder.encode(&mut buffer);

    let beads = BeadsSequence::new(buffer.as_slice(), &types);
    let out_values: Vec<i128> = beads.iter().map(|b|{b.to_int()}).collect();
    assert_eq!(out_values, values);
}

#[test]
fn roundtrip_type_priority_5_type() {
    let types = BeadTypeSet::new(&[BeadType::U8, BeadType::I8, BeadType::None, BeadType::TrueFlag, BeadType::FalseFlag]);
    let mut builder = BeadsSequenceBuilder::new(
        &types
    );
    let values = vec![-1, 20, 203, 204];

    for v in values.iter() {
        builder.push_int(*v);
    }

    let mut buffer: Vec<u8> = vec![];
    builder.encode(&mut buffer);

    let beads = BeadsSequence::new(buffer.as_slice(), &types);
    let out_values: Vec<i128> = beads.iter().map(|b|{b.to_int()}).collect();
    assert_eq!(out_values, values);
}

#[test]
fn roundtrip_type_priority_plus_unfit_value() {
    let types = BeadTypeSet::new(&[BeadType::U8, BeadType::I8]);
    let mut builder = BeadsSequenceBuilder::new(
        &types
    );
    let values = vec![-1, 403, 20, 204];

    for v in values.iter() {
        builder.push_int(*v);
    }

    let mut buffer: Vec<u8> = vec![];
    builder.encode(&mut buffer);

    let beads = BeadsSequence::new(buffer.as_slice(), &types);
    let out_values: Vec<i128> = beads.iter().map(|b|{b.to_int()}).collect();
    assert_eq!(out_values, vec![-1, 20, 204]);
}

#[test]
fn roundtrip_type_priority_plus_unfit_type() {
    let types = BeadTypeSet::new(&[BeadType::U8, BeadType::I8]);
    let mut builder = BeadsSequenceBuilder::new(
        &types
    );
    let values = vec![-1, 403, 20, 204];

    for v in values.iter() {
        builder.push_none();
        builder.push_int(*v);
    }

    let mut buffer: Vec<u8> = vec![];
    builder.encode(&mut buffer);

    let beads = BeadsSequence::new(buffer.as_slice(), &types);
    let out_values: Vec<i128> = beads.iter().map(|b|{b.to_int()}).collect();
    assert_eq!(out_values, vec![-1, 20, 204]);
}

#[test]
fn roundtrip_type_priority_5_types_plus_unfit_value() {
    let types = BeadTypeSet::new(&[BeadType::U8, BeadType::I8, BeadType::I16, BeadType::U16, BeadType::F16]);
    let mut builder = BeadsSequenceBuilder::new(
        &types
    );
    let values = vec![-1, 403, 20, 204];

    for v in values.iter() {
        builder.push_double(0.1);
        builder.push_int(*v);
    }

    let mut buffer: Vec<u8> = vec![];
    builder.encode(&mut buffer);

    let beads = BeadsSequence::new(buffer.as_slice(), &types);
    let out_values: Vec<i128> = beads.iter().map(|b|{b.to_int()}).collect();
    assert_eq!(out_values, values);
}

#[test]
fn roundtrip_push_double_with_accuracy() {
    let types = BeadTypeSet::new(&[BeadType::F16, BeadType::F32, BeadType::F64]);
    let mut builder = BeadsSequenceBuilder::new(
        &types
    );
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

    let beads = BeadsSequence::new(buffer.as_slice(), &types);
    let out_values: Vec<f64> = beads.iter().map(|b|{b.to_float()}).collect();
    assert_eq!(out_values, vec![0.1, 0.10000000149011612, 0.0999755859375]);
}

#[test]
fn indexed_beads_builder() {
    let mut builder = IndexedBeadsBuilder::new();
    builder.push(&[1, 2, 3, 4]);

    let mut buffer: Vec<u8> = vec![];
    builder.encode(&mut buffer);

    assert_eq!(buffer, &[8, 4, 1, 2, 3, 4]);

    builder.push(&[1, 7, 8, 5, 3, 4, 6, 7, 8, 5, 20]);

    buffer.clear();
    builder.encode(&mut buffer);

    assert_eq!(buffer, &[
        16,
        4, 15,
        1, 2, 3, 4,
        1, 7, 8, 5, 3, 4, 6, 7, 8, 5, 20
    ]);
}

#[test]
fn roundtrip_indexed_beads_builder() {
    let mut builder = IndexedBeadsBuilder::new();
    builder.push(&[1, 2, 3, 4]);

    builder.push(&[1, 7, 8, 5, 3, 4, 6, 7, 8, 5, 20]);

    let mut buffer: Vec<u8> = vec![];
    builder.encode(&mut buffer);

    let ib = IndexedBeads::new(buffer.as_slice());

    assert_eq!(ib.len(), 2);
    assert_eq!(ib[0].to_vec(), vec![1, 2, 3, 4]);
    assert_eq!(ib[1].to_vec(), vec![1, 7, 8, 5, 3, 4, 6, 7, 8, 5, 20]);
}

#[test]
fn roundtrip_large_indexed_beads_builder() {
    let mut builder = IndexedBeadsBuilder::new();
    let number_of_beads = 100_000;
    for _ in 0..number_of_beads {
        builder.push(&[1, 2, 3, 4]);
    }

    let mut buffer: Vec<u8> = vec![];
    builder.encode(&mut buffer);

    let ib = IndexedBeads::new(buffer.as_slice());

    assert_eq!(ib.len(), number_of_beads);
    assert_eq!(ib[number_of_beads / 2].to_vec(), vec![1, 2, 3, 4]);
}

#[test]
fn roundtrip_fixed_size_beads() {
    let mut builder = FixedSizeBeadsBuilder::new(3);
    builder.push(&[1, 2, 3]);
    builder.push(&[10, 20, 30]);
    builder.push(&[30, 50, 90]);
    builder.push(&[130, 150, 190]);

    let mut buffer: Vec<u8> = vec![];
    builder.encode(&mut buffer);

    assert_eq!(buffer, vec![3, 1, 2, 3, 10, 20, 30, 30, 50, 90, 130, 150, 190]);

    let fs_beads = FixedSizeBeads::new(&buffer);
    assert_eq!(fs_beads.len(), 4);

    assert_eq!(fs_beads[0].to_vec(), vec![1, 2, 3]);
    assert_eq!(fs_beads[1].to_vec(), vec![10, 20, 30]);
    assert_eq!(fs_beads[2].to_vec(), vec![30, 50, 90]);
    assert_eq!(fs_beads[3].to_vec(), vec![130, 150, 190]);
}

#[test]
fn roundtrip_fixed_size_beads_with_incremental_uint_builder() {
    let mut builder = FixedSizeBeadsIncrementalUintBuilder::new();
    builder.push(1);
    builder.push(2);
    builder.push(20);
    builder.push(205);

    let mut buffer: Vec<u8> = vec![];
    builder.encode(&mut buffer);

    assert_eq!(buffer, vec![1, 1, 2, 20, 205]);

    let fs_beads = FixedSizeBeads::new(&buffer);
    assert_eq!(fs_beads.len(), 4);

    assert_eq!(fs_beads[0].to_vec(), vec![1]);
    assert_eq!(fs_beads[1].to_vec(), vec![2]);
    assert_eq!(fs_beads[2].to_vec(), vec![20]);
    assert_eq!(fs_beads[3].to_vec(), vec![205]);


    builder.push(340);

    let mut buffer: Vec<u8> = vec![];
    builder.encode(&mut buffer);

    assert_eq!(buffer, vec![2, 1, 0, 2, 0, 20, 0, 205, 0, 84, 1]);

    let fs_beads = FixedSizeBeads::new(&buffer);
    assert_eq!(fs_beads.len(), 5);

    assert_eq!(fs_beads[0].to_vec(), vec![1, 0]);
    assert_eq!(fs_beads[1].to_vec(), vec![2, 0]);
    assert_eq!(fs_beads[2].to_vec(), vec![20, 0]);
    assert_eq!(fs_beads[3].to_vec(), vec![205, 0]);
    assert_eq!(fs_beads[4].to_vec(), vec![84, 1]);
}

#[test]
fn roundtrip_dedup_f64() {
    let mut builder = BeadsSequenceBuilder::new(&BeadTypeSet::new(&[BeadType::F64]));
    builder.push_double(0.1);
    builder.push_double(0.1);
    builder.push_double(0.3);
    builder.push_double(0.2);
    builder.push_double(0.3);

    let mut buffer: Vec<u8> = vec![];
    builder.encode(&mut buffer);

    assert_eq!(buffer, vec![5, 154, 153, 153, 153, 153, 153, 185, 63, 154, 153, 153, 153, 153, 153, 185, 63, 51, 51, 51, 51, 51, 51, 211, 63, 154, 153, 153, 153, 153, 153, 201, 63, 51, 51, 51, 51, 51, 51, 211, 63]);

    let mut dedup_buffer = vec![];

    beads_to_dedup_beads(&buffer, &BeadTypeSet::new(&[BeadType::F64]), &mut dedup_buffer);

    assert_eq!(dedup_buffer, vec![16, 6, 34, 1, 0, 0, 1, 2, 1, 24, 8, 16, 24, 154, 153, 153, 153, 153, 153, 185, 63, 51, 51, 51, 51, 51, 51, 211, 63, 154, 153, 153, 153, 153, 153, 201, 63]);

    let dedup = DedupBeads::new(dedup_buffer.as_slice());
    assert_eq!(dedup.len(), 5);
    assert_eq!(dedup.get(0), <f64>::to_le_bytes(0.1).to_vec());
    assert_eq!(dedup.get(1), <f64>::to_le_bytes(0.1).to_vec());
    assert_eq!(dedup.get(2), <f64>::to_le_bytes(0.3).to_vec());
    assert_eq!(dedup.get(3), <f64>::to_le_bytes(0.2).to_vec());
    assert_eq!(dedup.get(4), <f64>::to_le_bytes(0.3).to_vec());
}