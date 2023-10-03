//- The NAF Header object encodes the metadata for a NAF
//- object, including the fields contained within, as well as
//- the description of the object
use crate::nafobj::varlenint;
use crate::nafobj::File;
use crate::nafobj::NAFDataBlock;
use crate::nafobj::NAFError;
use crate::nafobj::NAFVersion;
use crate::nafobj::SequenceType;
use std::io::{Error, Read, Seek, SeekFrom};

/// Every NAF object has eight flags which describe the contents of the object
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Flags {
    /// Is the object using extended format?
    extended_fmt: bool,
    /// Does the object have a Title field?
    has_title: bool,
    /// Does the object have an IDs field?
    has_id: bool,
    /// Does the object have a comments field?
    has_comments: bool,
    /// Does the object have a lengths field?
    has_lengths: bool,
    /// Does the object have a mask field?
    has_mask: bool,
    /// Does the object have a sequence field?
    has_sequence: bool,
    /// Does the object have a quality field?
    pub has_quality: bool,
}

impl Flags {
    pub fn new() -> Flags {
        Flags {
            extended_fmt: false,
            has_title: false,
            has_id: false,
            has_comments: false,
            has_lengths: false,
            has_mask: false,
            has_sequence: false,
            has_quality: false,
        }
    }
    /// create list of NAFDataBlock types to populate
    pub fn to_data_blocks(&self) -> Vec<NAFDataBlock> {
        let naf_data_blocks = vec![
            NAFDataBlock::Title,
            NAFDataBlock::IDs,
            NAFDataBlock::Comments,
            NAFDataBlock::Lengths,
            NAFDataBlock::Mask,
            NAFDataBlock::Sequence,
            NAFDataBlock::Quality,
        ];
        self.list_of_part_ids()
            .iter()
            .map(|x| naf_data_blocks[*x])
            .collect::<Vec<NAFDataBlock>>()
    }
    /// Convert the Flags to a byte
    fn to_u8(&self) -> u8 {
        return (self.extended_fmt as u8) << 7
            | (self.has_title as u8) << 6
            | (self.has_id as u8) << 5
            | (self.has_comments as u8) << 4
            | (self.has_lengths as u8) << 3
            | (self.has_mask as u8) << 2
            | (self.has_sequence as u8) << 1
            | (self.has_quality as u8);
    }
    fn list_of_part_ids(&self) -> Vec<usize> {
        let mut listed_parts: Vec<usize> = Vec::new();
        if self.has_title {
            listed_parts.push(0)
        }
        if self.has_id {
            listed_parts.push(1)
        }
        if self.has_comments {
            listed_parts.push(2)
        }
        if self.has_lengths {
            listed_parts.push(3)
        }
        if self.has_mask {
            listed_parts.push(4)
        }
        if self.has_sequence {
            listed_parts.push(5)
        }
        if self.has_quality {
            listed_parts.push(6)
        }
        listed_parts
    }
    /// Get a String description of the parts in an object
    fn list_of_parts(&self) -> String {
        let part_names = vec![
            "Title", "IDs", "Names", "Lengths", "Mask", "Data", "Quality",
        ];
        self.list_of_part_ids()
            .iter()
            .map(|x| part_names[*x])
            .collect::<Vec<&str>>()
            .join(",")
    }
}

/// A NAF object has a header which includes much of the metadata for the file
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NAFHeader {
    /// An integer describing the format version to use
    /// (currently only 1 is supported)
    pub format_version: NAFVersion,
    /// Sequence type for object.
    sequence_type: SequenceType,
    /// Flags for object
    pub flags: Flags,
    /// Character which separates names
    name_sep: char,
    /// Original FASTA line length
    line_length: varlenint::VarlenInt,
    /// Number of sequences compressed in file
    number_of_seq: varlenint::VarlenInt,
}

