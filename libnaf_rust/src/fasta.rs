//- Interface between a NAF object and a FASTA file.
use crate::nafobj::{NAFObject, NAFVersion, SequenceType};
use bio::io::fasta; // provides support for FASTA-specific IO
use std::fs::File;

/// Fasta-specific errors
pub enum FastaError {
    /// If a Fasta contains an unexpected character
    /// especially at the start of a line
    InvalidFasta,
    /// If a character cannot be represented as a nucleotide or AA.
    /// This may be recoverable if the type is TEXT
    InvalidNucOrAA,
}

pub fn naf_from_fasta(fasta_file: File, buffer_size: usize) -> Result<NAFObject, FastaError> {
    // buffered reader
    let buffer = std::io::BufReader::with_capacity(buffer_size, fasta_file);
    let reader = fasta::Reader::from_bufread(buffer);
    let mut nafobj = NAFObject::new(NAFVersion::VExperimental, SequenceType::DNA);
    for record in reader.records() {
        let record = record.expect("Error reading record");
        nafobj
            .add_sequence(record.id(), record.seq())
            .expect("Could not add sequence to object")
    }
    Ok(nafobj)
}
