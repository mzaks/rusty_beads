use std::collections::HashMap;
use crate::bead_type::{BeadType, BeadTypeSet};
use std::cmp::max;
use crate::vlq::{add_as_vlq, VlqByteSize};
use std::io;
use std::cell::{RefCell, RefMut};
use std::borrow::{BorrowMut};

pub trait BeadsBuilder {
    fn encode<'a>(&self, writer: RefMut<dyn io::Write + 'a>);
    fn len(&self) -> usize;
}

pub struct BeadsSequenceBuilder {
    buffer: Vec<u8>,
    count: usize,
    flag_count: usize,
    flag_pointer: usize,
    data_pointer: usize,
    type_index: HashMap<BeadType, u8>,
}

impl BeadsSequenceBuilder {
    pub fn new(types: &BeadTypeSet) -> BeadsSequenceBuilder {
        if types.size() < 1 || types.size() > 16 {
            panic!("Beads sequence can carry only 1..=16 types");
        }
        let mut type_index = HashMap::new();
        let mut index = 0u8;
        for t in BeadType::cases_by_priority() {
            if types.contains(&t) {
                type_index.insert(t, index);
                index += 1;
            }
        }

        BeadsSequenceBuilder {
            buffer: vec![0; 1000],
            count: 0,
            flag_count: 0,
            flag_pointer: 0,
            data_pointer: 0,
            type_index
        }
    }

    pub fn push_bool(&mut self, value: bool) -> bool {
        let mut type_index = 255u8;
        if value {
            if let Some(_type_index) = self.type_index.get(&BeadType::TrueFlag) {
                type_index = *_type_index;
            }
        } else {
            if let Some(_type_index) = self.type_index.get(&BeadType::FalseFlag) {
                type_index = *_type_index;
            }
        }
        if type_index != 255 {
            self.add_flag(type_index);
            self.count += 1;
            return true;
        }
        return false;
    }

    pub fn push_none(&mut self) -> bool {
        let mut type_index = 255u8;
        if let Some(_type_index) = self.type_index.get(&BeadType::None) {
            type_index = *_type_index;
        }

        if type_index != 255 {
            self.add_flag(type_index);
            self.count += 1;
            return true;
        }
        return false;
    }

    pub fn push_string(&mut self, value: &str) -> bool {
        let mut type_index = 255u8;
        if let Some(_type_index) = self.type_index.get(&BeadType::Utf8) {
            type_index = *_type_index;
        }
        if type_index != 255 {
            self.add_flag(type_index);
            let bytes = value.as_bytes();
            let start = self.data_start();
            self.grow_buffer_if_needed(start, bytes.len() + 10);
            let vlq_len = add_as_vlq(self.buffer[start..].as_mut(), bytes.len() as u128);
            self.buffer[start+vlq_len..start+vlq_len+bytes.len()].as_mut().copy_from_slice(bytes);
            self.data_pointer = start + vlq_len + bytes.len();
            self.count += 1;
            return true;
        }
        return false;
    }

    pub fn push_bytes(&mut self, value: &[u8]) -> bool {
        let mut type_index = 255u8;
        if let Some(_type_index) = self.type_index.get(&BeadType::Bytes) {
            type_index = *_type_index;
        }
        if type_index != 255 {
            self.add_flag(type_index);
            let bytes = value;
            let start = self.data_start();
            self.grow_buffer_if_needed(start, bytes.len() + 10);
            let vlq_len = add_as_vlq(self.buffer[start..].as_mut(), bytes.len() as u128);
            self.buffer[start+vlq_len..start+vlq_len+bytes.len()].as_mut().copy_from_slice(bytes);
            self.data_pointer = start + vlq_len + bytes.len();
            self.count += 1;
            return true;
        }
        return false;
    }

