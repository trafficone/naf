/*
 * NAF decompressor
 * Copyright (c) 2018-2019 Kirill Kryukov
 * See README.md and LICENSE files of this repository
 */

#define VERSION "1.0.0"
#define DATE "2019-01-25"
#define COPYRIGHT_YEARS "2018-2019"

#define NDEBUG

#include <assert.h>
#include <ctype.h>
#include <stdlib.h>
#include <stdbool.h>
#include <string.h>
#include <stdio.h>
#include <unistd.h>
#include <zstd.h>

#include "utils.c"

typedef enum { UNDECIDED, FORMAT_NAME, PART_LIST, PART_SIZES, NUMBER_OF_SEQUENCES,
               TITLE, IDS, NAMES, LENGTHS, TOTAL_LENGTH, MASK, TOTAL_MASK_LENGTH,
               FOUR_BIT,
               DNA, MASKED_DNA, UNMASKED_DNA,
               SEQ,
               FASTA, MASKED_FASTA, UNMASKED_FASTA,
               FASTQ
             } OUTPUT_TYPE;

static OUTPUT_TYPE out_type = UNDECIDED;

enum { seq_type_dna, seq_type_rna, seq_type_protein, seq_type_text };
static int in_seq_type = seq_type_dna;
static const char *in_seq_type_name = "DNA";

static bool verbose = false;
static bool use_mask = true;

static char *in_file_path = NULL;
static FILE *IN = NULL;

static unsigned char format_version = 1;
static unsigned char name_separator = ' ';

static int has_title = 0;
static int has_ids = 0;
static int has_names = 0;
static int has_lengths = 0;
static int has_mask = 0;
static int has_data = 0;
static int has_quality = 0;
static unsigned long long max_line_length;
static unsigned long long N;


static char *ids_buffer = NULL;
static unsigned char *compressed_ids_buffer = NULL;
static char **ids = NULL;

static char *names_buffer = NULL;
static unsigned char *compressed_names_buffer = NULL;
static char **names = NULL;

static unsigned int *lengths_buffer = NULL;
static unsigned char *compressed_lengths_buffer = NULL;
static unsigned long long n_lengths = 0;

static unsigned long long mask_size = 0;
static unsigned char *mask_buffer = NULL;
static unsigned char *compressed_mask_buffer = NULL;

static unsigned long long total_seq_length = 0;
static unsigned long long compressed_seq_size = 0;
static unsigned char *compressed_seq_buffer = NULL;
static unsigned long long compressed_seq_pos = 0;

static unsigned long long total_quality_length = 0;
static unsigned long long compressed_quality_size = 0;


static size_t in_buffer_size = 0;
static char *in_buffer = NULL;

static size_t out_buffer_size = 0;
static char *out_buffer = NULL;

static size_t mem_out_buffer_size = 0;
static unsigned char *mem_out_buffer = NULL;

static size_t out_print_buffer_size = 0;
static unsigned char *out_print_buffer = NULL;

static ZSTD_DStream *input_decompression_stream = NULL;
static size_t file_bytes_to_read;
static ZSTD_inBuffer zstd_file_in_buffer;

static ZSTD_DStream *memory_decompression_stream = NULL;
static size_t memory_bytes_to_read;
static ZSTD_inBuffer zstd_mem_in_buffer;


static unsigned long long cur_seq_index = 0;

static unsigned char *dna_buffer = NULL;
static size_t dna_buffer_size = 0;
static size_t dna_buffer_flush_size = 0;
static unsigned dna_buffer_pos = 0;
static unsigned dna_buffer_filling_pos = 0;
static unsigned dna_buffer_printing_pos = 0;
static unsigned dna_buffer_remaining = 0;

static char *quality_buffer = NULL;
static size_t quality_buffer_size = 0;
static size_t quality_buffer_flush_size = 0;
static unsigned quality_buffer_filling_pos = 0;
static unsigned quality_buffer_printing_pos = 0;
static unsigned quality_buffer_remaining = 0;

