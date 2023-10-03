//! `LibNAF` implemented in Rust
//! Authors: Jason Schlesinger, Kirill Kryukov
//! Acknowldgements to MEXT, DDBJ, and NIG for their support
//!
//! This library implements compression and decompression of the
//! NAF Filetype. It is provided as-is and with no warranty.
/* Base lib file exposed for consumption */
#![warn(
    missing_docs,
    clippy::unwrap_used,
    clippy::pedantic,
    clippy::expect_used
)]

use std::fs::File;

use nafobj::NAFDataBlock;
use nafobj::NAFHeader;
use nafobj::NAFObject;
mod fasta;
mod files;
mod libennaf;
mod nafobj;
mod util;

// File IO Functions
/// Try to open a NAF file from a file name.
#[no_mangle]
pub extern "C" fn open_NAF(filename: String) -> Option<NAFObject> {
    match NAFObject::open_file(filename) {
        Ok(obj) => Some(obj),
        Err(_) => None,
    }
}

/// Open a FASTA file and create a NAFObject that populates
/// lazily given on how it is read.
#[no_mangle]
pub extern "C" fn open_FASTA(filename: String) -> NAFObject {
    todo!("Open a filename and return a readable file.");
}

/// Open a FASTQ file and create a NAFObject that populates
/// lazily given how it is read.
#[no_mangle]
pub extern "C" fn open_FASTQ(filename: String) -> NAFObject {
    todo!("Open a filename and return a readable file.");
}

/// Try to determine a file format heuristically from the
/// content of a file. This is only useful if the input isn't
/// available apriori, a scenario that should be avoided.
#[no_mangle]
pub extern "C" fn detect_input_format(test_file: File) -> String {
    todo!("Detect file type from content alone");
}

/// Read the header data from a NAF Object.
#[no_mangle]
pub extern "C" fn read_header(naf: NAFObject) -> NAFHeader {
    *naf.read_header()
}

#[no_mangle]
pub extern "C" fn read_ids(naf: NAFObject) -> NAFDataBlock {
    match naf.load_ids() {
        Ok(ids) => ids,
        Err(_) => NAFDataBlock::new_ids(),
    }
}

#[no_mangle]
pub extern "C" fn write_NAF(naf: NAFObject, outputfilename: String) {
    naf.save_to_file(outputfilename);
}

// FASTA/FASTQ Formatting
#[repr(C)]
pub struct Names {
    names: Vec<String>,
}

#[repr(C)]
pub struct Sequence {
    sequence: Vec<String>,
}

#[repr(C)]
pub struct Fasta {
    names: Names,
    sequence: Sequence,
}

#[repr(C)]
pub struct Fastq {
    names: Names,
    sequence: Sequence,
}

#[no_mangle]
pub fn read_fasta_entry(fasta_file: File, buffer_size: usize) -> &'static mut &'static mut Fasta {
    todo!("Read an entry from a FASTA file and return a pointer to the FASTA struct created");
}

#[no_mangle]
pub fn write_fasta_entry(fasta_entry: &mut &mut Fasta) -> String {
    todo!("Write the String equivalent of the fasta entry")
}
