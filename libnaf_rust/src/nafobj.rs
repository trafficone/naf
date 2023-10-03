use self::varlenint::VarlenInt;
use snafu::prelude::*;
use std::fs::File;
mod datablock;
pub use self::datablock::NAFDataBlock;
use datablock::Compressed;
mod nafheader;
pub use self::nafheader::NAFHeader;
mod varlenint;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum NAFVersion {
    V1 = 1,
    V2 = 2,
    V3 = 3,
    VUnknown = 254,
    VExperimental = 255,
}

impl NAFVersion {
    pub fn from_u8(input: u8) -> NAFVersion {
        match input {
            1 => NAFVersion::V1,
            2 => NAFVersion::V2,
            255 => NAFVersion::VExperimental,
            _ => NAFVersion::VUnknown,
        }
    }
}

/// Describe the sequence of data for a particular field.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SequenceType {
    DNA = 0,
    RNA = 1,
    Protein = 2,
    Text = 3,
}

#[repr(C)]
struct NAFMetadata {
    filename: Option<String>,
    file: Option<File>,
    is_writing: bool,
    header: nafheader::NAFHeader,
}

impl NAFMetadata {
    pub fn new(version: NAFVersion, sequence_type: SequenceType) -> NAFMetadata {
        let header = match nafheader::NAFHeader::new(version, sequence_type) {
            Ok(headvalue) => headvalue,
            Err(reason) => panic!("Could not create NAF object: {}", reason),
        };
        NAFMetadata {
            filename: None,
            file: None,
            is_writing: true,
            header,
        }
    }
}

#[derive(Debug, Snafu)]
pub enum NAFError {
    #[snafu(display("NAF Format is Invalid"))]
    InvalidNAF,
    #[snafu(display("NAF Block Type is Invalid"))]
    InvalidNAFBlockType,
    #[snafu(display("Error occurred during compression/decompression: {}", source))]
    NAFCompressionError { source: std::io::Error },
    #[snafu(display("Unable to FileIO with NAF File"))]
    NAFFileError,
    #[snafu(display("No such data block found for NAF object"))]
    NoSuchNAFBlock,
    #[snafu(display("Error from outside NAF ecosystem: {}", source))]
    NonNAFError { source: std::io::Error },
}

#[repr(C)]
pub struct NAFObject {
    metadata: NAFMetadata,
    datablocks: Vec<NAFDataBlock>,
}

impl NAFObject {
    pub fn new(version: NAFVersion, sequence_type: SequenceType) -> NAFObject {
        NAFObject {
            metadata: NAFMetadata::new(version, sequence_type),
            datablocks: Vec::new(),
        }
    }

    pub fn open_file(filename: String) -> Result<NAFObject, NAFError> {
        let mut naffile = File::open(&filename).expect("Could not open file"); //NAFError::NAFFileError);
        let header = nafheader::NAFHeader::read_header(&mut naffile).context(NonNAFSnafu)?;
        Ok(NAFObject {
            metadata: NAFMetadata {
                filename: Some(filename),
                file: Some(naffile),
                is_writing: false,
                header,
            },
            datablocks: Vec::new(),
        })
    }

    pub fn save_to_file(&self, output_filename: String) -> Result<(), std::io::Error> {
        //let mut naffile = File::open(output_filename).expect("Could not open file for writing");
        // write header
        std::fs::write(&output_filename, self.metadata.header.to_bytes()).unwrap();
        // write data blocks in order
        self.datablocks
            .iter()
            .map(|x| std::fs::write(&output_filename, x.to_bytes()))
            .collect::<Result<(), std::io::Error>>()
    }

    pub fn read_header(&self) -> &nafheader::NAFHeader {
        &self.metadata.header
    }

    pub fn load_names(&self) -> Vec<String> {
        todo!("Produce a Vec<String> of names from the IDs datablock");
    }

    pub fn load_ids(&self) -> Result<NAFDataBlock, NAFError> {
        for block in &self.datablocks {
            match block {
                NAFDataBlock::IDs(_) => return Ok(block.clone()),
                _ => continue,
            }
        }
        Err(NAFError::NoSuchNAFBlock)
    }

    pub fn load_length(&self) -> Result<NAFDataBlock, NAFError> {
        for block in &self.datablocks {
            match block {
                NAFDataBlock::Lengths(_) => return Ok(block.clone()),
                _ => continue,
            }
        }
        Err(NAFError::NoSuchNAFBlock)
    }

    pub fn load_compressed_sequence(&self) -> Result<NAFDataBlock, NAFError> {
        for block in &self.datablocks {
            match block {
                NAFDataBlock::Sequence(_) => return Ok(block.clone()),
                _ => continue,
            }
        }
        Err(NAFError::NoSuchNAFBlock)
    }

    pub fn add_sequence(&mut self, id: &str, seq: &[u8]) -> Result<(), NAFError> {
        let mut ids = self.load_ids().unwrap();
        if let NAFDataBlock::IDs(_names) = &ids {
            let new_id = Compressed::Bytes(id.as_bytes().to_vec());
            ids.append(new_id)?;
        }
        todo!("This won't work because it requires the entire sequence to be sucked into memory.")
    }

    pub fn print_ids(&self) -> Result<String, NAFError> {
        for block in &self.datablocks {
            match block {
                NAFDataBlock::IDs(_) => block.print()?,
                _ => continue,
            };
        }
        Err(NAFError::NoSuchNAFBlock)
    }
    pub fn print_names(&self) -> Result<String, NAFError> {
        for block in &self.datablocks {
            match block {
                NAFDataBlock::Comments(_) => block.print()?,
                _ => continue,
            };
        }
        Err(NAFError::NoSuchNAFBlock)
    }
    pub fn print_lengths(&self) -> Result<String, NAFError> {
        for block in &self.datablocks {
            match block {
                NAFDataBlock::Lengths(_) => block.print()?,
                _ => continue,
            };
        }
        Err(NAFError::NoSuchNAFBlock)
    }
    pub fn print_total_length(&self) -> Result<String, NAFError> {
        for block in &self.datablocks {
            match block {
                NAFDataBlock::Lengths(lengths) => lengths
                    .numbers_into_iter()?
                    .fold(VarlenInt::new(0), |a, b| (a + *b))
                    .to_string(),
                _ => continue,
            };
        }
        Err(NAFError::NoSuchNAFBlock)
    }
    pub fn print_mask(&self) -> Result<String, NAFError> {
        for block in &self.datablocks {
            match block {
                NAFDataBlock::Mask(_) => block.print()?,
                _ => continue,
            };
        }
        Err(NAFError::NoSuchNAFBlock)
    }
    pub fn print_total_mask_length(&self) -> Result<String, NAFError> {
        for block in &self.datablocks {
            match block {
                NAFDataBlock::Mask(lengths) => lengths
                    .numbers_into_iter()?
                    .fold(VarlenInt::new(0), |a, b| (a + *b))
                    .to_string(),
                _ => continue,
            };
        }
        Err(NAFError::NoSuchNAFBlock)
    }
    pub fn print_4bit(&self) -> Result<Vec<u8>, NAFError> {
        for block in &self.datablocks {
            match block {
                NAFDataBlock::Sequence(_) => {
                    return block.print_4bit();
                }
                _ => continue,
            };
        }
        Err(NAFError::NoSuchNAFBlock)
    }
    pub fn print_fastq(&self) -> String {
        todo!("Print to FASTQ - Did I already implement this? I feel like I did, but maybe not?")
    }
}
