document.addEventListener('DOMContentLoaded', function() {
    const verseNumbers = document.querySelectorAll('.verse-number');
    
    verseNumbers.forEach(verseNum => {
        verseNum.style.cursor = 'pointer';
        verseNum.title = 'Click to copy verse text and link';
        
        verseNum.addEventListener('click', function(e) {
            e.preventDefault();
            const verse = this.closest('.verse');
            const verseText = verse.querySelector('.verse-text').textContent.trim();
            const verseRef = verse.getAttribute('data-verse');
            const verseLink = window.location.href.split('#')[0] + '#' + verse.id;
            
            const textToCopy = `${verseRef} ${verseText}\n${verseLink}`;
            
            navigator.clipboard.writeText(textToCopy).then(() => {
                this.textContent = '✓';
                setTimeout(() => {
                    this.textContent = this.getAttribute('data-original');
                }, 1500);
            }).catch(err => {
                console.error('Failed to copy:', err);
            });
        });
        
        // Store original text
        verseNum.setAttribute('data-original', verseNum.textContent);
    });
});
