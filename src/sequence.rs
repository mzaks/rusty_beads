use crate::iterator::BeadsIterator;
use crate::bead_type::{BeadType, BeadTypeSet};
use crate::vlq::read_vlq;
use std::borrow::Borrow;
use crate::reference::BeadReference;
use std::ops::Index;
use crate::converters::u128_from_slice;

pub struct TypedBeads<'a> {
    buffer: &'a[u8],
    count: usize,
    types: Vec<BeadType>,
}

impl<'a> TypedBeads<'a> {
    pub fn new(buffer: &'a[u8], types: &'_ BeadTypeSet) -> Result<TypedBeads<'a>, &'static str> {
        if types.size() < 1 || types.size() > 16 {
            return Err("Beads sequence can carry only 1..=16 types");
        }
        let mut _types = vec![];
        for t in BeadType::cases_by_priority() {
            if types.contains(&t) {
                _types.push(t)
            }
        }

        let (vlq_len, count) = read_vlq(buffer)?;

        if buffer.len() <= vlq_len {
            return Err("Buffer is invalid due to its insufficient length");
        }

        Ok(TypedBeads {
            buffer: buffer[vlq_len..].as_ref(),
            count: count as usize,
            types: _types
        })
    }

    pub fn new_types_included(buffer: &'a[u8]) -> Result<TypedBeads<'a>, &'static str> {
        if buffer.len() < 5 {
            return Err("Buffer can't be a valid TypedBead including types as it is too small");
        }
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

    pub fn symmetric(&self) -> Result<SymmetricTypedBeads, &'static str> {
        if self.is_symmetrical() {
            Ok(SymmetricTypedBeads {
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

pub struct SymmetricTypedBeads<'a> {
    buffer: &'a[u8],
    count: usize,
    types: Vec<BeadType>,
    data_size: usize,
    tags_per_byte: usize,
    tag_mask: u8
}

impl <'a> SymmetricTypedBeads<'a> {
    pub fn get(&self, index: usize) -> Result<BeadReference<'a>, &'static str> {
        if index >= self.count {
            return Err("Index is out of bounds");
        }
        if self.types.len() == 1 {
            let bead_type = self.types[0];
            let data_start = index * self.data_size;
            Ok(BeadReference {
                value: 0,
                buffer: self.buffer[data_start..(data_start + self.data_size)].as_ref(),
                bead_type
            })
        } else {
            let tag_index = index % self.tags_per_byte;
            let number_of_tag_bytes = index / self.tags_per_byte;
            let number_of_data_bytes = self.data_size * self.tags_per_byte * number_of_tag_bytes;
            let tag = self.buffer.get(number_of_tag_bytes + number_of_data_bytes).ok_or("Bad buffer")?;
            let shift = tag_index * 8 / self.tags_per_byte;
            let mask = self.tag_mask << shift as u8;
            let type_index = ((tag & mask) as usize) >> shift;
            let bead_type = self.types.get(type_index).ok_or("Bad buffer")?;
            let tag_addition = if bead_type.has_no_data() { 0 } else { 1 };
            let data_start = number_of_tag_bytes + tag_addition + index * self.data_size;
            if self.buffer.len() < data_start + self.data_size {
                return Err("Bad buffer");
            }
            Ok(BeadReference {
                value: 0,
                buffer: self.buffer[data_start..(data_start + self.data_size)].as_ref(),
                bead_type: *bead_type
            })
        }
    }

    pub fn len(&self) -> usize {
        self.count
    }
}

pub struct IndexedBeads<'a> {
    index_buffer: &'a[u8],
    value_buffer: &'a[u8],
    count: usize,
    bytes_per_index_entry: usize
}

impl<'a> IndexedBeads<'a> {
    pub fn new(buffer: &'a[u8])-> Result<IndexedBeads<'a>, &'static str> {
        let (header_size, header) = read_vlq(buffer)?;
        let count = (header >> 3) as usize;
        let bytes_per_index = ((header & 7) + 1) as usize;
        if buffer.len() <= header_size + count * bytes_per_index {
            return Err("Bad buffer");
        }
        Ok(IndexedBeads {
            index_buffer: buffer[header_size..(header_size + count * bytes_per_index)].as_ref(),
            value_buffer: buffer[(header_size + count * bytes_per_index)..].as_ref(),
            count,
            bytes_per_index_entry: bytes_per_index
        })
    }

    pub fn len(&self) -> usize { self.count }

    pub fn get(&'a self, index: usize) -> Result<&'a[u8], &'static str> {
        if index >= self.count {
            return Err("Bad index")
        }

        fn position(b: &[u8], index: usize, bytes_per_index_entry: usize) -> Result<usize, &'static str> {
            let mut position = 0;
            for i in 0..bytes_per_index_entry {
                let p = b.get(index*bytes_per_index_entry + i).ok_or("Bad index")?;
                let part = *p as usize;
                position |= part << (i * 8)
            }
            Ok(position)
        }

        let start = if index == 0 {
            0
        } else {
            position(self.index_buffer, index - 1, self.bytes_per_index_entry)?
        };

        let end  = position(self.index_buffer, index, self.bytes_per_index_entry)?;

        if self.value_buffer.len() < end {
            return Err("Bad index")
        }

        Ok(&self.value_buffer[start..end])
    }
}

impl Index<usize> for IndexedBeads<'_> {
    type Output = [u8];

    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.count {
            panic!("bad index")
        }

        fn position(b: &[u8], index: usize, bytes_per_index_entry: usize) -> usize {
            let mut position = 0;
            for i in 0..bytes_per_index_entry {
                let part = b[index*bytes_per_index_entry + i] as usize;
                position |= part << (i * 8)
            }
            position
        }

        let start = if index == 0 {
            0
        } else {
            position(self.index_buffer, index - 1, self.bytes_per_index_entry)
        };

        let end  = position(self.index_buffer, index, self.bytes_per_index_entry);

        &self.value_buffer[start..end]
    }
}

