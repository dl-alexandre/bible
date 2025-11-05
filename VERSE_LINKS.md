# Linking Individual Verses

Your Bible site supports multiple ways to link to individual verses:

## 1. Direct Anchor Links (On-Page)

Each verse has an anchor ID in the format `#v{verse_number}`. You can link directly to verses within a chapter page:

**Format:** `/bible/{version}/{book}/{chapter}.html#v{verse}`

**Examples:**
- Genesis 1:1: `/bible/kjv/Genesis/1.html#v1`
- Genesis 1:31: `/bible/kjv/Genesis/1.html#v31`

These links work by scrolling to the verse on the chapter page. The verse numbers are now clickable links that point to their own anchors.

## 2. Short Redirect URLs

For shareable links, use the redirect format:

**Format:** `/bible/{version}/{book}/{book}.{chapter}.{verse}.html`

**Examples:**
- Genesis 1:1: `/bible/kjv/Genesis/Genesis.1.1.html`
- Genesis 1:31: `/bible/kjv/Genesis/Genesis.1.31.html`

These redirect pages automatically jump to the correct verse on the chapter page. They're ideal for:
- Sharing on social media
- Embedding in apps
- Creating short, memorable URLs

## 3. Canonical References

Each verse has a `data-verse` attribute with the canonical reference:

**Format:** `{book}.{chapter}.{verse}`

**Examples:**
- `Genesis.1.1`
- `Genesis.1.31`

These are used for cross-version mapping and can be accessed via JavaScript:

```javascript
const verse = document.querySelector('[data-verse="Genesis.1.1"]');
```

## 4. JSON API Access

Individual verses can be accessed via the JSON API:

**Format:** `/bible/{version}/{book}/{chapter}.json`

**Example:** `/bible/kjv/Genesis/1.json`

Returns all verses in the chapter as a JSON object with verse numbers as keys:

```json
{
  "schema_version": "1.0",
  "book": "Genesis",
  "chapter": 1,
  "version": "kjv",
  "verses": {
    "1": "In the beginning God created the heaven and the earth.",
    "2": "And the earth was without form, and void..."
  }
}
```

## Usage Examples

### HTML Links

```html
<a href="/bible/kjv/Genesis/1.html#v1">Genesis 1:1</a>
<a href="/bible/kjv/Genesis/Genesis.1.1.html">Genesis 1:1 (short URL)</a>
```

### JavaScript Navigation

```javascript
window.location.hash = '#v1';
window.location.href = '/bible/kjv/Genesis/Genesis.1.1.html';
```

### Cross-Version Linking

Use canonical references to link between versions:

```javascript
const canonical = 'Genesis.1.1';
const version = 'kjv';
const chapter = canonical.split('.')[1];
const verse = canonical.split('.')[2];
const url = `/bible/${version}/Genesis/${chapter}.html#v${verse}`;
```

## Current Status

✅ Verse anchors are generated (`id="v{number}"`)
✅ Verse numbers are clickable links
✅ Redirect pages are generated during build
✅ Canonical references are stored in `data-verse` attributes
✅ JSON API provides verse-level access

## Testing Your Links

1. **Anchor links**: Open `/bible/kjv/Genesis/1.html#v5` - should scroll to verse 5
2. **Redirect links**: Open `/bible/kjv/Genesis/Genesis.1.5.html` - should redirect to the chapter page with verse 5 highlighted
3. **Click verse numbers**: Click any verse number on a chapter page - should jump to that verse

