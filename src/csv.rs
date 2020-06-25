use crate::converters::CsvFirstLine;
use std::io;
use crate::builder::{BeadsBuilder, IndexedBeadsBuilder, TypedBeadsBuilder, FixedSizeBeadsIncrementalUintBuilder, OwningIndexedBeadsBuilder};
use std::cell::RefCell;
use std::collections::HashMap;
use crate::vlq::add_as_vlq;

#[derive(Debug)]
pub enum CsvColumnBuilder {
    Utf8(TypedBeadsBuilder),
    DedupUtf8(OwningDedupBeadsBuilder),
    OptionalUtf8(TypedBeadsBuilder),
    Int(TypedBeadsBuilder),
    Double(TypedBeadsBuilder, f64),
    Bool(TypedBeadsBuilder, &'static str, &'static str)
}

impl CsvColumnBuilder {
    pub fn utf8() -> CsvColumnBuilder {
        CsvColumnBuilder::Utf8(TypedBeadsBuilder::utf8())
    }
    pub fn dedup_utf8() -> CsvColumnBuilder {
        CsvColumnBuilder::DedupUtf8(OwningDedupBeadsBuilder::new())
    }
    pub fn optional_utf8() -> CsvColumnBuilder {
        CsvColumnBuilder::OptionalUtf8(TypedBeadsBuilder::optional_utf8())
    }
    pub fn f64() -> CsvColumnBuilder {
        CsvColumnBuilder::Double(TypedBeadsBuilder::f64(), 0.0)
    }
    pub fn optional_f64() -> CsvColumnBuilder {
        CsvColumnBuilder::Double(TypedBeadsBuilder::optional_f64(), 0.0)
    }
    pub fn f16_or_f32(accuracy: f64) -> CsvColumnBuilder {
        CsvColumnBuilder::Double(TypedBeadsBuilder::f16_or_f32(), accuracy)
    }
    pub fn float(accuracy: f64) -> CsvColumnBuilder {
        CsvColumnBuilder::Double(TypedBeadsBuilder::float(), accuracy)
    }
    pub fn f32(accuracy: f64) -> CsvColumnBuilder {
        CsvColumnBuilder::Double(TypedBeadsBuilder::f32(), accuracy)
    }
    pub fn optional_f32(accuracy: f64) -> CsvColumnBuilder {
        CsvColumnBuilder::Double(TypedBeadsBuilder::optional_f32(), accuracy)
    }
    pub fn f16(accuracy: f64) -> CsvColumnBuilder {
        CsvColumnBuilder::Double(TypedBeadsBuilder::f16(), accuracy)
    }
    pub fn optional_f16(accuracy: f64) -> CsvColumnBuilder {
        CsvColumnBuilder::Double(TypedBeadsBuilder::optional_f16(), accuracy)
    }

    pub fn bool(false_id: &'static str, true_id: &'static str) -> CsvColumnBuilder {
        CsvColumnBuilder::Bool(TypedBeadsBuilder::bool(), false_id, true_id)
    }

    fn push(&mut self, value: &str) -> bool {
        match self {
            CsvColumnBuilder::Utf8(builder) => {
                builder.push_string(value)
            }
            CsvColumnBuilder::OptionalUtf8(builder) => {
                if value.is_empty() {
                    builder.push_none()
                } else {
                    builder.push_string(value)
                }
            }
            CsvColumnBuilder::Int(builder) => {
                if value.is_empty() {
                    builder.push_none()
                } else {
                    match value.parse::<i128>() {
                        Ok(v) => builder.push_int(v),
                        Err(_) => false
                    }
                }
            }
            CsvColumnBuilder::Double(builder, accuracy) => {
                if value.is_empty() {
                    builder.push_none()
                } else {
                    match value.parse::<f64>() {
                        Ok(v) => builder.push_double_with_accuracy(v, *accuracy),
                        Err(_) => false
                    }
                }
            }
            CsvColumnBuilder::DedupUtf8(builder) => {
                builder.push(value.as_ref());
                true
            }
            CsvColumnBuilder::Bool(builder, false_id, true_id) => {
                let value = value.to_lowercase();
                if value == false_id.to_lowercase() {
                    builder.push_bool(false)
                } else if value == true_id.to_lowercase() {
                    builder.push_bool(true)
                } else {
                    false
                }
            }
        }
    }
}

macro_rules! csv_column_int_builder_factory {
    ( $x:ident ) => (
impl CsvColumnBuilder {
    pub fn $x() -> CsvColumnBuilder {
        CsvColumnBuilder::Int(TypedBeadsBuilder::$x())
    }
}
    )
}

csv_column_int_builder_factory!(i8);
csv_column_int_builder_factory!(optional_i8);
csv_column_int_builder_factory!(i16);
csv_column_int_builder_factory!(optional_i16);
csv_column_int_builder_factory!(i32);
csv_column_int_builder_factory!(optional_i32);
csv_column_int_builder_factory!(i64);
csv_column_int_builder_factory!(optional_i64);
csv_column_int_builder_factory!(i128);
csv_column_int_builder_factory!(optional_i128);

