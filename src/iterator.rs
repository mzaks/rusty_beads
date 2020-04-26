use std::convert::AsRef;
use crate::bead_type::BeadType;
use std::cmp::max;
use crate::vlq::read_vlq;
use crate::reference::BeadReference;

pub struct BeadsIterator<'a> {
    buffer: &'a[u8],
    types: &'a Vec<BeadType>,
    count: usize,
    index: usize,
    tag_cursor: usize,
    data_cursor: usize,
    tags_per_byte: usize,
    tag_mask: u8,
}

impl <'a> Iterator for BeadsIterator<'a> {
    type Item = BeadReference<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.count {
            return None;
        }

        let bead_type = if self.types.len() == 1 {self.types[0]} else {self.get_type()};
        let tag_addition = if bead_type.has_no_data() { 0 } else { 1 };
        let mut start = if self.types.len() == 1 {self.data_cursor} else {max(self.data_cursor, self.tag_cursor + tag_addition)};
        let (data_length, data_value) = Self::get_data_length_and_value(self.buffer[start..].as_ref(), bead_type);
        self.data_cursor = start + data_length;
        if bead_type == BeadType::Utf8 || bead_type == BeadType::Bytes {
            self.data_cursor += data_value as usize;
            start += data_length;
        }
        self.index += 1;
        Some(BeadReference {
            value: data_value,
            buffer: self.buffer[start..self.data_cursor].as_ref(),
            bead_type
        })
    }
}

impl <'a> BeadsIterator <'_> {
    pub(crate) fn new (buffer: &'a [u8], count: usize, types: &'a Vec<BeadType>) -> BeadsIterator<'a> {
        let mask = match types.len() {
            1..=2 => 1,
            3..=4 => 3,
            _ => 15
        };
        BeadsIterator {
            buffer,
            types,
            count,
            index: 0,
            tag_cursor: 0,
            data_cursor: 0,
            tag_mask: mask,
            tags_per_byte: match types.len() {
                1..=2 => 8,
                3..=4 => 4,
                _ => 2
            }
        }
    }

    fn get_type(&mut self) -> BeadType {
        let tag_index = self.index % self.tags_per_byte;
        if self.index > 0 && tag_index == 0 {
            self.tag_cursor = max(self.tag_cursor + 1, self.data_cursor);
        }
        let tag = self.buffer[self.tag_cursor];
        let shift = tag_index * 8 / self.tags_per_byte;
        let mask = self.tag_mask << shift as u8;
        let type_index = ((tag & mask) as usize) >> shift;
        self.types[type_index]
    }

    fn get_data_length_and_value(buffer: &[u8], bead_type: BeadType) -> (usize, u128) {
        return match bead_type {
            BeadType::None | BeadType::TrueFlag | BeadType::FalseFlag => (0, 0),
            BeadType::U8 | BeadType::I8 => (1, 0),
            BeadType::U16 | BeadType::I16 | BeadType::F16 => (2, 0),
            BeadType::U32 | BeadType::I32 | BeadType::F32 => (4, 0),
            BeadType::U64 | BeadType::I64 | BeadType::F64 => (8, 0),
            BeadType::U128 | BeadType::I128 => (8, 0),
            BeadType::Vlq | BeadType::VlqZ | BeadType::Utf8 | BeadType::Bytes => read_vlq(buffer),
        }
    }
}