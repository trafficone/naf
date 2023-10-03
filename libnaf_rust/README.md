# LibNAF

A library in Rust to implement compression/extraction of the NAF file protocol.

This library will be the basis for the next versions of `ennaf` and `unnaf` CLI utilities, 
as well as providing a way for developers to implement NAF support in their own software.

By default LibNAF performs lazy execution to minimize unnecessary usage of resources.
Eager execution will be supported at a future time.

## Datatypes

### NAF Object

A representation of the NAF file. Holds data intrinsic to NAF, such as a *Header*, 
as well as the NAF *Data Block* 
objects, such as *IDs*, *Sequence*, or *Quality*.

The NAF object can be created from NAF, FASTA, and FASTQ files
using the following methods:
- `NAF.Open_NAF(File)` 
- `NAF.Open_FASTA(File)` will create NafObj that lazy loads from FASTA
- `NAF.Open_FASTQ(File)`
If you're not sure which to use, use `NAF.detect_input_format(File)`

Once instantiated, the NAF object can be read from directly, or can write to a file.
- `naf.read_next_record() -> Record`
- `naf.read_next_sequence() -> Sequence`
- `naf.write_naf(File)`
- `naf.read_FASTA(*buffer)` # force eager reading of FASTA into buffer
- `naf.read_FASTQ(*buffer)`
- `naf.write(File)`
- `naf.export_fasta(File)`
- `naf.export_fastq(File)`

For simplicity's sake, NAF-specific parts are accessible directly.
- `naf.read_header() -> Header`
- `naf.load_ids() -> IDs`
- `naf.load_names() -> Vec<String>`
- `naf.load_compressed_sequence() -> Sequence` # Guarantees stored compressed - useful when reading FASTA/Q

There are metadata extraction functions as well. These all return a String representation of the data.
- `naf.print_ids()`
- `naf.print_names()`
- `naf.print_lengths()`
- `naf.print_total_length()`
- `naf.print_mask()`
- `naf.print_total_mask_length()`
- `naf.print_4bit()`
- `naf.print_dna()`
- `naf.print_fastq()`

#### Record

The *Record* object contains the ID, and sequence from a FASTA record, as well as the comment 
and quality from a FASTQ record.

#### Sequence

The *Sequence* object contains the data for a sequence. By default, it is stored in the fastest 
way possible (i.e. plaintext when reading FASTA). However, if it is stored compressed, then the
`seq.read_buffer(buflen: usize)` will uncempress and write in one move. A Sequence is a special
kind of DataBlock file type

#### DataBlock

NAF Stores data in blocks based on the information stored. Each block has three components:
- Raw Size in bytes
- Compressed Size in bytes
- Compressed data