static unsigned long long total_seq_n_bp_remaining = 0;

static unsigned long long cur_seq_len_index = 0;
static unsigned long long cur_seq_len_n_bp_remaining = 0;

static unsigned long long cur_qual_len_index = 0;

static unsigned long long cur_mask = 0;
static unsigned int cur_mask_remaining = 0;
static int mask_on = 0;

static unsigned long long cur_line_n_bp_remaining = 0;

static bool line_length_is_specified = false;
static unsigned long long requested_line_length = 0ull;


#include "input.c"
#include "output.c"
#include "output-fastq.c"


#define FREE(p) \
do { if ((p) != NULL) { free(p); (p) = NULL; } } while (0)


static void done(void)
{
    if (IN != NULL && IN != stdin) { fclose(IN); IN = NULL; }

    FREE(ids);
    FREE(ids_buffer);
    FREE(compressed_ids_buffer);

    FREE(names);
    FREE(names_buffer);
    FREE(compressed_names_buffer);

    FREE(lengths_buffer);
    FREE(compressed_lengths_buffer);

    FREE(mask_buffer);
    FREE(compressed_mask_buffer);

    FREE(compressed_seq_buffer);

    FREE(in_buffer);
    FREE(out_buffer);
    FREE(mem_out_buffer);
    FREE(out_print_buffer);
    FREE(input_decompression_stream);
    FREE(memory_decompression_stream);

    FREE(dna_buffer);
    FREE(quality_buffer);
}


static void set_out_type(OUTPUT_TYPE new_type)
{
    if (out_type != UNDECIDED) { fprintf(stderr, "Error: Only one output type should be specified\n"); exit(1); }
    out_type = new_type;
}


static void set_input_file_path(char *new_path)
{
    assert(new_path != NULL);

    if (in_file_path != NULL) { fprintf(stderr, "Error: Can process only one file at a time\n"); exit(1); }
    if (*new_path == '\0') { fprintf(stderr, "Error: empty input path specified\n"); exit(1); }
    in_file_path = new_path;
}


static void set_line_length(char *str)
{
    assert(str != NULL);

    char *end;
    long long a = strtoll(str, &end, 10);
    if (*end != '\0') { fprintf(stderr, "Can't parse the value of --line-length parameter\n"); exit(1); }
    if (a < 0ll) { fprintf(stderr, "Error: Negative line length specified\n"); exit(1); }

    char test_str[21];
    int nc = snprintf(test_str, 21, "%lld", a);
    if (nc < 1 || nc > 20 || strcmp(test_str, str) != 0) { fprintf(stderr, "Can't parse the value of --line-length parameter\n"); exit(1); }

    requested_line_length = (unsigned long long) a;
    line_length_is_specified = true;
}


static void show_version(void)
{
    fputs("unnaf - NAF decompressor, version " VERSION ", " DATE "\nCopyright (c) " COPYRIGHT_YEARS " Kirill Kryukov\n", stderr);
    if (verbose)
    {
        fprintf(stderr, "Built with zstd " ZSTD_VERSION_STRING ", using runtime zstd %s\n", ZSTD_versionString());
    }
}


static void show_help(void)
{
    fprintf(stderr,
        "Usage: unnaf [OUTPUT-TYPE] [file.naf]\n"
        "Options for selecting output type:\n"
        "  --format       - File format version\n"
        "  --part-list    - List of parts\n"
        "  --sizes        - Part sizes\n"
        "  --number       - Number of sequences\n"
        "  --title        - Dataset title\n"
        "  --ids          - Sequence ids (accession numbers)\n"
        "  --names        - Full sequence names (including ids)\n"
        "  --lengths      - Sequence lengths\n"
        "  --total-length - Sum of sequence lengths\n"
        "  --mask         - Masked region lengths\n"
        "  --4bit         - 4bit-encoded nucleotide sequence (binary data)\n"
        "  --seq          - Continuous concatenated sequence\n"
        "  --fasta        - FASTA-formatted sequences\n"
        "  --fastq        - FASTQ-formatted sequences\n"
        "Other options:\n"
        "  --line-length N - Use lines of width N for FASTA output\n"
        "  --no-mask      - Ignore mask\n"
        "  -h, --help     - Show help\n"
        "  -V, --version  - Show version\n"
    );
}


