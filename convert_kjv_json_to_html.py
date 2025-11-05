#!/usr/bin/env python3

import json
from pathlib import Path
from typing import Dict, Tuple
from jinja2 import Environment, FileSystemLoader, select_autoescape

def get_template_env():
    template_dir = Path("templates")
    env = Environment(
        loader=FileSystemLoader(str(template_dir)),
        autoescape=select_autoescape(['html', 'xml'])
    )
    return env

def parse_json_file(json_path: Path) -> Dict:
    with open(json_path, 'r', encoding="utf-8") as f:
        return json.load(f)

def get_adjacent_chapters(book_dir: Path, chapter_num: int) -> Tuple[int | None, int | None]:
    prev_chapter = None
    next_chapter = None
    
    if chapter_num > 1:
        prev_path = book_dir / f"{chapter_num - 1}.json"
        if prev_path.exists():
            prev_chapter = chapter_num - 1
    
    next_path = book_dir / f"{chapter_num + 1}.json"
    if next_path.exists():
        next_chapter = chapter_num + 1
    
    return prev_chapter, next_chapter

def generate_html_from_json(json_data: Dict, template, prev_chapter: int | None, next_chapter: int | None) -> str:
    book = json_data["book"]
    chapter = json_data["chapter"]
    version_code = json_data["version"]
    version_name = version_code.upper()
    last_updated = json_data.get("metadata", {}).get("last_updated", "Unknown")
    verses_data = json_data["verses"]
    
    verses = []
    for verse_num in sorted(verses_data.keys(), key=lambda x: int(x) if x.isdigit() else 0):
        verse_text = verses_data[verse_num]
        canonical_ref = f"{book}.{chapter}.{verse_num}"
        anchor = f"#v{verse_num}"
        
        verses.append({
            "number": verse_num,
            "text": verse_text,
            "canonical_ref": canonical_ref,
            "anchor": anchor
        })
    
    context = {
        "book": book,
        "chapter": chapter,
        "version_code": version_code,
        "version_name": version_name,
        "last_updated": last_updated,
        "verses": verses,
        "prev_chapter": prev_chapter,
        "next_chapter": next_chapter
    }
    
    return template.render(**context)

def convert_all_kjv_json_to_html():
    json_base = Path("out/kjv")
    html_base = Path("out/bible/kjv")
    
    if not json_base.exists():
        print(f"JSON directory not found: {json_base}")
        return
    
    env = get_template_env()
    template = env.get_template("chapter.html")
    
    converted = 0
    for book_dir in sorted(json_base.iterdir()):
        if not book_dir.is_dir():
            continue
        
        book_name = book_dir.name
        html_book_dir = html_base / book_name
        html_book_dir.mkdir(parents=True, exist_ok=True)
        
        json_files = sorted(book_dir.glob("*.json"), key=lambda x: int(x.stem) if x.stem.isdigit() else 0)
        
        for json_file in json_files:
            chapter_num = int(json_file.stem)
            
            try:
                json_data = parse_json_file(json_file)
                prev_chapter, next_chapter = get_adjacent_chapters(book_dir, chapter_num)
                
                html_content = generate_html_from_json(json_data, template, prev_chapter, next_chapter)
                
                html_path = html_book_dir / f"{chapter_num}.html"
                html_path.write_text(html_content, encoding="utf-8")
                
                converted += 1
                if converted % 100 == 0:
                    print(f"Converted {converted} files...")
                    
            except Exception as e:
                print(f"Error converting {json_file}: {e}")
                continue
    
    print(f"Conversion complete! Converted {converted} JSON files to HTML.")

if __name__ == "__main__":
    convert_all_kjv_json_to_html()
