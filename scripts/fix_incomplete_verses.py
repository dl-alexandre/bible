#!/usr/bin/env python3
import re
import sys
from pathlib import Path

def get_complete_verse_text(book, chapter, verse):
    verse_fixes = {
        ("Ephesians", 4, 21): "If so be that ye have heard him, and have been taught by him, as the truth is in Jesus:",
        ("Nehemiah", 12, 4): "Iddo, Ginnetho, Abijah,",
        ("Ephesians", 4, 23): "And be renewed in the spirit of your mind;",
        ("Ephesians", 4, 2): "With all lowliness and meekness, with longsuffering, forbearing one another in love;",
        ("Ephesians", 1, 8): "Wherein he hath abounded toward us in all wisdom and prudence;",
        ("Ephesians", 3, 11): "According to the eternal purpose which he purposed in Christ Jesus our Lord:",
        ("Ephesians", 6, 15): "And your feet shod with the preparation of the gospel of peace;",
        ("Ephesians", 6, 7): "With good will doing service, as to the Lord, and not to men:",
        ("Nehemiah", 12, 4): "Iddo, Ginnetho, Abijah,",
        ("Nehemiah", 10, 16): "Adonijah, Bigvai, Adin,",
        ("Nehemiah", 10, 17): "Ater, Hizkijah, Azzur,",
        ("Nehemiah", 10, 11): "Micha, Rehob, Hashabiah,",
        ("Nehemiah", 10, 12): "Zaccur, Sherebiah, Shebaniah,",
        ("Nehemiah", 10, 39): "And we will not forsake the house of our God.",
        ("Nehemiah", 10, 40): "Machnadebai, Shashai, Sharai,",
        ("Acts", 16, 30): "And brought them out, and said, Sirs, what must I do to be saved?",
        ("Acts", 2, 8): "And how hear we every man in our own tongue, wherein we were born?",
        ("Acts", 23, 4): "And they that stood by said, Revilest thou God's high priest?",
        ("Colossians", 3, 4): "When Christ, who is our life, shall appear, then shall ye also appear with him in glory.",
        ("Colossians", 6, 2): "Timothy my workfellow, and Lucius, and Jason, and Sosipater, my kinsmen, salute you.",
        ("Deuteronomy", 1, 5): "On this side Jordan, in the land of Moab, began Moses to declare this law, saying,",
        ("Ecclesiastes", 7, 17): "Be not over much wicked, neither be thou foolish: why shouldest thou die before thy time?",
        ("Esther", 9, 8): "And Poratha, and Adalia, and Aridatha,",
        ("Exodus", 1, 3): "Issachar, Zebulun, and Benjamin,",
        ("Exodus", 35, 18): "The pins of the tabernacle, and the pins of the court, and their cords,",
        ("Ezekiel", 16, 2): "Son of man, cause Jerusalem to know her abominations,",
        ("Ezekiel", 17, 2): "Son of man, put forth a riddle, and speak a parable unto the house of Israel;",
        ("Ezekiel", 20, 19): "I am the LORD your God; walk in my statutes, and keep my judgments, and do them;",
        ("Ezekiel", 23, 2): "Son of man, there were two women, the daughters of one mother:",
        ("Ezekiel", 24, 20): "Then I answered them, The word of the LORD came unto me, saying,",
        ("Ezekiel", 25, 2): "Son of man, set thy face against the Ammonites, and prophesy against them;",
        ("Ezekiel", 27, 2): "Now, thou son of man, take up a lamentation for Tyrus;",
        ("Ezekiel", 28, 21): "Son of man, set thy face against Zidon, and prophesy against it,",
        ("Ezekiel", 34, 9): "Therefore, O ye shepherds, hear the word of the LORD;",
        ("Ezekiel", 35, 2): "Son of man, set thy face against mount Seir, and prophesy against it,",
        ("Ezra", 10, 35): "Benaiah, Bedeiah, Chelluh,",
        ("Ezra", 10, 36): "Vaniah, Meremoth, Eliashib,",
        ("Ezra", 10, 37): "Mattaniah, Mattenai, and Jaasau,",
        ("Ezra", 10, 38): "And Bani, and Binnui, Shimei,",
        ("Ezra", 2, 45): "The children of Lebanah, the children of Hagabah, the children of Akkub,",
        ("Ezra", 2, 46): "The children of Hagab, the children of Shalmai, the children of Hanan,",
        ("Ezra", 2, 51): "The children of Bakbuk, the children of Hakupha, the children of Harhur,",
        ("Ezra", 2, 52): "The children of Bazluth, the children of Mehida, the children of Harsha,",
        ("Ezra", 2, 56): "The children of Jaalah, the children of Darkon, the children of Giddel,",
        ("Ezra", 7, 3): "The son of Amariah, the son of Azariah, the son of Meraioth,",
        ("Ezra", 7, 4): "The son of Zerahiah, the son of Uzzi, the son of Bukki,",
        ("Galatians", 1, 2): "And all the brethren which are with me, unto the churches of Galatia:",
        ("Galatians", 2, 15): "We who are Jews by nature, and not sinners of the Gentiles,",
        ("Genesis", 10, 16): "And the Jebusite, and the Amorite, and the Girgasite,",
        ("Genesis", 10, 17): "And the Hivite, and the Arkite, and the Sinite,",
        ("Genesis", 10, 28): "And Obal, and Abimael, and Sheba,",
        ("Genesis", 15, 19): "The Kenites, and the Kenizzites, and the Kadmonites,",
        ("Genesis", 15, 20): "And the Hittites, and the Perizzites, and the Rephaims,",
        ("Genesis", 17, 18): "And Abraham said unto God, O that Ishmael might live before thee!",
        ("Genesis", 26, 6): "And Isaac dwelt in Gerar:",
        ("Genesis", 28, 8): "And Esau seeing that the daughters of Canaan pleased not Isaac his father;",
    }
    
    return verse_fixes.get((book, chapter, verse))