impl NAFHeader {
    pub fn new(version: NAFVersion, sequence_type: SequenceType) -> Result<NAFHeader, NAFError> {
        if version == NAFVersion::V1 && sequence_type != SequenceType::DNA {
            return Err(NAFError::InvalidNAF);
        }
        Ok(NAFHeader {
            format_version: version,
            sequence_type,
            flags: Flags::new(),
            name_sep: ':',
            line_length: varlenint::VarlenInt::new(80),
            number_of_seq: varlenint::VarlenInt::new(0),
        })
    }
    /// Read a NAFHeader from a file
    pub fn read_header(naffile: &mut File) -> Result<NAFHeader, Error> {
        naffile.seek(SeekFrom::Start(0))?;
        let mut descriptor: [u8; 3] = [0; 3];
        let descreadlen = naffile.read(&mut descriptor)?;
        if descreadlen != 3 || descriptor != [0x01, 0xf9, 0xec] {
            return Err(Error::new(
                std::io::ErrorKind::InvalidData,
                "File does not match NAF format",
            ));
        }
        let mut versbuf: [u8; 1] = [0];
        let _ = naffile.read(&mut versbuf)?;
        let version = NAFVersion::from_u8(versbuf[0]);
        let sequence_type: SequenceType;
        if version == NAFVersion::V1 {
            sequence_type = SequenceType::DNA;
        } else {
            let mut sequence_buf: [u8; 1] = [0];
            let _ = naffile.read(&mut sequence_buf)?;
            sequence_type = match sequence_buf[0] {
                0 => SequenceType::DNA,
                1 => SequenceType::RNA,
                2 => SequenceType::Protein,
                3 => SequenceType::Text,
                _ => {
                    return Err(Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Non-supported sequence type",
                    ))
                }
            };
        }
        let mut flag_byte: [u8; 1] = [0];
        let _ = naffile.read(&mut flag_byte)?;
        let flags = Flags {
            extended_fmt: (flag_byte[0] >> 7) & 1 == 1,
            has_title: (flag_byte[0] >> 6) & 1 == 1,
            has_id: (flag_byte[0] >> 5) & 1 == 1,
            has_comments: (flag_byte[0] >> 4) & 1 == 1, // comments = names
            has_lengths: (flag_byte[0] >> 3) & 1 == 1,
            has_mask: (flag_byte[0] >> 2) & 1 == 1,
            has_sequence: (flag_byte[0] >> 1) & 1 == 1,
            has_quality: (flag_byte[0] & 1) == 1,
        };
        let mut name_sep_buf: [u8; 1] = [0];
        let _ = naffile.read(&mut name_sep_buf)? as u8;
        let line_length = varlenint::VarlenInt::read_from_file(naffile)?;
        let number_of_seq = varlenint::VarlenInt::read_from_file(naffile)?;
        Ok(NAFHeader {
            format_version: version,
            sequence_type,
            flags,
            name_sep: name_sep_buf[0] as char,
            line_length,
            number_of_seq,
        })
    }

    /// convert NAFHeader to bytes that can be written to a file
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut header = Vec::<u8>::new();
        //r"\x01\xf9\xec".as_bytes().iter().map(|x: &u8| header.push(*x));
        for val in r"\x01\xf9\xec".as_bytes().iter() {
            header.push(*val);
        }
        header.push(self.format_version as u8);
        header.push(self.sequence_type as u8);
        header.push(self.flags.to_u8());
        header.push(self.name_sep as u8);
        //util::write_number(self.line_length).iter().map(|x: &u8| header.push(*x));
        //header.append(&mut util::write_number(self.line_length));
        header.append(&mut self.line_length.as_bytes());
        //util::write_number(self.number_of_seq).iter().map(|x: &u8| header.push(*x));
        //header.append(&mut util::write_number(self.number_of_seq));
        header.append(&mut self.number_of_seq.as_bytes());
        header
    }
}

#[cfg(test)]
mod tests {
    use crate::nafobj::nafheader::Flags;
    use crate::nafobj::nafheader::NAFHeader;
    use crate::nafobj::varlenint::VarlenInt;
    use crate::nafobj::{NAFVersion, SequenceType};
    use std::fs::File;
    use std::path::Path;
    #[test]
    fn test_read_header_from_file() {
        let mut naf_file = match File::open(&Path::new(
            "/home/jschlesi/workspace/MyGithub/naf/libnaf_rust/test.fa.naf",
        )) {
            Err(reason) => panic!("Test Failure: couldn't open NAF file: {}", reason),
            Ok(file) => file,
        };
        let expect_header = NAFHeader {
            format_version: NAFVersion::V1,
            sequence_type: SequenceType::DNA,
            name_sep: ' ',
            line_length: VarlenInt::new(23),
            number_of_seq: VarlenInt::new(1),
            flags: Flags {
                extended_fmt: false,
                has_title: false,
                has_id: true,
                has_lengths: true,
                has_mask: true,
                has_comments: true,
                has_sequence: true,
                has_quality: false,
            },
        };
        assert_eq!(
            expect_header,
            NAFHeader::read_header(&mut naf_file).unwrap()
        );
    }
}