    pub fn push_uint(&mut self, value: u128) -> bool {
        let start = max(self.flag_pointer+1, self.data_pointer);
        self.grow_buffer_if_needed(start, 16);
        for t in BeadType::cases_for_uint() {
            let mut type_index = 255u8;
            if let Some(_type_index) = self.type_index.get(&t) {
                type_index = *_type_index;
            }
            if type_index != 255 {
                self.add_flag(type_index);
                let start = self.data_start();
                let (added, len) = t.push_uint(value, self.buffer[start..].as_mut());
                if added {
                    self.data_pointer = start + len;
                    self.count += 1;
                    self.flag_count = self.count;
                    return true;
                } else {
                    self.reset_flag();
                }
            }
        }
        false
    }

    pub fn push_int(&mut self, value: i128) -> bool {
        let start = max(self.flag_pointer+1, self.data_pointer);
        self.grow_buffer_if_needed(start, 16);
        for t in BeadType::cases_for_int() {
            let mut type_index = 255u8;
            if let Some(_type_index) = self.type_index.get(&t) {
                type_index = *_type_index;
            }
            if type_index != 255 {
                self.add_flag(type_index);
                let start = self.data_start();
                let (added, len) = t.push_int(value, self.buffer[start..].as_mut());
                if added {
                    self.data_pointer = start + len;
                    self.count += 1;
                    return true;
                } else {
                    self.reset_flag();
                }
            }
        }
        false
    }

    pub fn push_double(&mut self, value: f64) -> bool {
        self.push_double_with_accuracy(value, 0.0)
    }

    pub fn push_double_with_accuracy(&mut self, value: f64, accuracy: f64) -> bool {
        let start = max(self.flag_pointer+1, self.data_pointer);
        self.grow_buffer_if_needed(start, 8);
        for t in BeadType::cases_for_double() {
            let mut type_index = 255u8;
            if let Some(_type_index) = self.type_index.get(&t) {
                type_index = *_type_index;
            }
            if type_index != 255 {
                self.add_flag(type_index);
                let start = self.data_start();
                let (added, len) = t.push_double(value, accuracy, self.buffer[start..].as_mut());
                if added {
                    self.data_pointer = start + len;
                    self.count += 1;
                    self.flag_count = self.count;
                    return true;
                } else {
                    self.reset_flag();
                }
            }
        }
        false
    }

    pub fn encode<W>(&self, writer: &mut W) where W: io::Write {
        <dyn BeadsBuilder>::encode(self, RefCell::new(writer).borrow_mut());
    }

    pub fn encode_with_types<W>(&self, writer: &mut W) where W: io::Write {
        let keys: Vec<BeadType> = self.type_index.keys().map(|k| k.clone()).collect();
        let type_set = BeadTypeSet::new(keys.as_slice());

        writer.write_all(type_set.bytes().as_ref()).expect("could not write");
        self.encode(writer);
    }

    fn add_flag(&mut self, flag: u8) {
        if self.type_index.len() == 1 {
            return;
        }
        let (position_in_byte, shift) = self.compute_flag_info();
        self.move_flag_pointer_if_necessary(position_in_byte);
        self.grow_buffer_if_needed(self.flag_pointer, 1);
        self.buffer[self.flag_pointer] |= flag << (position_in_byte * shift) as u8;
        self.flag_count = self.count + 1;
    }

    fn reset_flag(&mut self) {
        let (position_in_byte, shift) = self.compute_flag_info();
        let reset_mask = if position_in_byte == 0 {0} else {255u8 >> (8 - position_in_byte * shift) as u8};
        self.buffer[self.flag_pointer] &= reset_mask;
    }

    fn compute_flag_info(&self) -> (usize, usize) {
        let type_number = self.type_index.len();
        if type_number <= 2 {
            (self.count & 7, 1)
        }  else if type_number <= 4 {
            (self.count & 3, 2)
        } else {
            (self.count & 1, 4)
        }
    }

