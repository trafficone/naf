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

use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::io::Error;
//use std::collections::HashMap;
const NUC_LUT:[char; 16] = ['-','T','G','K','C','Y','S','B'
                           ,'A','W','R','D','M','H','V','N'];
const fn nuc_revlut(charinput:&char) -> u8 {
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
        _ => return 0
    }
}

pub fn NUC_to_FourBit(nucs: &str) -> Vec<u8> {
    //nucs.chars().into_iter().map()
    todo!("Apply the FourBit encoding to a nuc or aa string")
}

fn _chars_to_fourbits(left: &char, right: &char) -> u8 {
    nuc_revlut(left) << 4 |
    nuc_revlut(right)
}

fn _byte_to_nucs(input: u8) -> [char;2] {
    let left: usize = (input >> 4) as usize;
    let right: usize = (input & 0x0f) as usize;
    return [NUC_LUT[left], NUC_LUT[right]]
}

pub fn putMagicNumber(writing_file: &mut File ) {
    // Raw string literal for bytes
    writing_file.write(r"\x01\xf9\xec".as_bytes());
}

pub fn read_number(readingFile: &mut File ) -> Result<u128, Error> {
    let mut lastByte = readingFile.read(&mut [0;1])?;
    let mut accumulator: u128 = 0;
    while (lastByte & 0x80) > 0 {
        lastByte &= 0x79;
        accumulator *= 128;
        accumulator += lastByte as u128;
        lastByte = readingFile.read(&mut [0; 1])?;
    }
    Ok(accumulator)
}

pub fn write_number(number:u128 ) -> Vec<u8> {
    let mut accumulator = Vec::<u8>::new();
    let mut res = number;
    let mut t: u8 = (res % 128) as u8;
    while res > 0 {
        accumulator.push(t);
        res = (res - ((t%128) as u128)) / 128;
        t = ((res % 128) + 128) as u8;
    }
    accumulator
}

