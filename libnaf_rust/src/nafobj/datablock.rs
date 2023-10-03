//- Description of data blocks used in NAF files
//- Covers memory management, but not file operations
//- Utilized as part of compression, but should only be
//- randomly accessed from a NAF file

use super::varlenint::VarlenInt;
use super::InvalidNAFBlockTypeSnafu;
use crate::nafobj::NAFError;
use crate::util;
//use huffman_coding::{HuffmanReader, HuffmanTree, HuffmanWriter};
use core::slice::Iter;
use std::io::Read;

/// Object which stores data along with encoding
#[repr(C)]
#[derive(Debug, Clone)]
pub enum Compressed {
    /// When the data is empty, but a field exists
    Empty,
    /// no encoding is used for compressed data - guaranteed to encode data
    /// regardless of the original type
    Bytes(Vec<u8>),
    /// Sequences (Nucleotide and Amino) are encoded using four bits
    /// Provides a guaranteed 50% reduction in storage before compression
    FourBit(Vec<u8>),
    /// Sequences of integers encoded using the VarlenInt encoding
    /// Can provide significant storage savings on many integers < u32
    VarlenInts(Vec<VarlenInt>),
    // Huffman encoding, stored with index for portability.
    // Has approximately 50% savings on English text.
    // Offers variable storage reduction based on entropy
    // of source data
    // Huffman { index: HuffmanTree, data: Vec<u8> },
}

impl Compressed {
    /// Extract converted data as a Vec of bytes so that it can be written to a file
    fn to_bytes(&self) -> Vec<u8> {
        match self {
            Compressed::Empty => return vec![],
            /*Compressed::Huffman { index: enc, data } => {
                let mut ret = enc.to_table().to_vec();
                ret.append(&mut data.clone());
                return ret;
            }*/
            Compressed::VarlenInts(data) => {
                // There must be a cleaner way to concatenate some bytes
                return data.iter().map(|i| VarlenInt::as_bytes(i)).fold(
                    Vec::<u8>::new(),
                    |mut acc, mut x| {
                        acc.append(&mut x);
                        acc
                    },
                );
            }
            Compressed::Bytes(data) | Compressed::FourBit(data) => return data.clone(),
        }
    }

    /*fn huffman_code(data: Vec<u8>) -> Compressed {
        let tree = HuffmanTree::from_data(&data[0..1024]);
        let mut output = Vec::new();
        {
            use std::io::Write;
            let mut writer = HuffmanWriter::new(&mut output, &tree);
            let mut buffer: [u8; 1024];
            for x in data.chunks(1024) {
                writer.write(x);
            }
        }
        Compressed::Huffman {
            index: tree,your
            data: output,
        }
    }

    fn huffman_decode(enc: HuffmanTree, data: Vec<u8>, buffer: &mut Vec<u8>) {
        use std::io::{Cursor, Read};
        let cursor = Cursor::new(data);
        let mut reader = HuffmanReader::new(cursor, enc);
        reader.read_exact(buffer);
    }*/
}

/// All NAF Data Blocks have their original and compressed sizes, as well as their encoded data.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct _NAFDataBlock {
    original_size: VarlenInt,
    compressed_size: VarlenInt,
    compressed_data: Compressed,
}

impl _NAFDataBlock {
    /// Support for writing a NAF data block to bytes, thus to files
    fn to_bytes(&self) -> Vec<u8> {
        let mut output_bytes = Vec::<u8>::new();
        output_bytes.append(&mut self.original_size.as_bytes());
        output_bytes.append(&mut self.compressed_size.as_bytes());
        output_bytes.append(&mut self.compressed_data.to_bytes());
        output_bytes
    }

    pub fn print(&self) -> String {
        String::from_utf8_lossy(self.compressed_data.to_bytes().as_slice()).to_string()
    }

