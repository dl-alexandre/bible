#!/usr/bin/env python3

from PIL import Image, ImageDraw, ImageFont
from pathlib import Path

def create_favicon():
    sizes = [16, 32, 48]
    images = []
    
    for size in sizes:
        img = Image.new('RGBA', (size, size), '#1a1a1a')
        draw = ImageDraw.Draw(img)
        
        try:
            font_size = int(size * 0.65)
            font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", font_size)
        except:
            try:
                font = ImageFont.truetype("/System/Library/Fonts/Arial.ttf", font_size)
            except:
                font = ImageFont.load_default()
        
        bbox = draw.textbbox((0, 0), 'B', font=font)
        text_width = bbox[2] - bbox[0]
        text_height = bbox[3] - bbox[1]
        
        x = (size - text_width) / 2 - bbox[0]
        y = (size - text_height) / 2 - bbox[1]
        
        draw.text((x, y), 'B', font=font, fill='#FFFFFF')
        images.append(img)
    
    output_path = Path("out/static/favicon.ico")
    output_path.parent.mkdir(parents=True, exist_ok=True)
    
    images[0].save(str(output_path), format='ICO', sizes=[(img.width, img.height) for img in images])
    print(f"Favicon created at {output_path} with sizes: {', '.join(map(str, sizes))}")

if __name__ == "__main__":
    create_favicon()

