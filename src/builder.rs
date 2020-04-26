use std::collections::HashMap;
use crate::bead_type::{BeadType, BeadTypeSet};
use std::cmp::max;
use crate::vlq::add_as_vlq;
use std::io;

pub struct BeadsSequenceBuilder {
    buffer: Vec<u8>,
    count: usize,
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
                    return true;
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
                    return true;
                }
            }
        }
        false
    }

    pub fn encode<W>(&self, writer: &mut W) where W: io::Write {
        let mut tmp = [0; 10];
        let count_length = add_as_vlq(tmp.as_mut(), self.count as u128);
        writer.write_all(tmp[..count_length].as_ref()).expect("could not write");
        let buffer_length = max(self.data_pointer, self.flag_pointer+1);
        writer.write_all(self.buffer[..buffer_length].as_ref()).expect("could not write");
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
        if self.count > 0 && position_in_byte == 0 {
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