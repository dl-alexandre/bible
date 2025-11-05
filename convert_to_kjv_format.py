#!/usr/bin/env python3
"""
Convert Project Gutenberg Bible format to kjv.txt format
Input: Project Gutenberg format (chapter:verse text)
Output: kjv.txt format (book name, Chapter X, numbered verses)
"""

import re
import sys

def convert_gutenberg_to_kjv_format(input_file, output_file, version_name="Bible"):
    with open(input_file, 'r', encoding='utf-8') as f:
        lines = f.readlines()
    
    output_lines = []
    current_book = None
    verses = []
    current_text = []
    current_chapter = None
    current_verse = None
    genesis_count = 0
    content_started = False
    
    book_mapping = {
        "The First Book of Moses: Called Genesis": "Genesis",
        "The Second Book of Moses: Called Exodus": "Exodus",
        "The Third Book of Moses: Called Leviticus": "Leviticus",
        "The Fourth Book of Moses: Called Numbers": "Numbers",
        "The Fifth Book of Moses: Called Deuteronomy": "Deuteronomy",
        "The Book of Joshua": "Joshua",
        "The Book of Judges": "Judges",
        "The Book of Ruth": "Ruth",
        "The First Book of Samuel": "1 Samuel",
        "The Second Book of Samuel": "2 Samuel",
        "The First Book of the Kings": "1 Kings",
        "The Second Book of the Kings": "2 Kings",
        "The First Book of the Chronicles": "1 Chronicles",
        "The Second Book of the Chronicles": "2 Chronicles",
        "Ezra": "Ezra",
        "The Book of Nehemiah": "Nehemiah",
        "The Book of Esther": "Esther",
        "The Book of Job": "Job",
        "The Book of Psalms": "Psalms",
        "The Proverbs": "Proverbs",
        "Ecclesiastes": "Ecclesiastes",
        "The Song of Solomon": "Song of Solomon",
        "The Book of the Prophet Isaiah": "Isaiah",
        "The Book of the Prophet Jeremiah": "Jeremiah",
        "The Lamentations of Jeremiah": "Lamentations",
        "The Book of the Prophet Ezekiel": "Ezekiel",
        "The Book of Daniel": "Daniel",
        "Hosea": "Hosea",
        "Joel": "Joel",
        "Amos": "Amos",
        "Obadiah": "Obadiah",
        "Jonah": "Jonah",
        "Micah": "Micah",
        "Nahum": "Nahum",
        "Habakkuk": "Habakkuk",
        "Zephaniah": "Zephaniah",
        "Haggai": "Haggai",
        "Zechariah": "Zechariah",
        "Malachi": "Malachi",
        "The Gospel According to Saint Matthew": "Matthew",
        "The Gospel According to Saint Mark": "Mark",
        "The Gospel According to Saint Luke": "Luke",
        "The Gospel According to Saint John": "John",
        "The Acts of the Apostles": "Acts",
        "The Epistle of Paul the Apostle to the Romans": "Romans",
        "The First Epistle of Paul the Apostle to the Corinthians": "1 Corinthians",
        "The Second Epistle of Paul the Apostle to the Corinthians": "2 Corinthians",
        "The Epistle of Paul the Apostle to the Galatians": "Galatians",
        "The Epistle of Paul the Apostle to the Ephesians": "Ephesians",
        "The Epistle of Paul the Apostle to the Philippians": "Philippians",
        "The Epistle of Paul the Apostle to the Colossians": "Colossians",
        "The First Epistle of Paul the Apostle to the Thessalonians": "1 Thessalonians",
        "The Second Epistle of Paul the Apostle to the Thessalonians": "2 Thessalonians",
        "The First Epistle of Paul the Apostle to Timothy": "1 Timothy",
        "The Second Epistle of Paul the Apostle to Timothy": "2 Timothy",
        "The Epistle of Paul the Apostle to Titus": "Titus",
        "The Epistle of Paul the Apostle to Philemon": "Philemon",
        "The Epistle of Paul the Apostle to the Hebrews": "Hebrews",
        "The General Epistle of James": "James",
        "The First Epistle General of Peter": "1 Peter",
        "The Second General Epistle of Peter": "2 Peter",
        "The First Epistle General of John": "1 John",
        "The Second Epistle General of John": "2 John",
        "The Third Epistle General of John": "3 John",
        "The General Epistle of Jude": "Jude",
        "The Revelation of Saint John the Divine": "Revelation",
    }
    
    for line in lines:
        line_stripped = line.strip()
        
        if "*** START" in line_stripped or "*** END" in line_stripped:
            continue
        
        if "The First Book of Moses: Called Genesis" in line_stripped:
            genesis_count += 1
            if genesis_count == 2:
                content_started = True
            continue
        
        if not content_started:
            continue
        
        if line_stripped in book_mapping:
            if current_verse is not None:
                verse_text = ' '.join(current_text).strip()
                if verse_text:
                    verses.append({
                        'chapter': current_chapter,
                        'verse': current_verse,
                        'text': verse_text
                    })
            
            if verses:
                write_verses(verses, output_lines)
                verses = []
            
            if current_book is not None:
                output_lines.append("")
            
            current_book = book_mapping[line_stripped]
            output_lines.append(current_book)
            current_text = []
            current_chapter = None
            current_verse = None
            continue
        
        if not line_stripped:
            if current_text:
                current_text.append("")
            continue
        
        match = re.search(r'(\d+):(\d+)', line_stripped)
        if match:
            chapter_num = int(match.group(1))
            verse_num = int(match.group(2))
            
            if current_verse is not None:
                verse_text = ' '.join(current_text).strip()
                if verse_text:
                    verses.append({
                        'chapter': current_chapter,
                        'verse': current_verse,
                        'text': verse_text
                    })
                current_text = []
            
            text_start = match.end()
            verse_text = line_stripped[text_start:].strip()
            current_chapter = chapter_num
            current_verse = verse_num
            current_text = [verse_text] if verse_text else []
        elif line_stripped and current_verse is not None:
            current_text.append(line_stripped)
    
    if current_verse is not None:
        verse_text = ' '.join(current_text).strip()
        if verse_text:
            verses.append({
                'chapter': current_chapter,
                'verse': current_verse,
                'text': verse_text
            })
    
    if verses:
        write_verses(verses, output_lines)
    
    with open(output_file, 'w', encoding='utf-8') as f:
        f.write('\n'.join(output_lines))
        if output_lines:
            f.write('\n')
    
    print(f"Converted {len(output_lines)} lines to {output_file}")

def write_verses(verses, output_lines):
    if not verses:
        return
    
    current_chapter = None
    for verse in verses:
        if verse['chapter'] != current_chapter:
            if current_chapter is not None:
                output_lines.append("")
            output_lines.append(f"Chapter {verse['chapter']}")
            current_chapter = verse['chapter']
        
        verse_text = ' '.join(verse['text'].split())
        output_lines.append(f"{verse['verse']} {verse_text}")

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: convert_to_kjv_format.py <input_file> <output_file>")
        sys.exit(1)
    
    convert_gutenberg_to_kjv_format(sys.argv[1], sys.argv[2])

