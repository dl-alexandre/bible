document.addEventListener('DOMContentLoaded', () => {
    const bookmarksList = document.querySelector('#study-bookmarks');
    const notesList = document.querySelector('#study-notes');
    const keys = () => Object.keys(localStorage).filter(key => key.startsWith('bible:'));

    function render() {
        bookmarksList.replaceChildren();
        JSON.parse(localStorage.getItem('bible:bookmarks') || '[]').forEach(bookmark => {
            const item = document.createElement('li');
            const link = document.createElement('a');
            link.href = bookmark.url;
            link.textContent = bookmark.label;
            item.appendChild(link);
            bookmarksList.appendChild(item);
        });

        notesList.replaceChildren();
        keys().filter(key => key.includes(':note:')).forEach(key => {
            const note = localStorage.getItem(key);
            if (!note) return;
            const parts = key.split(':');
            const item = document.createElement('li');
            item.textContent = `${parts[1]} ${parts[2]} ${parts[3]}${parts[5] || ''}: ${note}`;
            notesList.appendChild(item);
        });
    }

    document.querySelector('#export-study').addEventListener('click', () => {
        const data = Object.fromEntries(keys().map(key => [key, localStorage.getItem(key)]));
        const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
        const link = document.createElement('a');
        link.href = URL.createObjectURL(blob);
        link.download = 'bible-study.json';
        link.click();
        URL.revokeObjectURL(link.href);
    });
    document.querySelector('#import-study').addEventListener('change', async event => {
        const file = event.target.files[0];
        if (!file) return;
        const data = JSON.parse(await file.text());
        Object.entries(data).filter(([key]) => key.startsWith('bible:')).forEach(([key, value]) => localStorage.setItem(key, value));
        render();
    });
    document.querySelector('#clear-study').addEventListener('click', () => {
        if (!confirm('Clear all bookmarks and notes on this device?')) return;
        keys().forEach(key => localStorage.removeItem(key));
        render();
    });
    render();
});