csv_column_int_builder_factory!(u8);
csv_column_int_builder_factory!(optional_u8);
csv_column_int_builder_factory!(u16);
csv_column_int_builder_factory!(optional_u16);
csv_column_int_builder_factory!(u32);
csv_column_int_builder_factory!(optional_u32);
csv_column_int_builder_factory!(u64);
csv_column_int_builder_factory!(optional_u64);
csv_column_int_builder_factory!(u128);
csv_column_int_builder_factory!(optional_u128);

csv_column_int_builder_factory!(vlq);
csv_column_int_builder_factory!(optional_vlq);
csv_column_int_builder_factory!(vlqz);
csv_column_int_builder_factory!(optional_vlqz);

impl BeadsBuilder for CsvColumnBuilder {
    fn encode(&self, writer: &RefCell<dyn io::Write + '_>) {
        match self {
            CsvColumnBuilder::Utf8(builder) => {
                BeadsBuilder::encode(builder, writer);
            }
            CsvColumnBuilder::OptionalUtf8(builder) => {
                BeadsBuilder::encode(builder, writer);
            }
            CsvColumnBuilder::Int(builder) => {
                BeadsBuilder::encode(builder, writer);
            }
            CsvColumnBuilder::Double(builder, _) => {
                BeadsBuilder::encode(builder, writer);
            }
            CsvColumnBuilder::DedupUtf8(builder) => {
                BeadsBuilder::encode(builder, writer);
            }
            CsvColumnBuilder::Bool(builder, _, _) => {
                BeadsBuilder::encode(builder, writer)
            }
        }
    }

    fn len(&self) -> usize {
        match self {
            CsvColumnBuilder::Utf8(builder) => {
                builder.len()
            }
            CsvColumnBuilder::OptionalUtf8(builder) => {
                builder.len()
            }
            CsvColumnBuilder::Int(builder) => {
                builder.len()
            }
            CsvColumnBuilder::Double(builder, _) => {
                builder.len()
            }
            CsvColumnBuilder::DedupUtf8(builder) => {
                builder.len()
            }
            CsvColumnBuilder::Bool(builder, _, _) => {
                builder.len()
            }
        }
    }
}

pub fn csv_to_beads<W>(csv: &str, first_line_rule: CsvFirstLine, builders: Vec<RefCell<CsvColumnBuilder>>, writer: &mut W) -> Result<(), String> where W: io::Write {
    let csv = csv.as_bytes();
    let mut offset = 0;
    let quote = "\"".as_bytes()[0];
    let separator = ",".as_bytes()[0];
    let n = "\n".as_bytes()[0];
    let r = "\r".as_bytes()[0];
    let mut is_in_double_quotes = false;
    let mut bytes: Vec<u8> = vec![];
    let mut column_index = 0;
    let mut first_line = true;

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
            let builder = builders.get(column_index).ok_or(format!("Number of provided builders {} does not match number of columns {}", builders.len(), column_index+1))?;
            if first_line == false || first_line_rule == CsvFirstLine::Include {
                if builder.borrow_mut().push(std::str::from_utf8(&bytes).map_err(|_| "")?) == false {
                    return Err(format!("Could not add {:?} at column: {} with builder: {:?}", std::str::from_utf8(&bytes), column_index, builder))
                }
            }
            bytes.clear();

            offset += 1;
            column_index += 1;
        } else if (char == n || char == r) && is_in_double_quotes == false {
            let builder = builders.get(column_index).ok_or("Number of provided builders does not match number of columns")?;
            if first_line == false || first_line_rule == CsvFirstLine::Include {
                if builder.borrow_mut().push(std::str::from_utf8(&bytes).map_err(|_| "")?) == false {
                    return Err(format!("Could not add {:?}", std::str::from_utf8(&bytes)))
                }
            }
            bytes.clear();

            offset += 1;
            column_index = 0;
            first_line = false;
        } else if char == r && csv.len() > offset + 1 && csv[offset+1] == n && is_in_double_quotes == false {
            let builder = builders.get(column_index).ok_or("Number of provided builders does not match number of columns")?;
            if first_line == false || first_line_rule == CsvFirstLine::Include {
                if builder.borrow_mut().push(std::str::from_utf8(&bytes).map_err(|_| "")?) == false {
                    return Err(format!("Could not add {:?}", std::str::from_utf8(&bytes)))
                }
            }
            bytes.clear();

            offset += 2;
            column_index = 0;
            first_line = false;
        } else {
            bytes.push(char);
            offset += 1;
        }
    }

    if bytes.is_empty() == false {
        let builder = builders.get(column_index).ok_or("Number of provided builders does not match number of columns")?;
        if first_line == false || first_line_rule == CsvFirstLine::Include {
            if builder.borrow_mut().push(std::str::from_utf8(&bytes).map_err(|_| "")?) == false {
                return Err(format!("Could not add {:?}", std::str::from_utf8(&bytes)))
            }
        }
        bytes.clear();
    }

    IndexedBeadsBuilder::encode_from_csv(writer, builders);
    Ok(())
}

