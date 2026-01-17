document.addEventListener('DOMContentLoaded', function () {
    const verseNumbers = document.querySelectorAll('.verse-number');

    // Create toast element
    const toast = document.createElement('div');
    toast.className = 'toast';
    // Add a simple checkmark icon using SVG
    toast.innerHTML = `
        <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="20 6 9 17 4 12"></polyline>
        </svg>
        <span>Verse copied to clipboard</span>
    `;
    document.body.appendChild(toast);

    let toastTimeout;

    function showToast(message) {
        const span = toast.querySelector('span');
        if (span) span.textContent = message;

        toast.classList.add('show');

        if (toastTimeout) clearTimeout(toastTimeout);

        toastTimeout = setTimeout(() => {
            toast.classList.remove('show');
        }, 3000);
    }

    function getBookAbbreviation(bookName) {
        const abbreviations = {
            "Genesis": "Gen", "Exodus": "Exod", "Leviticus": "Lev",
            "Numbers": "Num", "Deuteronomy": "Deut", "Joshua": "Josh",
            "Judges": "Judg", "Ruth": "Ruth", "1 Samuel": "1Sam",
            "2 Samuel": "2Sam", "1 Kings": "1Kgs", "2 Kings": "2Kgs",
            "1 Chronicles": "1Chr", "2 Chronicles": "2Chr", "Ezra": "Ezra",
            "Nehemiah": "Neh", "Esther": "Esth", "Job": "Job",
            "Psalm": "Ps", "Psalms": "Ps", "Proverbs": "Prov",
            "Ecclesiastes": "Eccl", "Song of Solomon": "Song", "Song of Songs": "Song",
            "Isaiah": "Isa", "Jeremiah": "Jer", "Lamentations": "Lam",
            "Ezekiel": "Ezek", "Daniel": "Dan", "Hosea": "Hos",
            "Joel": "Joel", "Amos": "Amos", "Obadiah": "Obad",
            "Jonah": "Jonah", "Micah": "Mic", "Nahum": "Nah",
            "Habakkuk": "Hab", "Zephaniah": "Zeph", "Haggai": "Hag",
            "Zechariah": "Zech", "Malachi": "Mal",
            "Matthew": "Matt", "Mark": "Mark", "Luke": "Luke",
            "John": "John", "Acts": "Acts", "Romans": "Rom",
            "1 Corinthians": "1Cor", "2 Corinthians": "2Cor", "Galatians": "Gal",
            "Ephesians": "Eph", "Philippians": "Phil", "Colossians": "Col",
            "1 Thessalonians": "1Thess", "2 Thessalonians": "2Thess",
            "1 Timothy": "1Tim", "2 Timothy": "2Tim", "Titus": "Titus",
            "Philemon": "Phlm", "Hebrews": "Heb",
            "James": "James", "1 Peter": "1Pet", "2 Peter": "2Pet",
            "1 John": "1John", "2 John": "2John", "3 John": "3John",
            "Jude": "Jude", "Revelation": "Rev"
        };
        return abbreviations[bookName] || bookName;
    }

    verseNumbers.forEach(verseNum => {
        verseNum.style.cursor = 'pointer';
        verseNum.title = 'Click to copy verse link';

        verseNum.addEventListener('click', function (e) {
            // Allow default behavior (anchor navigation) if holding modifier keys
            if (e.metaKey || e.ctrlKey || e.altKey || e.shiftKey) return;

            e.preventDefault();
            const verse = this.closest('.verse');
            const verseText = verse.querySelector('.verse-text').textContent.trim();
            const verseRef = verse.getAttribute('data-verse');
            const verseLink = window.location.href.split('#')[0] + '#' + verse.id;

            const parts = verseRef.split('.');
            const bookName = parts[0];
            const chapter = parts[1];
            const verseNum = parts[2];
            const bookAbbrev = getBookAbbreviation(bookName);
            const readableRef = `${bookAbbrev} ${chapter}:${verseNum}`;

            const textToCopy = `${verseText} - ${readableRef}\n${verseLink}`;

            navigator.clipboard.writeText(textToCopy).then(() => {
                showToast('Verse copied to clipboard');

                // Update URL without scrolling
                history.pushState(null, null, '#' + verse.id);

                // Highlight the verse
                document.querySelectorAll('.verse').forEach(v => v.classList.remove('target'));
                // We use CSS :target usually, but for JS interaction we might want a class too
                // However, our CSS uses :target. To fake it without hash change jumping:
                // The pushState updates the hash, so :target matches!
                // But sometimes browsers jump. Let's see.
                // Actually, pushState doesn't trigger hashchange event or scroll usually.
                // So :target pseudo-class might NOT update immediately in some browsers without a real hash change.
                // Let's force a class for the animation.

                // Remove any existing temporary highlights
                document.querySelectorAll('.verse-highlight').forEach(v => v.classList.remove('verse-highlight'));
                verse.classList.add('verse-highlight');

                // Remove class after animation
                setTimeout(() => {
                    verse.classList.remove('verse-highlight');
                }, 2000);

            }).catch(err => {
                console.error('Failed to copy:', err);
                showToast('Failed to copy');
            });
        });
    });

    // Handle initial hash for highlighting
    if (window.location.hash) {
        const target = document.querySelector(window.location.hash);
        if (target) {
            target.scrollIntoView({ behavior: 'smooth', block: 'center' });
            target.classList.add('verse-highlight');
        }
    }

    // Version Switcher
    const versionBtn = document.querySelector('.version-switcher-btn');
    const versionDropdown = document.querySelector('.version-dropdown');

    if (versionBtn && versionDropdown) {
        versionBtn.addEventListener('click', (e) => {
            e.stopPropagation();
            const isExpanded = versionBtn.getAttribute('aria-expanded') === 'true';
            versionBtn.setAttribute('aria-expanded', !isExpanded);
            versionDropdown.classList.toggle('show');
        });

        document.addEventListener('click', (e) => {
            if (!versionDropdown.contains(e.target) && !versionBtn.contains(e.target)) {
                versionBtn.setAttribute('aria-expanded', 'false');
                versionDropdown.classList.remove('show');
            }
        });
    }
});
