#!/usr/bin/env python3
"""
Convert USFM (Unified Standard Format Markup) Bible files to kjv.txt format
Handles multiple USFM files in a directory with proper tag stripping
"""

import re
import sys
import os
from pathlib import Path

book_order = [
    "GEN", "EXO", "LEV", "NUM", "DEU", "JOS", "JDG", "RUT", "1SA", "2SA",
    "1KI", "2KI", "1CH", "2CH", "EZR", "NEH", "EST", "JOB", "PSA", "PRO",
    "ECC", "SNG", "ISA", "JER", "LAM", "EZK", "DAN", "HOS", "JOL", "AMO",
    "OBA", "JON", "MIC", "NAM", "HAB", "ZEP", "HAG", "ZEC", "MAL",
    "MAT", "MRK", "LUK", "JHN", "ACT", "ROM", "1CO", "2CO", "GAL", "EPH",
    "PHP", "COL", "1TH", "2TH", "1TI", "2TI", "TIT", "PHM", "HEB", "JAS",
    "1PE", "2PE", "1JN", "2JN", "3JN", "JUD", "REV"
]

book_names = {
    "GEN": "Genesis", "EXO": "Exodus", "LEV": "Leviticus", "NUM": "Numbers",
    "DEU": "Deuteronomy", "JOS": "Joshua", "JDG": "Judges", "RUT": "Ruth",
    "1SA": "1 Samuel", "2SA": "2 Samuel", "1KI": "1 Kings", "2KI": "2 Kings",
    "1CH": "1 Chronicles", "2CH": "2 Chronicles", "EZR": "Ezra", "NEH": "Nehemiah",
    "EST": "Esther", "JOB": "Job", "PSA": "Psalms", "PRO": "Proverbs",
    "ECC": "Ecclesiastes", "SNG": "Song of Solomon", "ISA": "Isaiah", "JER": "Jeremiah",
    "LAM": "Lamentations", "EZK": "Ezekiel", "DAN": "Daniel", "HOS": "Hosea",
    "JOL": "Joel", "AMO": "Amos", "OBA": "Obadiah", "JON": "Jonah", "MIC": "Micah",
    "NAM": "Nahum", "HAB": "Habakkuk", "ZEP": "Zephaniah", "HAG": "Haggai",
    "ZEC": "Zechariah", "MAL": "Malachi", "MAT": "Matthew", "MRK": "Mark",
    "LUK": "Luke", "JHN": "John", "ACT": "Acts", "ROM": "Romans",
    "1CO": "1 Corinthians", "2CO": "2 Corinthians", "GAL": "Galatians", "EPH": "Ephesians",
    "PHP": "Philippians", "COL": "Colossians", "1TH": "1 Thessalonians", "2TH": "2 Thessalonians",
    "1TI": "1 Timothy", "2TI": "2 Timothy", "TIT": "Titus", "PHM": "Philemon",
    "HEB": "Hebrews", "JAS": "James", "1PE": "1 Peter", "2PE": "2 Peter",
    "1JN": "1 John", "2JN": "2 John", "3JN": "3 John", "JUD": "Jude", "REV": "Revelation"
}

