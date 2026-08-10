#include "bible_core.h"

#include <ctype.h>
#include <stdlib.h>

int bible_core_parse_line(const char *line, BibleVerse *verse) {
    const char *cursor;
    char *end;

    if (line == NULL || verse == NULL) return 0;
    cursor = line;
    while (isspace((unsigned char)*cursor)) cursor++;
    if (!isdigit((unsigned char)*cursor)) return 0;

    verse->verse = (unsigned int)strtoul(cursor, &end, 10);
    if (end == cursor || !isspace((unsigned char)*end)) return 0;
    while (isspace((unsigned char)*end)) end++;
    if (*end == '\0') return 0;

    verse->chapter = 0;
    verse->text = end;
    verse->text_length = 0;
    while (end[verse->text_length] != '\0') verse->text_length++;
    return 1;
}