static void parse_command_line(int argc, char **argv)
{
    bool print_version = false;

    for (int i = 1; i < argc; i++)
    {
        if (argv[i][0] == '-')
        {
            if (argv[i][1] == '-')
            {
                if (i < argc - 1)
                {
                    if (!strcmp(argv[i], "--line-length")) { i++; set_line_length(argv[i]); continue; }
                }
                if (!strcmp(argv[i], "--format"           )) { set_out_type(FORMAT_NAME        ); continue; }
                if (!strcmp(argv[i], "--part-list"        )) { set_out_type(PART_LIST          ); continue; }
                if (!strcmp(argv[i], "--sizes"            )) { set_out_type(PART_SIZES         ); continue; }
                if (!strcmp(argv[i], "--number"           )) { set_out_type(NUMBER_OF_SEQUENCES); continue; }
                if (!strcmp(argv[i], "--title"            )) { set_out_type(TITLE              ); continue; }
                if (!strcmp(argv[i], "--ids"              )) { set_out_type(IDS                ); continue; }
                if (!strcmp(argv[i], "--names"            )) { set_out_type(NAMES              ); continue; }
                if (!strcmp(argv[i], "--lengths"          )) { set_out_type(LENGTHS            ); continue; }
                if (!strcmp(argv[i], "--total-length"     )) { set_out_type(TOTAL_LENGTH       ); continue; }
                if (!strcmp(argv[i], "--mask"             )) { set_out_type(MASK               ); continue; }
                if (!strcmp(argv[i], "--total-mask-length")) { set_out_type(TOTAL_MASK_LENGTH  ); continue; }
                if (!strcmp(argv[i], "--4bit"             )) { set_out_type(FOUR_BIT           ); continue; }
                if (!strcmp(argv[i], "--seq"              )) { set_out_type(SEQ                ); continue; }
                if (!strcmp(argv[i], "--fasta"            )) { set_out_type(FASTA              ); continue; }
                if (!strcmp(argv[i], "--fastq"            )) { set_out_type(FASTQ              ); continue; }
                if (!strcmp(argv[i], "--no-mask")) { use_mask = false; continue; }
                if (!strcmp(argv[i], "--help")) { show_help(); exit(0); }
                if (!strcmp(argv[i], "--verbose")) { verbose = true; continue; }
                if (!strcmp(argv[i], "--version")) { print_version = true; continue; }

                // Deprecated undocumented options.
                if (!strcmp(argv[i], "--dna"              )) { set_out_type(DNA                ); continue; }
                if (!strcmp(argv[i], "--masked-dna"       )) { set_out_type(MASKED_DNA); continue; }     // Instead use "--dna"
                if (!strcmp(argv[i], "--unmasked-dna"     )) { set_out_type(UNMASKED_DNA); continue; }   // Instead use "--dna --no-mask"
                if (!strcmp(argv[i], "--masked-fasta"     )) { set_out_type(MASKED_FASTA); continue; }   // Instead use "--fasta"
                if (!strcmp(argv[i], "--unmasked-fasta"   )) { set_out_type(UNMASKED_FASTA); continue; } // Instead use "--fasta --no-mask"
            }
            if (!strcmp(argv[i], "-h")) { show_help(); exit(0); }
            if (!strcmp(argv[i], "-V")) { print_version = true; continue; }

            fprintf(stderr, "Unknown or incomplete argument \"%s\"\n", argv[i]);
            exit(1);
        }
        set_input_file_path(argv[i]);
    }

    if (print_version)
    {
        show_version();
        exit(0);
    }
}


