#include <stdio.h>
#include <stdbool.h>
//#include <zstd.h>

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

static bool success = false;
static bool created_output_file = false;

