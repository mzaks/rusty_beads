use crate::vlq::{add_as_vlq, add_as_vlqz};
use half::f16;

#[derive(PartialEq, Hash, Clone, Copy)]
pub enum BeadType {
    None = 1,
    TrueFlag = 1 << 1,
    FalseFlag = 1 << 2,
    U8 = 1 << 3,
    U16 = 1 << 4,
    U32 = 1 << 5,
    U64 = 1 << 6,
    U128 = 1 << 7,
    I8 = 1 << 8,
    I16 = 1 << 9,
    I32 = 1 << 10,
    I64 = 1 << 11,
    I128 = 1 << 12,
    F16 = 1 << 13,
    F32 = 1 << 14,
    F64 = 1 << 15,
    Vlq =  1 << 16,
    VlqZ =  1 << 17,
    Utf8 =  1 << 18,
    Bytes =  1 << 19,
}

impl Eq for BeadType {
}

impl BeadType {
    pub(crate) fn cases_by_priority() -> Vec<BeadType> {
        vec![
            BeadType::None,
            BeadType::TrueFlag,
            BeadType::FalseFlag,
            BeadType::U8,
            BeadType::U16,
            BeadType::U32,
            BeadType::U64,
            BeadType::U128,
            BeadType::I8,
            BeadType::I16,
            BeadType::I32,
            BeadType::I64,
            BeadType::I128,
            BeadType::F16,
            BeadType::F32,
            BeadType::F64,
            BeadType::Vlq,
            BeadType::VlqZ,
            BeadType::Utf8,
            BeadType::Bytes,
        ]
    }

    pub(crate) fn cases_for_uint() -> Vec<BeadType> {
        vec![
            BeadType::U8,
            BeadType::I8,
            BeadType::Vlq,
            BeadType::VlqZ,
            BeadType::U16,
            BeadType::I16,
            BeadType::F16,
            BeadType::U32,
            BeadType::I32,
            BeadType::F32,
            BeadType::U64,
            BeadType::I64,
            BeadType::F64,
            BeadType::U128,
            BeadType::I128,
        ]
    }

    pub(crate) fn cases_for_int() -> Vec<BeadType> {
        vec![
            BeadType::I8,
            BeadType::U8,
            BeadType::VlqZ,
            BeadType::Vlq,
            BeadType::I16,
            BeadType::U16,
            BeadType::F16,
            BeadType::I32,
            BeadType::U32,
            BeadType::F32,
            BeadType::I64,
            BeadType::U64,
            BeadType::F64,
            BeadType::I128,
            BeadType::U128,
        ]
    }

    pub(crate) fn cases_for_double() -> Vec<BeadType> {
        vec![
            BeadType::U8,
            BeadType::I8,
            BeadType::VlqZ,
            BeadType::Vlq,
            BeadType::F16,
            BeadType::I16,
            BeadType::U16,
            BeadType::F32,
            BeadType::I32,
            BeadType::U32,
            BeadType::F64,
        ]
    }

    pub(crate) fn has_no_data(&self) -> bool {
        return match self {
            BeadType::None | BeadType::TrueFlag | BeadType::FalseFlag => true,
            _ => false
        }
    }

    pub(crate) fn data_size(&self) -> u8 {
        return match self {
            BeadType::None | BeadType::TrueFlag | BeadType::FalseFlag => 0,
            BeadType::U8 | BeadType::I8 => 1,
            BeadType::U16 | BeadType::I16 | BeadType::F16 => 2,
            BeadType::U32 | BeadType::I32 | BeadType::F32 => 4,
            BeadType::U64 | BeadType::I64 | BeadType::F64 => 8,
            BeadType::U128 | BeadType::I128 => 16,
            _ => 255
        }
    }