#[derive(Debug)]
pub struct DedupBeadsBuilder<'a> {
    value_builder: IndexedBeadsBuilder<'a>,
    index_builder: FixedSizeBeadsIncrementalUintBuilder,
    lookup: HashMap<&'a[u8], u128>,
    index: u128,
}

impl <'a> DedupBeadsBuilder<'a> {

    fn new() -> DedupBeadsBuilder<'a> {
        DedupBeadsBuilder {
            value_builder: IndexedBeadsBuilder::new(),
            index_builder: FixedSizeBeadsIncrementalUintBuilder::new(),
            lookup: HashMap::new(),
            index: 0
        }
    }

    fn push(&mut self, value: &'a[u8]) {
        if self.lookup.contains_key(value) {
            self.index_builder.push(self.lookup[value]);
        } else {
            self.lookup.insert(value, self.index);
            self.value_builder.push(value);
            self.index_builder.push(self.index);
            self.index += 1;
        }
    }
}

impl BeadsBuilder for DedupBeadsBuilder<'_> {
    fn encode<'a>(&self, writer: &RefCell<dyn io::Write + 'a>) {

        let cursor: usize = self.index_builder.len() + self.value_builder.len();
        let bytes_per_index_entry = (8 - cursor.leading_zeros() / 8) as usize;

        let header = ((2u128) << 3) | ((bytes_per_index_entry - 1) as u128);

        let mut tmp = [0; 2];
        let count_length = add_as_vlq(tmp.as_mut(), header);
        writer.borrow_mut().write_all(tmp[..count_length].as_ref()).expect("could not write");
        let mut cursor = 0;
        cursor += self.index_builder.len();
        let bytes = cursor.to_le_bytes();
        writer.borrow_mut().write_all(&bytes[..bytes_per_index_entry]).expect("could not write");

        cursor += self.value_builder.len();
        let bytes = cursor.to_le_bytes();
        writer.borrow_mut().write_all(&bytes[..bytes_per_index_entry]).expect("could not write");

        BeadsBuilder::encode(&self.index_builder, writer);
        BeadsBuilder::encode(&self.value_builder, writer);
    }

    fn len(&self) -> usize {
        let cursor: usize = self.index_builder.len() + self.value_builder.len();
        let bytes_per_index_entry = (8 - cursor.leading_zeros() / 8) as usize;
        self.index_builder.len() + self.value_builder.len() + 1 + bytes_per_index_entry * 2
    }
}


#[derive(Debug)]
pub struct OwningDedupBeadsBuilder {
    value_builder: OwningIndexedBeadsBuilder,
    index_builder: FixedSizeBeadsIncrementalUintBuilder,
    lookup: HashMap<Vec<u8>, u128>,
    index: u128,
}

impl OwningDedupBeadsBuilder {

    pub fn new() -> OwningDedupBeadsBuilder {
        OwningDedupBeadsBuilder {
            value_builder: OwningIndexedBeadsBuilder::new(),
            index_builder: FixedSizeBeadsIncrementalUintBuilder::new(),
            lookup: HashMap::new(),
            index: 0
        }
    }

    fn push(&mut self, value: &[u8]) {
        let vec = value.to_vec();
        if self.lookup.contains_key(value) {
            self.index_builder.push(self.lookup[value]);
        } else {
            self.lookup.insert(vec, self.index);
            self.value_builder.push(value);
            self.index_builder.push(self.index);
            self.index += 1;
        }
    }
}

impl BeadsBuilder for OwningDedupBeadsBuilder {
    fn encode(&self, writer: &RefCell<dyn io::Write + '_>) {

        let cursor: usize = self.index_builder.len() + self.value_builder.len();
        let bytes_per_index_entry = (8 - cursor.leading_zeros() / 8) as usize;

        let header = ((2u128) << 3) | ((bytes_per_index_entry - 1) as u128);

        let mut tmp = [0; 2];
        let count_length = add_as_vlq(tmp.as_mut(), header);
        writer.borrow_mut().write_all(tmp[..count_length].as_ref()).expect("could not write");
        let mut cursor = 0;
        cursor += self.index_builder.len();
        let bytes = cursor.to_le_bytes();
        writer.borrow_mut().write_all(&bytes[..bytes_per_index_entry]).expect("could not write");

        cursor += self.value_builder.len();
        let bytes = cursor.to_le_bytes();
        writer.borrow_mut().write_all(&bytes[..bytes_per_index_entry]).expect("could not write");

        BeadsBuilder::encode(&self.index_builder, writer);
        BeadsBuilder::encode(&self.value_builder, writer);
    }

    fn len(&self) -> usize {
        let cursor: usize = self.index_builder.len() + self.value_builder.len();
        let bytes_per_index_entry = (8 - cursor.leading_zeros() / 8) as usize;
        self.index_builder.len() + self.value_builder.len() + 1 + bytes_per_index_entry * 2
    }
}