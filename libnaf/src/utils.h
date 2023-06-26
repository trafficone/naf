#ifndef NAFUTILS_H_
#define NAFUTILS_H_
#include <stdio.h>
#include <stdbool.h>
unsigned char code_to_nuc[16] = {'-','T','G','K','C','Y','S','B','A','W','R','D','M','H','V','N'};
unsigned short codes_to_nucs[256];

void die(const char *format, ...);
void malloc_or_die(const size_t size);
bool string_has_characters_unsafe_in_file_names(char *str);
void init_tables(void);
void fread_or_die(void *ptr, size_t element_size, size_t n_elements, FILE *F);
void fwrite_or_die(const void *ptr, size_t element_size, size_t n_elements, FILE *F);
unsigned char fgetc_or_incomplete(FILE *F);
void fputc_or_die(int c, FILE *F);
void fflush_or_die(FILE *F);
void fclose_or_die(FILE *F);
unsigned long long read_number(FILE *F);
#endif //NAFUTILS_H_
