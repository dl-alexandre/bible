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
    
    # Clean up whitespace and punctuation spacing
    text = re.sub(r'\s+', ' ', text)
    text = re.sub(r'\s+([,.;:!?])', r'\1', text)  # Remove space before punctuation
    text = re.sub(r'\s+', ' ', text)  # Clean up again
    text = text.strip()
    
    # Remove quotes if they wrap the entire text
    if text.startswith('"') and text.endswith('"') and text.count('"') >= 2:
        text = text[1:-1].strip()
    
    return text

def parse_usfm_file(filepath):
    with open(filepath, 'r', encoding='utf-8') as f:
        lines = f.readlines()
    
    book_code = None
    for code, name in book_names.items():
        if code in Path(filepath).name.upper():
            book_code = code
            break
    
    if not book_code:
        return None, []
    
    book_name = book_names[book_code]
    chapters = []
    
    current_chapter = None
    current_verses = []
    current_verse_num = None
    current_verse_text = []
    
    # Pattern to match verse markers: \v 1 text...
    verse_pattern = re.compile(r'^\\v\s+(\d+)\s+(.*)$')
    
    for line in lines:
        line = line.rstrip()
        
        # Skip metadata lines
        if line.startswith('\\id') or line.startswith('\\ide') or line.startswith('\\h') or \
           line.startswith('\\toc') or line.startswith('\\mt'):
            continue
        
        # Chapter marker: \c 1
        if line.startswith('\\c'):
            match = re.match(r'\\c\s+(\d+)', line)
            if match:
                # Save previous chapter if exists
                if current_chapter is not None and current_verses:
                    chapters.append((current_chapter, current_verses))
                
                current_chapter = int(match.group(1))
                current_verses = []
                current_verse_num = None
                current_verse_text = []
            continue
        
        # Verse marker: \v 1 text...
        verse_match = verse_pattern.match(line)
        if verse_match:
            # Save previous verse if exists
            if current_verse_num is not None:
                verse_text = ' '.join(current_verse_text).strip()
                verse_text = clean_usfm_text(verse_text)
                if verse_text:
                    current_verses.append((current_verse_num, verse_text))
            
            current_verse_num = int(verse_match.group(1))
            current_verse_text = [verse_match.group(2)]
            continue
        
        # Continuation of verse text
        if current_verse_num is not None:
            current_verse_text.append(line)
    
    # Save last verse and chapter
    if current_verse_num is not None:
        verse_text = ' '.join(current_verse_text).strip()
        verse_text = clean_usfm_text(verse_text)
        if verse_text:
            current_verses.append((current_verse_num, verse_text))
    
    if current_chapter is not None and current_verses:
        chapters.append((current_chapter, current_verses))
    
    return book_name, chapters

def convert_usfm_directory(input_dir, output_file):
    usfm_files = sorted(Path(input_dir).glob("*.usfm"))
    
    # Filter out front matter
    usfm_files = [f for f in usfm_files if not f.name.startswith('00-')]
    
    if not usfm_files:
        print(f"No USFM files found in {input_dir}")
        return
    
    output_lines = []
    processed_books = {}
    
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

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: convert_usfm_to_kjv_format.py <input_directory> <output_file>")
        sys.exit(1)
    
    convert_usfm_directory(sys.argv[1], sys.argv[2])
