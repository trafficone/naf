#ifndef NAFIO_H_
#define NAFIO_H_

struct flag_struct {
    unsigned char has_quality : 1;
    unsigned char has_data : 1;
    unsigned char has_mask : 1;
    unsigned char has_lengths : 1;
    unsigned char has_names : 1;
    unsigned char has_ids : 1;
    unsigned char has_title : 1;
    unsigned char has_extended : 1;
};

enum seq_type { seq_type_dna, seq_type_rna, seq_type_protein, seq_type_text };

typedef struct {
    unsigned char format_version;
    enum seq_type in_seq_type;
    struct flag_struct flags;
    unsigned char name_separator;
    long long line_length; // VARIABLE LENGTH
    long long  sequence_count; // VARIABLE LENGTH
} Header;

typedef struct {
    long long original_size; // VARIABLE LENGTH
    long long compressed_size; // VARIABLE LENGTH
    char * data;
} Metadata;
#endif //NAFIO_H_