    pub(crate) fn push_uint(&self, value: u128, buffer: &mut [u8]) -> (bool, usize) {
        return match self {
            BeadType::U8 => {
                if value >> 8 == 0 {
                    buffer[0] = value as u8;
                    return (true, 1)
                }
                (false, 0)
            }
            BeadType::U16 => {
                if value >> 16 == 0 {
                    let v = value as u16;
                    let b = v.to_le_bytes();
                    buffer[..b.len()].copy_from_slice(&b);
                    return (true, b.len())
                }
                (false, 0)
            }
            BeadType::U32 => {
                if value >> 32 == 0 {
                    let v = value as u32;
                    let b = v.to_le_bytes();
                    buffer[..b.len()].copy_from_slice(&b);
                    return (true, b.len())
                }
                (false, 0)
            }
            BeadType::U64 => {
                if value >> 64 == 0 {
                    let v = value as u64;
                    let b = v.to_le_bytes();
                    buffer[..b.len()].copy_from_slice(&b);
                    return (true, b.len())
                }
                (false, 0)
            }
            BeadType::U128 => {
                let b = value.to_le_bytes();
                buffer[..b.len()].copy_from_slice(&b);
                (true, b.len())
            }
            BeadType::I8 => {
                if value >> 7 == 0 {
                    buffer[0] = value as u8;
                    return (true, 1)
                }
                (false, 0)
            }
            BeadType::I16 => {
                if value >> 15 == 0 {
                    let v = value as i16;
                    let b = v.to_le_bytes();
                    buffer[..b.len()].copy_from_slice(&b);
                    return (true, b.len())
                }
                (false, 0)
            }
            BeadType::I32 => {
                if value >> 31 == 0 {
                    let v = value as i32;
                    let b = v.to_le_bytes();
                    buffer[..b.len()].copy_from_slice(&b);
                    return (true, b.len())
                }
                (false, 0)
            }
            BeadType::I64 => {
                if value >> 63 == 0 {
                    let v = value as i64;
                    let b = v.to_le_bytes();
                    buffer[..b.len()].copy_from_slice(&b);
                    return (true, b.len())
                }
                (false, 0)
            }
            BeadType::I128 => {
                if value >> 127 == 0 {
                    let v = value as i128;
                    let b = v.to_le_bytes();
                    buffer[..b.len()].copy_from_slice(&b);
                    return (true, b.len())
                }
                (false, 0)
            }
            BeadType::Vlq => {
                let length = add_as_vlq(buffer, value);
                (true, length)
            }
            BeadType::VlqZ => {
                if value >> 127 == 0 {
                    let v = value as i128;
                    let length = add_as_vlqz(buffer, v);
                    return (true, length)
                }
                (false, 0)
            }
            BeadType::F32 => {
                let f = value as f32;

                if value == f as u128 {
                    let b = f.to_le_bytes();
                    buffer[..b.len()].copy_from_slice(&b);
                    return (true, b.len())
                }
                (false, 0)
            }
            BeadType::F64 => {
                let f = value as f64;

                if value == f as u128 {
                    let b = f.to_le_bytes();
                    buffer[..b.len()].copy_from_slice(&b);
                    return (true, b.len())
                }
                (false, 0)
            }
            BeadType::F16 => {
                let f = f16::from_f32(value as f32);

                if value == f.to_f32() as u128 {
                    let b = f.to_bits().to_le_bytes();
                    buffer[..b.len()].copy_from_slice(&b);
                    return (true, b.len())
                }
                (false, 0)
            }
            _ => (false, 0)
        }
    }

    pub(crate) fn push_int(&self, value: i128, buffer: &mut [u8]) -> (bool, usize) {
        return match self {
            BeadType::I8 => {
                let v = value as i8;
                if value == v as i128 {
                    let b = v.to_le_bytes();
                    buffer[..b.len()].copy_from_slice(&b);
                    return (true, b.len())
                }
                (false, 0)
            }
            BeadType::I16 => {
                let v = value as i16;
                if value == v as i128 {
                    let b = v.to_le_bytes();
                    buffer[..b.len()].copy_from_slice(&b);
                    return (true, b.len())
                }
                (false, 0)
            }
            BeadType::I32 => {
                let v = value as i32;
                if value == v as i128 {
                    let b = v.to_le_bytes();
                    buffer[..b.len()].copy_from_slice(&b);
                    return (true, b.len())
                }
                (false, 0)
            }
            BeadType::I64 => {
                let v = value as i64;
                if value == v as i128 {
                    let b = v.to_le_bytes();
                    buffer[..b.len()].copy_from_slice(&b);
                    return (true, b.len())
                }
                (false, 0)
            }
            BeadType::I128 => {
                let b = value.to_le_bytes();
                buffer[..b.len()].copy_from_slice(&b);
                (true, b.len())
            }
            BeadType::U8 => {
                if value >= 0 && value >> 8 == 0 {
                    buffer[0] = value as u8;
                    return (true, 1)
                }
                (false, 0)
            }
            BeadType::U16 => {
                if value >= 0 && value >> 16 == 0 {
                    let v = value as u16;
                    let b = v.to_le_bytes();
                    buffer[..b.len()].copy_from_slice(&b);
                    return (true, b.len())
                }
                (false, 0)
            }
            BeadType::U32 => {
                if value >= 0 && value >> 32 == 0 {
                    let v = value as u32;
                    let b = v.to_le_bytes();
                    buffer[..b.len()].copy_from_slice(&b);
                    return (true, b.len())
                }
                (false, 0)
            }
            BeadType::U64 => {
                if value >= 0 && value >> 64 == 0 {
                    let v = value as u64;
                    let b = v.to_le_bytes();
                    buffer[..b.len()].copy_from_slice(&b);
                    return (true, b.len())
                }
                (false, 0)
            }
            BeadType::U128 => {
                if value >= 0 {
                    let b = value.to_le_bytes();
                    buffer[..b.len()].copy_from_slice(&b);
                    return (true, b.len())
                }
                (false, 0)
            }

            BeadType::Vlq => {
                if value >= 0 {
                    let length = add_as_vlq(buffer, value as u128);
                    return (true, length)
                }
                (false, 0)
            }
            BeadType::VlqZ => {
                let length = add_as_vlqz(buffer, value);
                return (true, length)
            }
            BeadType::F32 => {
                let f = value as f32;

                if value == f as i128 {
                    let b = f.to_le_bytes();
                    buffer[..b.len()].copy_from_slice(&b);
                    return (true, b.len())
                }
                (false, 0)
            }
            BeadType::F64 => {
                let f = value as f64;

                if value == f as i128 {
                    let b = f.to_le_bytes();
                    buffer[..b.len()].copy_from_slice(&b);
                    return (true, b.len())
                }
                (false, 0)
            }
            BeadType::F16 => {
                let f = f16::from_f32(value as f32);

                if value == f.to_f32() as i128 {
                    let b = f.to_bits().to_le_bytes();
                    buffer[..b.len()].copy_from_slice(&b);
                    return (true, b.len())
                }
                (false, 0)
            }
            _ => (false, 0)
        }
    }

