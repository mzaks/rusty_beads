use crate::bead_type::BeadType;
use crate::vlq::zigzag_decode;
use half::f16;
use std::convert::TryFrom;

pub struct BeadReference<'a> {
    pub(crate) value: u128,
    pub(crate) buffer: &'a[u8],
    pub(crate) bead_type: BeadType
}

impl BeadReference<'_> {
    pub fn is_none(&self) -> bool {
        self.bead_type == BeadType::None
    }
    pub fn is_true(&self) -> bool {
        self.bead_type == BeadType::TrueFlag
    }
    pub fn is_false(&self) -> bool {
        self.bead_type == BeadType::FalseFlag
    }
    pub fn is_bool(&self) -> bool {
        self.bead_type == BeadType::TrueFlag || self.bead_type == BeadType::FalseFlag
    }
    pub fn is_uint(&self) -> bool {
        self.bead_type == BeadType::U8
            || self.bead_type == BeadType::U16
            || self.bead_type == BeadType::U32
            || self.bead_type == BeadType::U64
            || self.bead_type == BeadType::U128
            || self.bead_type == BeadType::Vlq
    }
    pub fn is_int(&self) -> bool {
        self.bead_type == BeadType::I8
            || self.bead_type == BeadType::I16
            || self.bead_type == BeadType::I32
            || self.bead_type == BeadType::I64
            || self.bead_type == BeadType::I128
            || self.bead_type == BeadType::VlqZ
    }
    pub fn is_float(&self) -> bool {
        self.bead_type == BeadType::F16
            || self.bead_type == BeadType::F32
            || self.bead_type == BeadType::F64
    }
    pub fn is_bytes(&self) -> bool {
        self.bead_type == BeadType::Bytes
    }
    pub fn is_string(&self) -> bool {
        self.bead_type == BeadType::Utf8
    }
    pub fn to_bool(&self) -> bool {
        if self.is_bool() {
            self.bead_type == BeadType::TrueFlag
        } else {
            panic!("Not a bool value")
        }
    }
    pub fn to_str(&self) -> &str {
        if self.is_string() {
            std::str::from_utf8(self.buffer).expect("value is not string")
        } else {
            panic!("value is not string")
        }
    }
    pub fn to_bytes(&self) -> &[u8] {
        self.buffer
    }
    pub fn to_uint(&self) -> u128 {
        return match self.bead_type {
            BeadType::Vlq => self.value,
            BeadType::U8 => self.buffer[0] as u128,
            BeadType::U16 => u16::from_le_bytes(Self::clone_into_array(self.buffer)) as u128,
            BeadType::U32 => u32::from_le_bytes(Self::clone_into_array(self.buffer)) as u128,
            BeadType::U64 => u64::from_le_bytes(Self::clone_into_array(self.buffer)) as u128,
            BeadType::U128 => u128::from_le_bytes(Self::clone_into_array(self.buffer)) as u128,
            _ => if self.is_int() {
                let int = self.to_int();
                if int >= 0  {
                    int as u128
                } else {
                    panic!("Is a negative int value")
                }
            } else {
                panic!("Not an int type")
            }
        }
    }

    pub fn to_int(&self) -> i128 {
        return match self.bead_type {
            BeadType::VlqZ => zigzag_decode(self.value),
            BeadType::I8 => i8::from_le_bytes(Self::clone_into_array(self.buffer)) as i128,
            BeadType::I16 => i16::from_le_bytes(Self::clone_into_array(self.buffer)) as i128,
            BeadType::I32 => i32::from_le_bytes(Self::clone_into_array(self.buffer)) as i128,
            BeadType::I64 => i64::from_le_bytes(Self::clone_into_array(self.buffer)) as i128,
            BeadType::I128 => i128::from_le_bytes(Self::clone_into_array(self.buffer)) as i128,
            _ => if self.is_uint() {
                let uint = self.to_uint();
                if uint <= std::i128::MAX as u128  {
                    uint as i128
                } else {
                    panic!("Is a very high uint value")
                }
            } else {
                panic!("Not an int type")
            }
        }
    }

    pub fn to_float(&self) -> f64 {
        return match self.bead_type {
            BeadType::F16 => f16::from_bits(u16::from_le_bytes(Self::clone_into_array(self.buffer))).to_f64(),
            BeadType::F32 => f32::from_le_bytes(Self::clone_into_array(self.buffer)) as f64,
            BeadType::F64 => f64::from_le_bytes(Self::clone_into_array(self.buffer)) as f64,
            _ => if self.is_int() || self.is_uint() {
                self.to_int() as f64
            } else {
                panic!("Not an int type")
            }
        }
    }

    pub(crate) fn clone_into_array<A, T>(slice: &[T]) -> A
        where A: Sized + Default + AsMut<[T]>,
              T: Clone
    {
        let mut a = Default::default();
        <A as AsMut<[T]>>::as_mut(&mut a).clone_from_slice(slice);
        a
    }
}

macro_rules! try_from_int {
    ( $( $x:ident ),* ) => {
        $(
impl TryFrom<BeadReference<'_>> for $x {
    type Error = ();
    fn try_from(value: BeadReference) -> Result<Self, Self::Error> {
        if value.is_uint() {
            <$x as TryFrom<u128>>::try_from(value.to_uint()).map_err(|_| ())
        } else if value.is_int() {
            <$x as TryFrom<i128>>::try_from(value.to_int()).map_err(|_| ())
        } else {
            Err(())
        }
    }
}
        )*
    }
}

try_from_int![u8, u16, u32, u64, u128, i8, i16, i32, i64, i128];

impl TryFrom<BeadReference<'_>> for String {
    type Error = ();
    fn try_from(value: BeadReference) -> Result<Self, Self::Error> {
        if value.is_string() {
            Ok(String::from(value.to_str()))
        } else {
            Err(())
        }
    }
}