def clean_usfm_text(text):
    """Strip all USFM markers and clean up text"""
    # Remove footnotes first (they can contain complex content)
    # Handle \f ... \fr ... \ft ... \f* format
    text = re.sub(r'\\f\s+[^\\]*?\\fr\s+[^\\]*?\\ft\s+[^\\]*?\\f\*', '', text)
    # Handle simple \f ... \f* format
    text = re.sub(r'\\f[^\\]*?\\f\*', '', text)
    # Remove standalone footnote markers and references
    # Pattern: "word 1:1 The footnote text." or "word 1:1 The footnote text" followed by next word
    text = re.sub(r'\s+\d+:\d+\s+[^\\]*?(?=\\|\s+[a-z]|$)', '', text, flags=re.IGNORECASE)  # Remove 1:1 footnote text patterns
    text = re.sub(r'\+\s*\d+:\d+\s+[^.]*\.', '', text)  # Remove + 1:1 footnote text.
    text = re.sub(r'\+[^\\]*', '', text)  # Remove any remaining + markers
    text = re.sub(r'\\+', '', text)  # Remove any escaped backslashes
    # Clean up any remaining footnote artifacts
    text = re.sub(r'\s+The\s+[A-Z][^.]*?rendered[^.]*?is[^.]*?"[^"]*"[^.]*?\.', '', text)  # Remove "The ... rendered ... is ..."
    
    # Extract text from word markers
    # Pattern: \w text|strong="..."\w* or \w text\w*
    # We want to keep the text, remove the markers
    def replace_word(match):
        # Get everything between \w and \w*
        word_group = match.group(0)
        # Extract text before | or before \w*
        if '|' in word_group:
            # Text is before the pipe
            text_part = word_group.split('|')[0].replace('\\w', '').strip()
        else:
            # Text is between \w and \w*
            text_part = re.sub(r'\\w\s*', '', word_group)
            text_part = re.sub(r'\\w\*', '', text_part).strip()
        return text_part + ' '  # Add space to preserve word boundaries
    
    # Replace word markers with their text content
    text = re.sub(r'\\w\s+[^\\]*?\\w\*', replace_word, text)
    
    # Remove strong's number attributes
    text = re.sub(r'\\strong="[^"]*"', '', text)
    
    # Remove special character markers
    text = re.sub(r'\+wh[^+]*\+wh\*', '', text)
    text = re.sub(r'\+[^+]*\+', '', text)
    
    # Remove Hebrew characters (transliteration markers)
    text = re.sub(r'[א-ת֑-״]+', '', text)
    
    # Remove any remaining backslash control sequences
    text = re.sub(r'\\[A-Za-z0-9*]+\s*', '', text)
    
    # Remove standalone USFM marker remnants (b, m, q1, q2, s1, s2, li1, pmo, etc.)
    # These are formatting markers that may appear as standalone words
    text = re.sub(r'\b[bmsq]\s+[bmsq]\b', '', text)  # Remove "b m", "s q" patterns
    text = re.sub(r'\b[bmsq]\d*\b', '', text)  # Remove standalone markers like "b", "m", "q1", "q2", "s1", "s2"
    text = re.sub(r'\bli\d+\b', '', text)  # Remove list markers like "li1", "li2"
    text = re.sub(r'\bpmo\b', '', text)  # Remove paragraph marker "pmo"
    text = re.sub(r'\bp\b', '', text)  # Remove paragraph marker "p" (but be careful not to remove words)
    # Remove section headings that appear mid-verse (like "The First Day", "The Second Day")
    # These typically appear after verse text and before the next verse
    text = re.sub(r'\s+The\s+(First|Second|Third|Fourth|Fifth|Sixth|Seventh|Eighth|Ninth|Tenth)\s+Day\s*', '', text, flags=re.IGNORECASE)
    
    # Clean up whitespace and punctuation spacing
    text = re.sub(r'\s+', ' ', text)
    text = re.sub(r'\s+([,.;:!?])', r'\1', text)  # Remove space before punctuation
    text = re.sub(r'\s+', ' ', text)  # Clean up again
    text = text.strip()
    
    # Remove quotes if they wrap the entire text
    if text.startswith('"') and text.endswith('"') and text.count('"') >= 2:
        text = text[1:-1].strip()
    
    return text

def preprocess_usfm_line(line):
    """Remove cross-references and footnotes before parsing"""
    # Remove cross-references \x ... \x*
    line = re.sub(r'\\x\s+[^\\]*?\\x\*', '', line)
    # Remove footnotes \f ... \f* (handle both simple and complex formats)
    line = re.sub(r'\\f\s+[^\\]*?\\fr\s+[^\\]*?\\ft\s+[^\\]*?\\f\*', '', line)
    line = re.sub(r'\\f[^\\]*?\\f\*', '', line)
    return line