    pub(crate) fn push_double(&self, value: f64, accuracy: f64, buffer: &mut [u8]) -> (bool, usize) {
        return match self {
            BeadType::I8 => {
                let v = value as i8;
                if (value - v as f64).abs() <= accuracy {
                    let b = v.to_le_bytes();
                    buffer[..b.len()].copy_from_slice(&b);
                    return (true, b.len())
                }
                (false, 0)
            }
            BeadType::I16 => {
                let v = value as i16;
                if (value - v as f64).abs() <= accuracy {
                    let b = v.to_le_bytes();
                    buffer[..b.len()].copy_from_slice(&b);
                    return (true, b.len())
                }
                (false, 0)
            }
            BeadType::I32 => {
                let v = value as i32;
                if (value - v as f64).abs() <= accuracy {
                    let b = v.to_le_bytes();
                    buffer[..b.len()].copy_from_slice(&b);
                    return (true, b.len())
                }
                (false, 0)
            }
            BeadType::U8 => {
                let v = value as u8;
                if (value - v as f64).abs() <= accuracy {
                    buffer[0] = value as u8;
                    return (true, 1)
                }
                (false, 0)
            }
            BeadType::U16 => {
                let v = value as u16;
                if (value - v as f64).abs() <= accuracy {
                    let b = v.to_le_bytes();
                    buffer[..b.len()].copy_from_slice(&b);
                    return (true, b.len())
                }
                (false, 0)
            }
            BeadType::U32 => {
                let v = value as u32;
                if (value - v as f64).abs() <= accuracy {
                    let b = v.to_le_bytes();
                    buffer[..b.len()].copy_from_slice(&b);
                    return (true, b.len())
                }
                (false, 0)
            }
            BeadType::Vlq => {
                let v = value as u128;
                if (value - v as f64).abs() <= accuracy {
                    let length = add_as_vlq(buffer, v);
                    return (true, length)
                }
                (false, 0)
            }
            BeadType::VlqZ => {
                let v = value as i128;
                if (value - v as f64).abs() <= accuracy {
                    let length = add_as_vlqz(buffer, v);
                    return (true, length)
                }
                (false, 0)
            }
            BeadType::F32 => {
                let v = value as f32;

                if (value - v as f64).abs() <= accuracy {
                    let b = v.to_le_bytes();
                    buffer[..b.len()].copy_from_slice(&b);
                    return (true, b.len())
                }
                (false, 0)
            }
            BeadType::F64 => {
                let b = value.to_le_bytes();
                buffer[..b.len()].copy_from_slice(&b);
                return (true, b.len())
            }
            BeadType::F16 => {
                let v = f16::from_f32(value as f32);

                if (value - v.to_f64()).abs() <= accuracy {
                    let b = v.to_bits().to_le_bytes();
                    buffer[..b.len()].copy_from_slice(&b);
                    return (true, b.len())
                }
                (false, 0)
            }
            _ => (false, 0)
        }
    }
}


pub struct BeadTypeSet {
    value: u32
}

impl BeadTypeSet {
    pub fn new(types: &[BeadType]) -> BeadTypeSet {
        let mut value = 0u32;
        for t in types {
            let t1 = *t;
            value |= t1 as u32;
        }
        BeadTypeSet {
            value
        }
    }

    pub fn contains(&self, value: &BeadType) -> bool {
        (*value as u32) & self.value != 0
    }

    pub fn size(&self) -> usize {
        self.value.count_ones() as usize
    }

    pub fn bytes(&self) -> [u8;4] {
        self.value.to_le_bytes()
    }
}

impl From<u32> for BeadTypeSet {
    fn from(value: u32) -> Self {
        BeadTypeSet {
            value
        }
    }
}