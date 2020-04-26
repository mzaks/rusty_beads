use crate::iterator::BeadsIterator;
use crate::bead_type::{BeadType, BeadTypeSet};
use crate::vlq::read_vlq;
use std::borrow::Borrow;
use crate::reference::BeadReference;

pub struct BeadsSequence<'a> {
    buffer: &'a[u8],
    count: usize,
    types: Vec<BeadType>,
}

impl<'a> BeadsSequence<'a> {
    pub fn new(buffer: &'a[u8], types: &'_ BeadTypeSet) -> BeadsSequence<'a> {
        if types.size() < 1 || types.size() > 16 {
            panic!("Beads sequence can carry only 1..=16 types");
        }
        let mut _types = vec![];
        for t in BeadType::cases_by_priority() {
            if types.contains(&t) {
                _types.push(t)
            }
        }

        let (vlq_len, count) = read_vlq(buffer);

        BeadsSequence {
            buffer: buffer[vlq_len..].as_ref(),
            count: count as usize,
            types: _types
        }
    }

    pub fn new_types_included(buffer: &'a[u8]) -> BeadsSequence<'a> {
        let types_value = u32::from_le_bytes(BeadReference::clone_into_array(&buffer[..4]));
        Self::new(&buffer[4..], (BeadTypeSet::from(types_value)).borrow())
    }

    pub fn iter(&self) -> BeadsIterator {
        BeadsIterator::new(self.buffer, self.count, self.types.borrow())
    }

    pub fn len(&self) -> usize {self.count}

    pub fn is_symmetrical(&self) -> bool {
        if let Some(t1) = self.types.first() {
            let t1_data_size = t1.data_size();
            for t in self.types.iter() {
                let t_data_size = t.data_size();
                if t_data_size == 255 || t_data_size != t1_data_size {
                    return false
                }
            }
            return true
        }
        false
    }

    pub fn symmetric(&self) -> Result<SymmetricBeadsSequence, &'static str> {
        if self.is_symmetrical() {
            Ok(SymmetricBeadsSequence{
                buffer: self.buffer,
                count: self.count,
                types: self.types.clone(),
                data_size: self.types.first().unwrap().data_size() as usize,
                tags_per_byte: match self.types.len() {
                    1..=2 => 8,
                    3..=4 => 4,
                    _ => 2
                },
                tag_mask: match self.types.len() {
                    1..=2 => 1,
                    3..=4 => 3,
                    _ => 15
                }
            })
        } else {
            Err("Beads sequence is not symmetrical")
        }
    }
}

pub struct SymmetricBeadsSequence<'a> {
    buffer: &'a[u8],
    count: usize,
    types: Vec<BeadType>,
    data_size: usize,
    tags_per_byte: usize,
    tag_mask: u8
}

impl <'a> SymmetricBeadsSequence<'a> {
    pub fn get(&self, index: usize) -> BeadReference<'a> {
        if index >= self.count {
            panic!("Index: {} is out of bounds. Sequence size is {}", index, self.count);
        }
        if self.types.len() == 1 {
            let bead_type = self.types[0];
            let data_start = index * self.data_size;
            BeadReference {
                value: 0,
                buffer: self.buffer[data_start..(data_start + self.data_size)].as_ref(),
                bead_type
            }
        } else {
            let tag_index = index % self.tags_per_byte;
            let number_of_tag_bytes = index / self.tags_per_byte;
            let number_of_data_bytes = self.data_size * self.tags_per_byte * number_of_tag_bytes;
            let tag = self.buffer[number_of_tag_bytes + number_of_data_bytes];
            let shift = tag_index * 8 / self.tags_per_byte;
            let mask = self.tag_mask << shift as u8;
            let type_index = ((tag & mask) as usize) >> shift;
            let bead_type = self.types[type_index];
            let tag_addition = if bead_type.has_no_data() { 0 } else { 1 };
            let data_start = number_of_tag_bytes + tag_addition + index * self.data_size;
            BeadReference {
                value: 0,
                buffer: self.buffer[data_start..(data_start + self.data_size)].as_ref(),
                bead_type
            }
        }

    }

    pub fn len(&self) -> usize {
        self.count
    }
}