/* IO Functions for NAF 
    Input-specific:
    - ok read header
    - seek to section (TITLE, IDS, NAMES, LENGTH, etc.)
    - load IDs
    - load names
    - load lengths
    - load mask
    - load compressed sequence
    Output-specific - print to Outfile
    - ok print header
    - print list of parts
    - print part sizes
    - print title
    - print IDs
    - print name (name or ID inclusive)
    - print fasta name
    - print fastq name
    - print names
    - print lengths
    - print total length
    - print mask
    - print total mask length
    - print 4bit
    - print dna buffer
    - print dna split into lanes
    - print dna buffer as fasta
    - print dna
    - print charcount
    - print fasta
*/
use std::fs::File;
use std::io::Seek;
use std::io::Read;
use std::io::SeekFrom;
use std::io::Error;
use std::io::Write;

use crate::util;

#[derive(Clone,Copy)]
pub enum SequenceType  {
    DNA = 0,
    RNA = 1,
    Protein = 2,
    Text = 3
}

pub struct Flags {
    extended_fmt: bool,
    has_title:bool ,
    has_id:bool ,
    has_comments:bool ,
    has_lengths:bool ,
    has_mask:bool ,
    has_sequence:bool ,
    has_quality:bool 
}

impl Flags {
    fn to_u8(&self) -> u8 {
        return (self.extended_fmt as u8) << 7 |
               (self.has_title as u8) << 6 |
               (self.has_id as u8) << 5 |
               (self.has_comments as u8) << 4 |
               (self.has_lengths as u8) << 3 |
               (self.has_mask as u8) << 2 |
               (self.has_sequence as u8) << 1 |
               (self.has_quality as u8) 
    }
    fn list_of_parts(&self) -> String {
        let part_names = vec!["Title","IDs","Names","Lengths","Mask","Data","Quality"];
        let mut listed_parts: Vec<usize> = Vec::new();
        if self.has_title {listed_parts.push(0)}
        if self.has_id {listed_parts.push(1)}
        if self.has_comments {listed_parts.push(2)}
        if self.has_lengths {listed_parts.push(3)}
        if self.has_mask {listed_parts.push(4)}
        if self.has_sequence {listed_parts.push(5)}
        if self.has_quality {listed_parts.push(6)}
        listed_parts.iter().map(|x| {part_names[*x]}).collect::<Vec<&str>>().join(",") 
    }
}
    
pub struct NAFHeader {
 format_version:u8   ,
 sequence_type:SequenceType   , 
 flags:Flags   , 
 name_sep:char ,
 line_length:u128 ,
 number_of_seq:u128 
}

impl NAFHeader {
    fn read_header(naffile:&mut File ) -> Result<NAFHeader, Error> {
        naffile.seek(SeekFrom::Start(0))?;
        let _descriptor = naffile.read(&mut [0; 3])?;
        let version = naffile.read(&mut [0; 1])?;
        let sequence_type = match naffile.read(&mut [0; 1])? {
            0 => SequenceType::DNA,
            1 => SequenceType::RNA,
            2 => SequenceType::Protein,
            3 => SequenceType::Text,
            _ => panic!("Non-supported sequence type")
        };
        let flag_byte = naffile.read(&mut [0; 1])?;
        let flags = Flags{
            extended_fmt: flag_byte & 0x80 > 0,
            has_title: flag_byte & 0x40 > 0,
            has_id: flag_byte & 0x20 > 0,
            has_comments: flag_byte & 0x10 > 0,
            has_lengths: flag_byte & 0x8 > 0,
            has_mask: flag_byte & 0x4 > 0,
            has_sequence: flag_byte & 0x2 > 0,
            has_quality: flag_byte & 0x1 > 0,
        };
        let name_sep = naffile.read(&mut [0; 1])? as u8;
        let line_length = util::read_number(naffile)?;
        let number_of_seq = util::read_number(naffile)?;
        Ok(NAFHeader{
          format_version: version as u8,
          sequence_type,
          flags,
          name_sep: name_sep as char,
          line_length,
          number_of_seq,
        })
    }
}

impl NAFHeader {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut header = Vec::<u8>::new();
        //r"\x01\xf9\xec".as_bytes().iter().map(|x: &u8| header.push(*x));
        for val in r"\x01\xf9\xec".as_bytes().iter() {
            header.push(*val);
        }
        header.push(self.format_version);
        header.push(self.sequence_type as u8);
        header.push(self.flags.to_u8());
        header.push(self.name_sep as u8);
        //util::write_number(self.line_length).iter().map(|x: &u8| header.push(*x));
        header.append(&mut util::write_number(self.line_length));
        //util::write_number(self.number_of_seq).iter().map(|x: &u8| header.push(*x));
        header.append(&mut util::write_number(self.number_of_seq));
        header
    }
}

struct NAFMetadata {
    filename:String ,
    is_writing:bool ,
    header:NAFHeader ,
    // implement indecies here too?
}

enum Compressed {
    Empty,
    Bytes(Vec<u8>),
    FourBit(Vec<u8>),
    Huffman{
        encoding:Vec<u8> ,
        data:Vec<u8> 
    },
}

impl Compressed {
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            Compressed::Empty => return vec![0],
            Compressed::Huffman{encoding: enc, data: data } => {
                let mut ret = enc.clone();
                ret.append(&mut data.clone());
                return ret
            },
            Compressed::Bytes(data) | Compressed::FourBit(data) => return data.clone(),
        }
    }
}

struct NAFDataBlock {
    original_size:u128 ,
    compressed_size:u128 ,
    compressed_data:Compressed 
}

/*
impl NAFDataBlock {
    fn to_bytes(&self) -> Vec<u8> {
        let mut ret = String::new();
        ret.push_str(util::write_number(self.original_size).as_str());
        ret.push_str(util::write_number(self.compressed_size).as_str());
        ret.push_str(self.compressed_data.to_string().as_str());
        return ret
    }
}*/

impl NAFDataBlock {
    fn to_bytes(&self) -> Vec<u8> {
        return self.compressed_data.to_bytes()
    }
}

struct Title(String);
struct IDs(NAFDataBlock);
struct Comments(NAFDataBlock);
struct Lengths(NAFDataBlock);
struct Mask(NAFDataBlock);
struct Sequence(NAFDataBlock);
struct Quality(NAFDataBlock);

pub fn write_header(header: NAFHeader, mut output: File) {
    output.write(header.to_bytes().as_slice());
}

pub fn write_list_of_parts(flags: Flags, output: File) {
    
}