pub struct FixedSizeBeads<'a> {
    size: usize,
    buffer: &'a [u8]
}

impl <'a> FixedSizeBeads<'a> {
    pub fn new(buffer: &'a[u8]) -> Result<FixedSizeBeads, &'static str> {
        let (header_size, header) = read_vlq(buffer)?;
        Ok(FixedSizeBeads {
            size: header as usize,
            buffer: &buffer[header_size..]
        })
    }

    pub fn len(&self) -> usize {
        self.buffer.len() / self.size
    }

    pub fn get(&self, index: usize) -> Result<&'a[u8], &'static str> {
        let start = index * self.size;
        let end = (index + 1) * self.size;

        if self.buffer.len() < end {
            return Err("Bad index")
        }
        Ok(&self.buffer[start..end])
    }
}

impl Index<usize> for FixedSizeBeads<'_> {
    type Output = [u8];

    fn index(&self, index: usize) -> &Self::Output {
        let start = index * self.size;
        let end = (index + 1) * self.size;

        &self.buffer[start..end]
    }
}

pub struct DedupBeads<'a> {
    buffer: &'a[u8]
}

impl <'a> DedupBeads<'a> {
    pub fn new(buffer: &'a[u8]) -> DedupBeads<'a> {
        DedupBeads {
            buffer
        }
    }

    pub fn len(&self) -> Result<usize, &'static str> {
        let root = IndexedBeads::new(self.buffer)?;
        let index_beads = FixedSizeBeads::new(root.get(0)?)?;
        Ok(index_beads.len())
    }

    pub fn get(&self, index: usize) -> Result<Vec<u8>, &'static str> {
        let root = IndexedBeads::new(self.buffer)?;
        let index_beads = FixedSizeBeads::new(root.get(0)?)?;
        let values = IndexedBeads::new(root.get(1)?)?;
        let index = u128_from_slice(&index_beads.get(index)?) as usize;
        Ok(values.get(index)?.to_vec())
    }
}