int main(int argc, char **argv)
{
    atexit(done);
    init_tables();

    parse_command_line(argc, argv);
    if (in_file_path == NULL && isatty(fileno(stdin)))
    {
        fprintf(stderr, "No input specified, use \"unnaf -h\" for help\n");
        exit(0);
    }

    if (in_file_path != NULL)
    {
        IN = fopen(in_file_path, "rb");
        if (IN == NULL) { fprintf(stderr, "Can't open input file\n"); exit(1); }
    }
    else
    {
        if (!freopen(NULL, "rb", stdin)) { fprintf(stderr, "Can't read input in binary mode\n"); exit(1); }
        IN = stdin;
    }

    if (out_type == FOUR_BIT && isatty(fileno(stdout)))
    {
        fprintf(stderr, "Won't write binary data to terminal\n");
        exit(1);
    }

    read_header();

    if (!has_quality && out_type == FASTQ)
    { 
        fprintf(stderr, "Error: FASTQ output requested, but input has no qualities\n");
        exit(1);
    }
    if ((out_type == DNA || out_type == MASKED_DNA || out_type == UNMASKED_DNA) && (in_seq_type != seq_type_dna))
    {
        fprintf(stderr, "Input has not DNA, but %s data\n", in_seq_type_name);
        exit(1);
    }
    if (out_type == FOUR_BIT && in_seq_type >= seq_type_protein)
    {
        fprintf(stderr, "Input has no 4-bit encoded data, but %s sequences\n", in_seq_type_name);
        exit(1);
    }

    if (out_type == UNDECIDED)
    {
        out_type = has_quality ? FASTQ : MASKED_FASTA;
    }

    if (out_type == FORMAT_NAME) { printf("NAF v.%d\n", format_version); exit(0); }
    if (out_type == PART_LIST) { print_list_of_parts_and_exit(); }

    max_line_length = read_number(IN);
    if (line_length_is_specified) { max_line_length = requested_line_length; }

    N = read_number(IN);
    if (out_type == NUMBER_OF_SEQUENCES) { printf("%llu\n", N); exit(0); }
    if (!N) { exit(0); }

    if (out_type == PART_SIZES) { print_part_sizes_and_exit(); }
    if (out_type == TITLE) { print_title_and_exit(); }

    skip_title();

    if (out_type == IDS) { print_ids_and_exit(); }
    if (out_type == NAMES) { print_names_and_exit(); }
    if (out_type == LENGTHS) { print_lengths_and_exit(); }
    if (out_type == TOTAL_LENGTH) { print_total_length_and_exit(); }
    if (out_type == MASK) { print_mask_and_exit(); }
    if (out_type == TOTAL_MASK_LENGTH) { print_total_mask_length_and_exit(); }
    if (out_type == FOUR_BIT) { print_4bit_and_exit(); }

    dna_buffer_flush_size = ZSTD_DStreamOutSize() * 2;
    dna_buffer_size = dna_buffer_flush_size * 2 + 10;
    dna_buffer = (unsigned char *)malloc(dna_buffer_size);
    if (!dna_buffer) { fprintf(stderr, "Can't allocate %zu bytes for dna buffer\n", dna_buffer_size); exit(1); }

    out_print_buffer_size = dna_buffer_size * 2;
    out_print_buffer = (unsigned char *)malloc(out_print_buffer_size);
    if (!out_print_buffer) { fprintf(stderr, "Can't allocate %zu bytes for dna buffer\n", out_print_buffer_size); exit(1); }

    if (out_type == DNA) { print_dna_and_exit(use_mask && has_mask); }
    if (out_type == SEQ) { print_dna_and_exit(use_mask && has_mask); }
    if (out_type == MASKED_DNA) { print_dna_and_exit(use_mask && has_mask); }
    if (out_type == UNMASKED_DNA) { print_dna_and_exit(0); }

    if (out_type == FASTA) { print_fasta_and_exit(use_mask && has_mask); }
    if (out_type == MASKED_FASTA) { print_fasta_and_exit(use_mask && has_mask); }
    if (out_type == UNMASKED_FASTA) { print_fasta_and_exit(0); }

    if (out_type == FASTQ) { print_fastq_and_exit(0); }

    return 0;
}
