#ifndef BIBLE_CORE_H
#define BIBLE_CORE_H

#include <stddef.h>

typedef struct {
    unsigned int chapter;
    unsigned int verse;
    const char *text;
    size_t text_length;
} BibleVerse;

/* Parse a line shaped like: "3 In the beginning". */
int bible_core_parse_line(const char *line, BibleVerse *verse);

#endif