    fn move_flag_pointer_if_necessary(&mut self, position_in_byte: usize) {
        if self.flag_count > 0 && self.flag_count == self.count && position_in_byte == 0 {
            self.flag_pointer = max(self.flag_pointer + 1, self.data_pointer);
        }
    }

    fn grow_buffer_if_needed(&mut self, offset: usize, size: usize) {
        while self.buffer.len() < offset + size {
            self.buffer.resize(self.buffer.len() >> 1, 0)
        }
    }

    fn data_start(&self) -> usize {
        let flag_pointer_additive = if self.type_index.len() == 1 { 0 } else { 1 };
        max(self.flag_pointer+flag_pointer_additive, self.data_pointer)
    }
}

impl BeadsBuilder for BeadsSequenceBuilder {
    fn encode(&self, mut writer: RefMut<dyn io::Write + '_>) {
        let mut tmp = [0; 10];
        let count_length = add_as_vlq(tmp.as_mut(), self.count as u128);
        writer.borrow_mut().write_all(tmp[..count_length].as_ref()).expect("could not write");
        let buffer_length = max(self.data_pointer, self.flag_pointer+1);
        writer.borrow_mut().write_all(self.buffer[..buffer_length].as_ref()).expect("could not write");
    }

    fn len(&self) -> usize {
        let leading_zeros = self.count.leading_zeros();
        let count_length = max(1, (9 - (leading_zeros / 7)) as usize);
        let buffer_length = max(self.data_pointer, self.flag_pointer+1) as usize;
        count_length + buffer_length
    }
}

pub struct IndexedBeadsBuilder <'a> {
    indexes: Vec<u8>,
    buffers: Vec<&'a[u8]>,
    cursor: u64
}

impl <'a> IndexedBeadsBuilder <'a>{
    pub fn new() -> IndexedBeadsBuilder<'a> {
        IndexedBeadsBuilder {
            indexes: vec![],
            buffers: vec![],
            cursor: 0
        }
    }

    pub fn push(&mut self, buffer: &'a [u8]) {
        self.buffers.push(buffer);
        self.cursor += buffer.len() as u64;
        let bytes = self.cursor.to_le_bytes();
        let mut size_as_buf: Vec<u8> = bytes.to_vec();
        self.indexes.append(size_as_buf.as_mut());
    }

    pub fn encode<W>(&self, writer: &mut W) where W: io::Write {
        <dyn BeadsBuilder>::encode(self, RefCell::new(writer).borrow_mut());
    }

    pub fn encode_from_beads_builders<W>(writer: &mut W, builders: Vec<Box<dyn BeadsBuilder + '_>>) where W: io::Write {
        if builders.is_empty() {
            return
        }
        let cursor: usize = builders.iter().map(|b|b.len()).sum();
        let bytes_per_index_entry = (8 - cursor.leading_zeros() / 8) as usize;

        let header = ((builders.len() as u128) << 3) | ((bytes_per_index_entry - 1) as u128);

        let mut tmp = [0; 19];
        let count_length = add_as_vlq(tmp.as_mut(), header);
        writer.write_all(tmp[..count_length].as_ref()).expect("could not write");
        let mut cursor = 0;
        for b in builders.iter() {
            cursor += b.len();
            let bytes = cursor.to_le_bytes();
            writer.write_all(&bytes[..bytes_per_index_entry]).expect("could not write");
        }
        let writer_cell = RefCell::new(writer);
        for b in builders.iter() {
            b.encode(writer_cell.borrow_mut());
        }
    }
}

