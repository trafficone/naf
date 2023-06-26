#ifndef LIBNAF
#define LIBNAF

#include <stdint.h>
#import <stdio.h>

// File IO
FILE open_input_file(char **s);
FILE open_output_file(char **s);
char *detect_temp_directory();
char *make_temp_prefix();

typedef struct _custom_error Error;

int32_t close_output_file(FILE);
int32_t close_output_file_and_set_stat(FILE);

// FASTA/Q Formatting
typedef struct _names Names;
typedef struct _sequence Sequence;
typedef struct _fasta Fasta;
typedef struct _fastq Fastq;

Fasta *read_fasta_entry(FILE, int);
char *write_fasta_entry(Fasta **f);
Fastq *read_fastq_entry(FILE, int);
char *write_fastq_entry(Fastq **f);
char *detect_input_format(FILE);

// NAF Formatting
typedef struct _header Header;
typedef struct _ids IDs;
typedef struct _lengths Lengths;
typedef struct _compressor Compressor;
typedef struct _mask Mask;

Header *read_header(FILE, int);
IDs *load_ids(FILE, int);
char *load_names(FILE, int);
Lengths *load_length(FILE, int);
char *load_mask(FILE, int);
Sequence *load_compressed_sequence(FILE, int);

// Util
long long read_number(FILE, int);
char *write_number(long long);
void die(char *);
void atexit();
char *put_magic_number();

// UnNAF methods
typedef struct _decompressor Decompressor;

char *print_ids(IDs *);
char *print_names(Names *);
char *print_lengths(Lengths *);
char *print_total_length(Lengths *);
char *print_mask(Mask *);
char *print_total_mask_length(Mask *);
char *print_4bit(Sequence *);
char *print_dna(Sequence *);
char *print_fastq(Fastq *);

Decompressor *initialize_input_decompression();
Decompressor *initialize_quality_file_decompression();

#endif