    pub fn numbers_into_iter(
        &self,
    ) -> Result<Iter<'_, crate::nafobj::varlenint::VarlenInt>, NAFError> {
        match self.get_compressed() {
            Compressed::VarlenInts(ints) => Ok(ints.into_iter()),
            _ => return InvalidNAFBlockTypeSnafu.fail(),
        }
    }

    fn get_compressed(&self) -> &Compressed {
        &self.compressed_data
    }
}

/// In practice, the information within a NAF data block comes
/// with the meta-information of what is contained within.
/// This data structure captures that, and allows for type-specific implementations.
#[repr(C)]
#[derive(Clone, Debug)]
pub enum NAFDataBlock {
    Title(_NAFDataBlock),
    IDs(_NAFDataBlock),
    Comments(_NAFDataBlock),
    Lengths(_NAFDataBlock),
    Mask(_NAFDataBlock),
    Sequence(_NAFDataBlock),
    Quality(_NAFDataBlock),
}

impl NAFDataBlock {
    pub fn new_ids() -> Self {
        NAFDataBlock::IDs(_NAFDataBlock {
            original_size: VarlenInt::new(0),
            compressed_size: VarlenInt::new(0),
            compressed_data: Compressed::Empty,
        })
    }
    pub fn from_reader(r: BufReader<File>, block_type: NAFDataBlock) -> Self {
        todo!("IDK what I'm doing here")
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            NAFDataBlock::Title(t) => t.to_bytes(),
            NAFDataBlock::IDs(ids) => ids.to_bytes(),
            NAFDataBlock::Comments(comments) => comments.to_bytes(),
            NAFDataBlock::Lengths(lengths) => lengths.to_bytes(),
            NAFDataBlock::Mask(mask) => mask.to_bytes(),
            NAFDataBlock::Sequence(seq) => seq.to_bytes(),
            NAFDataBlock::Quality(q) => q.to_bytes(),
        }
    }

    pub fn print(&self) -> Result<String, NAFError> {
        match self {
            NAFDataBlock::Title(t) => Ok(t.print()),
            NAFDataBlock::IDs(ids) => Ok(ids.print()),
            NAFDataBlock::Comments(comments) => Ok(comments.print()),
            NAFDataBlock::Lengths(lengths) => Ok(lengths.print()),
            NAFDataBlock::Mask(mask) => Ok(mask.print()),
            NAFDataBlock::Sequence(seq) => Ok(seq.print()),
            NAFDataBlock::Quality(q) => Ok(q.print()),
        }
    }
    pub fn print_4bit(&self) -> Result<Vec<u8>, NAFError> {
        match self {
            NAFDataBlock::Sequence(seq) => {
                let seq_data = seq.get_compressed();
                match seq_data {
                    Compressed::Empty => return Ok(Vec::<u8>::new()),
                    Compressed::FourBit(data) => return Ok(data.clone()),
                    Compressed::Bytes(data) => return Ok(util::chars_to_fourbits(data.to_vec())),
                    /* Compressed::Huffman { index: enc, data } => {
                        let mut bytes = Vec::new();
                        Compressed::huffman_decode(enc, data, &mut bytes);
                        return Ok(util::chars_to_fourbits(bytes));
                    }*/
                    Compressed::VarlenInts(_) => return InvalidNAFBlockTypeSnafu.fail(),
                }
            }
            _ => Err(NAFError::InvalidNAFBlockType),
        }
    }
    pub fn append(&mut self, new_data: Compressed) -> Result<(), NAFError> {
        todo!("Append compressed data to self")
    }
}

#[cfg(test)]
mod tests {
    use crate::nafobj::Compressed;
    #[test]
    fn test_compressed_to_bytes() {
        let empty_compressed = Compressed::Empty;
        assert_eq!(empty_compressed.to_bytes(), vec![]);
        let compressed_bytes =
            Compressed::Bytes(vec![0x32, 0x24, 0x15, 0xfa, 0xc2, 0x00, 0x00, 0x00, 0xa1]);
        assert_eq!(
            compressed_bytes.to_bytes(),
            vec![0x32, 0x24, 0x15, 0xfa, 0xc2, 0x00, 0x00, 0x00, 0xa1]
        );
    }
}
