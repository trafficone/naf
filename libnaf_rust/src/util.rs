/*
    - put magic number
    - read variable length encoded number
    - write variable length encoded number
    - init tables
    - out of memory (Error)
    - incomplete (Error)
    - (die) (panic handle function)
    - (err) (print error function)
    - (msg) (print message function)
    - (malloc or die)
    - (fgetc or incomplete)
*/

use serde::de::{self, Deserializer, Visitor};
use serde::{ser, Serialize};
use std::fmt;
use std::fs::File;
use std::io::Error;
use std::io::Read;
use std::io::Write;
//use std::collections::HashMap;
const NUC_LUT: [char; 16] = [
    '-', 'T', 'G', 'K', 'C', 'Y', 'S', 'B', 'A', 'W', 'R', 'D', 'M', 'H', 'V', 'N',
];
const fn nuc_revlut(charinput: &char) -> u8 {
    match charinput {
        '-' => return 0,
        'T' => return 1,
        'G' => return 2,
        'K' => return 3,
        'C' => return 4,
        'Y' => return 5,
        'S' => return 6,
        'B' => return 7,
        'A' => return 8,
        'W' => return 9,
        'R' => return 10,
        'D' => return 11,
        'M' => return 12,
        'H' => return 13,
        'V' => return 14,
        'N' => return 15,
        _ => return 0,
    }
}

pub fn NUC_to_FourBit(nucs: &str) -> Vec<u8> {
    //nucs.chars().into_iter().map()
    todo!("Apply the FourBit encoding to a nuc or aa string")
}

fn _chars_to_fourbits(left: &char, right: &char) -> u8 {
    nuc_revlut(left) << 4 | nuc_revlut(right)
}

fn _byte_to_nucs(input: u8) -> [char; 2] {
    let left: usize = (input >> 4) as usize;
    let right: usize = (input & 0x0f) as usize;
    return [NUC_LUT[left], NUC_LUT[right]];
}

pub fn putMagicNumber(writing_file: &mut File) {
    // Raw string literal for bytes
    writing_file.write(r"\x01\xf9\xec".as_bytes());
}

pub struct VarlenInt(u128, bool);
#[derive(Debug)]
enum VarlenIntError {
    NegativeNumber,
    IncompleteBuffer,
}

impl VarlenInt {
    fn parse_byte_buf(&self, input_buffer: &[u8]) -> Self {
        let VarlenInt(mut accumulator, _) = self;
        for a_byte in input_buffer {
            let mut last_byte = *a_byte;
            last_byte &= 0x79;
            accumulator *= 128;
            accumulator += last_byte as u128;
            if (a_byte & 0x80) == 0 {
                return VarlenInt(accumulator, true);
            }
        }
        VarlenInt(accumulator, false)
    }

    pub fn read_from_bytes(input_bytes: &[u8]) -> Result<Self, VarlenIntError> {
        let mut res = VarlenInt(0, false);
        res = res.parse_byte_buf(input_bytes);
        if {
            let VarlenInt(_, done) = res;
            !done
        } {
            return Err(VarlenIntError::IncompleteBuffer);
        }
        Ok(res)
    }

    pub fn read_from_file(input_file: &mut File) -> Result<VarlenInt, Error> {
        let mut buffer = [0; 8];
        let mut number = VarlenInt(0, false);
        while {
            let VarlenInt(_, done) = number;
            !done
        } {
            input_file.read(&mut buffer[..])?;
            number.parse_byte_buf(buffer.as_slice());
        }
        Ok(number)
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
        E: de::Error,
    {
        Ok(VarlenInt(v, true))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(VarlenInt(u128::from(v), true))
    }

    fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(VarlenInt(u128::from(v), true))
    }

    fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(VarlenInt(u128::from(v), true))
    }

    fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(VarlenInt(u128::from(v), true))
    }

    fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if v < 0 {
            return Err(E::custom(format!("VarlenInt cannot be negative: {}", v)));
        }
        Ok(VarlenInt(u128::from(v as u8), true))
    }

    fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if v < 0 {
            return Err(E::custom(format!("VarlenInt cannot be negative: {}", v)));
        }
        Ok(VarlenInt(u128::from(v as u16), true))
    }

    fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if v < 0 {
            return Err(E::custom(format!("VarlenInt cannot be negative: {}", v)));
        }
        Ok(VarlenInt(u128::from(v as u32), true))
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match VarlenInt::read_from_bytes(v) {
            Ok(res) => Ok(res),
            Err(e) => {
                return Err(E::custom(format!(
                    "VarlenInt invalid or incomplete, byte string does not end with value <x80"
                )))
            }
        }
    }
}

impl<'de> de::Deserialize<'de> for VarlenInt {
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
        S: ser::Serializer,
    {
        let mut accumulator = Vec::<u8>::new();
        let VarlenInt(mut res, _) = *self;
        let mut t: u8 = (res % 128) as u8;
        while res > 0 {
            accumulator.push(t);
            res = (res - ((t % 128) as u128)) / 128;
            t = ((res % 128) + 128) as u8;
        }
        serializer.serialize_bytes(accumulator.as_slice())
    }
}
