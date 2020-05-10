#[inline]
pub(crate) fn add_as_vlq(buffer: &mut[u8], mut value: u128) -> usize {
    let mut index = 0;
    while value > 127 {
        buffer[index] = ((value & 127) | 128) as u8;
        index += 1;
        value >>= 7;
    }
    buffer[index] = value as u8;
    index +=1;
    index
}

#[inline]
pub(crate) fn add_as_vlqz(buffer: &mut[u8], value: i128) -> usize {
    add_as_vlq(buffer, zigzag_encode(value))
}

#[inline]
fn zigzag_encode(value: i128) -> u128 {
    ((value >> 127) ^ (value << 1)) as u128
}

#[inline]
pub(crate) fn zigzag_decode(value: u128) -> i128 {((value >> 1) as i128) ^ (-1 * ((value & 1) as i128))}

#[inline]
pub(crate) fn read_vlq(buffer: &[u8]) -> Result<(usize, u128), &'static str> {
    let mut result = 0u128;
    let mut index = 0u8;
    loop {
        let b = buffer.get(index as usize).ok_or("Buffer does not contain a valid vlq")?;
        let b1 = (b & 127) as u128;
        let shift = 7 * index as u128;
        result |= b1 << shift;
        if b & 128 == 0 {
            break
        }
        index += 1;
    }
    Ok(((index + 1) as usize, result))
}

pub(crate) trait VlqByteSize {
    fn vlq_byte_size(&self) -> usize;
}

impl VlqByteSize for usize {
    fn vlq_byte_size(&self) -> usize {
        if self >> 7 == 0usize {
            1
        } else if self >> 14 == 0usize {
            2
        } else if self >> 21 == 0usize {
            3
        } else if self >> 28 == 0usize {
            4
        } else if self >> 35 == 0usize {
            5
        } else if self >> 42 == 0usize {
            6
        } else if self >> 49 == 0usize {
            7
        } else if self >> 56 == 0usize {
            8
        } else if self >> 63 == 0usize {
            9
        } else {
            10
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::vlq::{add_as_vlq, zigzag_encode, read_vlq, zigzag_decode};

    #[test]
    fn add_vlq() {
        let mut buf = vec![0u8, 0, 0];
        assert_eq!(add_as_vlq(buf.as_mut_slice(), 45), 1);
        assert_eq!(buf[0], 45);
        // assert_eq!(read_vlq(buf.as_slice()), (1, 45));
        assert_eq!(add_as_vlq(buf.as_mut_slice(), 146), 2);
        assert_eq!(buf[0], 146);
        assert_eq!(buf[1], 1);
        // assert_eq!(read_vlq(buf.as_slice()), (2, 146));
        assert_eq!(add_as_vlq(buf.as_mut_slice(), 256), 2);
        assert_eq!(buf[0], 128);
        assert_eq!(buf[1], 2);
        assert_eq!(read_vlq(buf.as_slice()).ok().unwrap(), (2, 256));
        assert_eq!(add_as_vlq(buf.as_mut_slice(), 257), 2);
        assert_eq!(buf[0], 129);
        assert_eq!(buf[1], 2);
        assert_eq!(read_vlq(buf.as_slice()).ok().unwrap(), (2, 257));
        assert_eq!(add_as_vlq(buf.as_mut_slice(), 258), 2);
        assert_eq!(buf[0], 130);
        assert_eq!(buf[1], 2);
        assert_eq!(read_vlq(buf.as_slice()).ok().unwrap(), (2, 258));
        assert_eq!(add_as_vlq(buf.as_mut_slice(), 2580), 2);
        assert_eq!(buf[0], 148);
        assert_eq!(buf[1], 20);
        assert_eq!(read_vlq(buf.as_slice()).ok().unwrap(), (2, 2580));
        assert_eq!(add_as_vlq(buf.as_mut_slice(), 22580), 3);
        assert_eq!(buf[0], 180);
        assert_eq!(buf[1], 176);
        assert_eq!(buf[2], 1);
        assert_eq!(read_vlq(buf.as_slice()).ok().unwrap(), (3, 22580));
    }

    #[test]
    fn zig_zag_encode() {
        assert_eq!(zigzag_encode(0), 0);
        assert_eq!(zigzag_encode(-1), 1);
        assert_eq!(zigzag_encode(1), 2);
        assert_eq!(zigzag_encode(-2), 3);
        assert_eq!(zigzag_encode(2), 4);
        assert_eq!(zigzag_encode(-3), 5);
        assert_eq!(zigzag_encode(3), 6);
        assert_eq!(zigzag_encode(-127), 253);
        assert_eq!(zigzag_encode(127), 254);
        assert_eq!(zigzag_encode(-128), 255);
    }

    #[test]
    fn zig_zag_decode() {
        assert_eq!(zigzag_decode(zigzag_encode(0)), 0);
        assert_eq!(zigzag_decode(zigzag_encode(-1)), -1);
        assert_eq!(zigzag_decode(zigzag_encode(1)), 1);
        assert_eq!(zigzag_decode(zigzag_encode(-2)), -2);
        assert_eq!(zigzag_decode(zigzag_encode(2)), 2);
        assert_eq!(zigzag_decode(zigzag_encode(-3)), -3);
        assert_eq!(zigzag_decode(zigzag_encode(3)), 3);
        assert_eq!(zigzag_decode(zigzag_encode(-127)), -127);
        assert_eq!(zigzag_decode(zigzag_encode(127)), 127);
        assert_eq!(zigzag_decode(zigzag_encode(-128)), -128);
    }
}