def fix_incomplete_verses(input_file, output_file):
    with open(input_file, 'r', encoding='utf-8') as f:
        lines = f.readlines()
    
    current_book = None
    current_chapter = None
    fixes_applied = []
    
    output_lines = []
    i = 0
    
    while i < len(lines):
        line = lines[i]
        stripped = line.strip()
        
        if not stripped:
            output_lines.append(line)
            i += 1
            continue
        
        if stripped and not stripped.startswith('Chapter') and not any(c.isdigit() for c in stripped[:3]):
            book_names = [
                "Genesis", "Exodus", "Leviticus", "Numbers", "Deuteronomy",
                "Joshua", "Judges", "Ruth", "1 Samuel", "2 Samuel", "1 Kings", "2 Kings",
                "1 Chronicles", "2 Chronicles", "Ezra", "Nehemiah", "Esther", "Job",
                "Psalm", "Psalms", "Proverbs", "Ecclesiastes", "Song of Solomon", "Song of Songs",
                "Isaiah", "Jeremiah", "Lamentations", "Ezekiel", "Daniel", "Hosea", "Joel",
                "Amos", "Obadiah", "Jonah", "Micah", "Nahum", "Habakkuk", "Zephaniah",
                "Haggai", "Zechariah", "Malachi",
                "Matthew", "Mark", "Luke", "John", "Acts", "Romans", "1 Corinthians",
                "2 Corinthians", "Galatians", "Ephesians", "Philippians", "Colossians",
                "1 Thessalonians", "2 Thessalonians", "1 Timothy", "2 Timothy", "Titus",
                "Philemon", "Hebrews", "James", "1 Peter", "2 Peter", "1 John", "2 John",
                "3 John", "Jude", "Revelation",
            ]
            if stripped in book_names:
                current_book = stripped
                current_chapter = None
                output_lines.append(line)
                i += 1
                continue
        
        if stripped.startswith('Chapter '):
            try:
                current_chapter = int(stripped.split()[1])
            except:
                pass
            output_lines.append(line)
            i += 1
            continue
        
        if stripped[0].isdigit():
            parts = stripped.split(None, 1)
            if len(parts) >= 1:
                try:
                    verse_num = int(parts[0])
                    verse_text = parts[1] if len(parts) > 1 else ""
                    
                    complete_text = get_complete_verse_text(current_book, current_chapter, verse_num)
                    
                    if complete_text and (len(verse_text) < 30 or verse_text.endswith((';', ',', ':')) and not verse_text.endswith('.')):
                        fixed_line = f"{verse_num} {complete_text}\n"
                        output_lines.append(fixed_line)
                        fixes_applied.append(f"{current_book} {current_chapter}:{verse_num}")
                        i += 1
                        continue
                except ValueError:
                    pass
        
        output_lines.append(line)
        i += 1
    
    with open(output_file, 'w', encoding='utf-8') as f:
        f.writelines(output_lines)
    
    return fixes_applied

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python3 fix_incomplete_verses.py <input_file> [output_file]")
        sys.exit(1)
    
    input_file = Path(sys.argv[1])
    output_file = Path(sys.argv[2]) if len(sys.argv) > 2 else input_file
    
    if not input_file.exists():
        print(f"Error: Input file not found: {input_file}")
        sys.exit(1)
    
    print(f"Fixing incomplete verses in {input_file}...")
    fixes = fix_incomplete_verses(input_file, output_file)
    
    if fixes:
        print(f"\n✓ Fixed {len(fixes)} verses:")
        for fix in fixes:
            print(f"  - {fix}")
    else:
        print("\n✓ No fixes needed")
    
    print(f"\nOutput written to: {output_file}")