def parse_usfm_file(filepath):
    with open(filepath, 'r', encoding='utf-8') as f:
        lines = f.readlines()
    
    book_code = None
    filename_upper = Path(filepath).name.upper()
    # Match book codes in order of specificity
    # Sort by: 1) length descending, 2) non-digit-prefixed first, 3) alphabetically
    # This ensures "COL" matches before "2CO" can match "52COLBSB"
    import re
    def sort_key(code):
        # Return tuple: (-length, is_digit_prefixed, code)
        # is_digit_prefixed: 0 for codes starting with digit, 1 for others
        is_digit = 1 if code[0].isdigit() else 0
        return (-len(code), is_digit, code)
    
    sorted_codes = sorted(book_names.keys(), key=sort_key)
    for code in sorted_codes:
        # Simple substring match - since we check longer codes and non-digit codes first,
        # "COL" will match "52COLBSB" before "2CO" can match
        if code in filename_upper:
            book_code = code
            break
    
    if not book_code:
        return None, []
    
    book_name = book_names[book_code]
    chapters = []
    
    current_chapter = None
    current_verses = {}  # Use dict to handle duplicates: verse_num -> text
    current_verse_num = None
    current_verse_text = []
    
    # Pattern to match verse markers ONLY at start of line: ^\v <num>
    verse_pattern = re.compile(r'^\\v\s+(\d+)\s+(.*)$')
    
    for line in lines:
        original_line = line.rstrip()
        
        # Preprocess: remove cross-refs and footnotes
        line = preprocess_usfm_line(original_line)
        line = line.rstrip()
        
        # Skip metadata lines
        if line.startswith('\\id') or line.startswith('\\ide') or line.startswith('\\h') or \
           line.startswith('\\toc') or line.startswith('\\mt'):
            continue
        
        # Chapter marker: \c 1 (only at start of line)
        if line.startswith('\\c'):
            match = re.match(r'\\c\s+(\d+)', line)
            if match:
                # Save previous chapter if exists
                if current_chapter is not None and current_verses:
                    # Convert dict to list for output
                    verse_list = [(num, text) for num, text in sorted(current_verses.items())]
                    chapters.append((current_chapter, verse_list))
                
                current_chapter = int(match.group(1))
                current_verses = {}
                current_verse_num = None
                current_verse_text = []
            continue
        
        # Verse marker: ONLY lines starting with \v <num>
        verse_match = verse_pattern.match(line)
        if verse_match:
            # Save previous verse if exists
            if current_verse_num is not None and current_verse_text:
                verse_text = ' '.join(current_verse_text).strip()
                verse_text = clean_usfm_text(verse_text)
                if verse_text:
                    # Merge if duplicate verse number exists
                    if current_verse_num in current_verses:
                        current_verses[current_verse_num] += " " + verse_text
                    else:
                        current_verses[current_verse_num] = verse_text
            
            current_verse_num = int(verse_match.group(1))
            current_verse_text = [verse_match.group(2)] if verse_match.group(2).strip() else []
            continue
        
        # Continuation of verse text (only if we're in a verse)
        if current_verse_num is not None:
            # Skip lines that start with \v (these are embedded, not continuations)
            if not line.startswith('\\v'):
                current_verse_text.append(line)
    
    # Save last verse and chapter
    if current_verse_num is not None and current_verse_text:
        verse_text = ' '.join(current_verse_text).strip()
        verse_text = clean_usfm_text(verse_text)
        if verse_text:
            if current_verse_num in current_verses:
                current_verses[current_verse_num] += " " + verse_text
            else:
                current_verses[current_verse_num] = verse_text
    
    if current_chapter is not None and current_verses:
        verse_list = [(num, text) for num, text in sorted(current_verses.items())]
        chapters.append((current_chapter, verse_list))
    
    return book_name, chapters

def convert_usfm_directory(input_dir, output_file):
    usfm_files = sorted(list(Path(input_dir).glob("*.usfm")) + 
                       list(Path(input_dir).glob("*.SFM")) + 
                       list(Path(input_dir).glob("*.sfm")))
    
    # Filter out front matter
    usfm_files = [f for f in usfm_files if not f.name.startswith('00-')]
    
    if not usfm_files:
        print(f"No USFM files found in {input_dir}")
        return
    
    output_lines = []
    processed_books = {}
    duplicate_warnings = []
    
    # Sort files by book order
    def get_book_code(filename):
        for code in book_order:
            if code in filename.upper():
                return book_order.index(code)
        return 999
    
    usfm_files.sort(key=lambda f: get_book_code(f.name))
    
    for usfm_file in usfm_files:
        book_name, chapters = parse_usfm_file(usfm_file)
        if not book_name or not chapters:
            continue
        
        if book_name in processed_books:
            continue
        
        processed_books[book_name] = True
        output_lines.append(book_name)
        
        for chapter_num, verses in chapters:
            # Sanity check: count unique verse numbers
            verse_nums = [v[0] for v in verses]
            unique_verses = len(set(verse_nums))
            total_verses = len(verse_nums)
            
            if unique_verses != total_verses:
                # Find duplicates
                from collections import Counter
                verse_counts = Counter(verse_nums)
                duplicates = {v: c for v, c in verse_counts.items() if c > 1}
                for verse_num, count in duplicates.items():
                    duplicate_warnings.append(f"{book_name} Chapter {chapter_num}, Verse {verse_num}: {count} occurrences (merged)")
            
            output_lines.append(f"Chapter {chapter_num}")
            for verse_num, verse_text in sorted(verses, key=lambda x: x[0]):
                output_lines.append(f"{verse_num} {verse_text}")
            output_lines.append("")
    
    if output_lines and output_lines[-1] == "":
        output_lines.pop()
    
    with open(output_file, 'w', encoding='utf-8') as f:
        f.write('\n'.join(output_lines))
        if output_lines:
            f.write('\n')
    
    print(f"Converted {len(processed_books)} books to {output_file} ({len(output_lines)} lines)")
    if duplicate_warnings:
        print(f"\nDuplicate verse warnings (merged): {len(duplicate_warnings)}")
        for warning in duplicate_warnings[:10]:  # Show first 10
            print(f"  {warning}")
        if len(duplicate_warnings) > 10:
            print(f"  ... and {len(duplicate_warnings) - 10} more")

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: convert_usfm_to_kjv_format.py <input_directory> <output_file>")
        sys.exit(1)
    
    convert_usfm_directory(sys.argv[1], sys.argv[2])
