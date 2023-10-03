use crate::nafobj::File;
// use bio::io::fastq::Error as FastQError;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize};
use std::error::Error;
use std::fmt::{self, Display};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::ops::Add;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VarlenInt(u128);
#[derive(Debug, Clone)]
pub enum VarlenIntError {
    NegativeNumber,
    IncompleteBuffer,
}

#[derive(Debug, PartialEq)]
struct ReadOffset {
    readlen: usize,
    complete: bool,
}

impl VarlenInt {
    pub fn new(value: u128) -> VarlenInt {
        VarlenInt(value)
    }

    fn parse_byte_buf(&mut self, input_buffer: &[u8]) -> ReadOffset {
        let mut i = 1;
        for a_byte in input_buffer {
            let mut last_byte = *a_byte;
            last_byte &= 0x7f;
            self.0 *= 128;
            self.0 += last_byte as u128;
            if (a_byte & 0x80) == 0 {
                // last byte in varlen int
                return ReadOffset {
                    readlen: i,
                    complete: true,
                };
            }
            i += 1;
        }
        ReadOffset {
            readlen: input_buffer.len(),
            complete: false,
        }
    }

    pub fn read_from_bytes(input_bytes: &[u8]) -> Result<Self, VarlenIntError> {
        let mut res = VarlenInt(0);
        let is_done = res.parse_byte_buf(input_bytes);
        if !is_done.complete {
            return Err(VarlenIntError::IncompleteBuffer);
        }
        Ok(res)
    }

    pub fn read_from_file(input_file: &mut File) -> Result<VarlenInt, std::io::Error> {
        let mut number = VarlenInt(0);
        let mut is_done_reading = false;
        let mut buffer_seek: i64 = 0;
        let mut buff_reader = BufReader::with_capacity(8, input_file);
        while !is_done_reading {
            let read_offset = number.parse_byte_buf(buff_reader.fill_buf()?);
            is_done_reading = read_offset.complete;
            buffer_seek += read_offset.readlen as i64;
        }
        match buff_reader.seek(SeekFrom::Current(buffer_seek)) {
            //buffer_seek) {
            Err(e) => {
                println!(
                    "Tried to seek {}, and all I got was this lousy {}",
                    buffer_seek, e
                );
                return Err(e);
            }
            _ => {}
        }; //..(SeekFrom::Current(buffer_seek))?;

        Ok(number)
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut accumulator = Vec::<u8>::new();
        let VarlenInt(mut res) = *self;
        let mut t: u8 = (res % 128) as u8;
        while res > 0 {
            accumulator.push(t);
            res = (res - ((t % 128) as u128)) / 128;
            t = ((res % 128) + 128) as u8;
        }
        accumulator.reverse();
        accumulator
    }
}
#[cfg(test)]
mod tests {
    use crate::nafobj::VarlenInt;
    #[test]
    fn test_parse_byte_buf() {
        let mut test_int = VarlenInt::new(0);
        let test_read = test_int.parse_byte_buf(&[0x50, 0x81, 0x75]);
        assert_eq!(test_read.complete, true);
        assert_eq!(test_read.readlen, 1);
        assert_eq!(test_int, VarlenInt::new(80));
        let mut test_int = VarlenInt::new(0);
        assert_eq!(test_int.parse_byte_buf(&[0x81, 0x75]).complete, true);
        assert_eq!(test_int, VarlenInt::new(245));
        let mut test_int = VarlenInt::new(0);
        assert_eq!(test_int.parse_byte_buf(&[0x81, 0x80, 0x80]).complete, false);
        assert_eq!(test_int.parse_byte_buf(&[0x80, 0x80, 0x00]).complete, true);
        assert_eq!(test_int, VarlenInt::new(34_359_738_368));
        let mut test_int = VarlenInt::new(0);
        assert_eq!(test_int.parse_byte_buf(&[0x01]).complete, true);
        assert_eq!(test_int, VarlenInt::new(1));
        let mut test_int = VarlenInt::new(0);
        assert_eq!(test_int.parse_byte_buf(&[0x00]).complete, true);
        assert_eq!(test_int, VarlenInt::new(0));
    }

    #[test]
    fn test_as_bytes() {
        assert_eq!(VarlenInt::new(245).as_bytes(), vec![0x81, 0x75])
    }

    #[test]
    fn test_add() {
        assert_eq!(
            VarlenInt::new(1024) + VarlenInt::new(9099),
            VarlenInt::new(10123)
        )
    }
}

impl Add for VarlenInt {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        VarlenInt(self.0 + rhs.0)
    }
}

impl Display for VarlenInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}
struct VarlenIntVisitor;

impl<'de> Visitor<'de> for VarlenIntVisitor {
    type Value = VarlenInt;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("An integer greater than or equal to zero.")
    }

    fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(VarlenInt(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(VarlenInt(u128::from(v)))
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(VarlenInt(u128::from(v)))
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(VarlenInt(u128::from(v)))
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: Error,
    {
        Ok(VarlenInt(u128::from(v)))
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v < 0 {
            return Err(E::custom(format!("VarlenInt cannot be negative: {}", v)));
        }
        Ok(VarlenInt(u128::from(v as u8)))
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v < 0 {
            return Err(E::custom(format!("VarlenInt cannot be negative: {}", v)));
        }
        Ok(VarlenInt(u128::from(v as u16)))
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v < 0 {
            return Err(E::custom(format!("VarlenInt cannot be negative: {}", v)));
        }
        Ok(VarlenInt(u128::from(v as u32)))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match VarlenInt::read_from_bytes(v) {
            Ok(res) => Ok(res),
            Err(_) => {
                return Err(E::custom(format!(
                    "VarlenInt invalid or incomplete, byte string does not end with value <x80"
                )))
            }
        }
    }
}

impl<'de> Deserialize<'de> for VarlenInt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u128(VarlenIntVisitor)
    }
}

impl Serialize for VarlenInt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_bytes(self.as_bytes().as_slice())
    }
}