impl BeadsBuilder for IndexedBeadsBuilder<'_> {
    fn encode(&self, mut writer: RefMut<dyn io::Write + '_>) {
        if self.cursor == 0 {
            return
        }

        let bytes_per_index_entry = (8 - self.cursor.leading_zeros() / 8) as usize;

        let header = ((self.buffers.len() as u128) << 3) | ((bytes_per_index_entry - 1) as u128);

        let mut tmp = [0; 19];
        let count_length = add_as_vlq(tmp.as_mut(), header);
        writer.borrow_mut().write_all(tmp[..count_length].as_ref()).expect("could not write");
        for i in (0..self.indexes.len()).step_by(8) {
            writer.borrow_mut().write_all(&self.indexes[i..i+bytes_per_index_entry]).expect("could not write");
        }
        for b in self.buffers.iter() {
            writer.borrow_mut().write_all(b).expect("could not write");
        }
    }

    fn len(&self) -> usize {
        let bytes_per_index_entry = (8 - self.cursor.leading_zeros() / 8) as usize;
        let count_length = (self.buffers.len() + 3).vlq_byte_size();
        let index_bytes = ((self.indexes.len() / 8) * bytes_per_index_entry) as usize;
        let values_bytes: usize = self.buffers.iter().map(|b| b.len()).sum();
        count_length + index_bytes + values_bytes
    }
}

pub struct FixedSizeBeadsBuilder {
    size: usize,
    buffer: Vec<u8>
}

impl FixedSizeBeadsBuilder {
    pub fn new(bead_size: usize) -> FixedSizeBeadsBuilder {
        FixedSizeBeadsBuilder {
            size: bead_size,
            buffer: vec![]
        }
    }

    pub fn push(&mut self, value: &[u8]) {
        if value.len() != self.size {
            panic!(format!("Value {:?} is not of fix size {}", value, self.size));
        }
        self.buffer.append(value.to_vec().as_mut());
    }

    pub fn encode<W>(&self, writer: &mut W) where W: io::Write {
        <dyn BeadsBuilder>::encode(self, RefCell::new(writer).borrow_mut());
    }
}

impl BeadsBuilder for FixedSizeBeadsBuilder {
    fn encode(&self, mut writer: RefMut<dyn io::Write + '_>) {
        let mut tmp = [0; 10];
        let count_length = add_as_vlq(tmp.as_mut(), self.size as u128);
        writer.borrow_mut().write_all(tmp[..count_length].as_ref()).expect("could not write");
        writer.borrow_mut().write_all(self.buffer.as_slice()).expect("could not write");
    }

    fn len(&self) -> usize {
        let count_length = self.size.vlq_byte_size();
        count_length + self.buffer.len()
    }
}

pub struct FixedSizeBeadsIncrementalUintBuilder {
    size: usize,
    buffer: Vec<u8>
}

impl FixedSizeBeadsIncrementalUintBuilder {
    pub fn new() -> FixedSizeBeadsIncrementalUintBuilder {
        FixedSizeBeadsIncrementalUintBuilder {
            size: 0,
            buffer: vec![]
        }
    }

    pub fn push(&mut self, value: u128) {
        let bytes_per_entry = (16 - value.leading_zeros() / 8) as usize;
        self.size = max(self.size, bytes_per_entry);
        self.buffer.append(value.to_le_bytes().to_vec().as_mut());
    }

    pub fn encode<W>(&self, writer: &mut W) where W: io::Write {
        <dyn BeadsBuilder>::encode(self, RefCell::new(writer).borrow_mut());
    }
}

impl BeadsBuilder for FixedSizeBeadsIncrementalUintBuilder {
    fn encode(&self, mut writer: RefMut<dyn io::Write + '_>) {
        let mut tmp = [0; 10];
        let b_writer = writer.borrow_mut();
        let count_length = add_as_vlq(tmp.as_mut(), self.size as u128);
        b_writer.write_all(tmp[..count_length].as_ref()).expect("could not write");
        for i in (0..self.buffer.len()).step_by(16) {
            b_writer.write_all(&self.buffer[i..i+self.size]).expect("could not write");
        }
    }
    fn len(&self) -> usize {
        let count_length = self.size.vlq_byte_size();
        count_length + (self.buffer.len() / 16) * self.size
    }
}