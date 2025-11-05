# Creating Social Media Images

For proper link previews on X (Twitter), Facebook, and other platforms, you need:

1. **Open Graph Image** (`og-image.png`)
   - Size: 1200x630 pixels
   - Format: PNG or JPG
   - Location: `out/static/og-image.png`

2. **Favicon** (`favicon.ico`)
   - Size: 16x16, 32x32, or 48x48 pixels
   - Format: ICO
   - Location: `out/static/favicon.ico`

## Quick Creation Options

### Option 1: Use an Online Tool
- **OG Image**: Use https://www.canva.com/ or similar to create a 1200x630 image
- **Favicon**: Use https://favicon.io/ to generate from text or image

### Option 2: Create Simple Placeholder
You can create a simple text-based image with:
- Text: "Bible" or "Holy Bible"
- Background: Solid color or gradient
- Size: 1200x630 for OG image

### Option 3: Use a Bible-themed Image
- Use a free stock photo site (Unsplash, Pexels)
- Search for "bible" or "book" themes
- Resize to 1200x630

## Image Requirements

- **OG Image**: Should represent your site well - could be a Bible, open book, or text saying "Bible"
- **Favicon**: Simple icon - could be a cross, book icon, or "B" letter
- **Both**: Must be accessible at `/static/og-image.png` and `/static/favicon.ico`

Once created, place them in `out/static/` directory and they'll be included in the build.

