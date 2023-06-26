#include "platform.h"

typedef enum { UNDECIDED, FORMAT_NAME, PART_LIST, PART_SIZES, NUMBER_OF_SEQUENCES,
               TITLE, IDS, NAMES, LENGTHS, TOTAL_LENGTH, MASK, TOTAL_MASK_LENGTH,
               FOUR_BIT,
               DNA, MASKED_DNA, UNMASKED_DNA,
               SEQ, SEQUENCES, CHARCOUNT,
               FASTA, MASKED_FASTA, UNMASKED_FASTA,
               FASTQ
             } OUTPUT_TYPE;
static void compressor_done(compressor_t *w);

// file functions
static void open_input_file(void);
static void close_input_file(void);
static void close_output_file(void);
static void close_output_file_and_set_stat(void);

static void init_encoders(void);
