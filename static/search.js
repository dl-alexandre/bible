document.addEventListener('DOMContentLoaded', async () => {
    const input = document.querySelector('#search-input');
    const results = document.querySelector('#search-results');
    const status = document.querySelector('#search-status');
    const versionSelect = document.querySelector('#search-version');
    const bookSelect = document.querySelector('#search-book');
    if (!input || !results || !status) return;

    const baseUrl = document.querySelector('html').dataset.baseUrl || '/bible/';
    const worker = new Worker(`${baseUrl}static/search-worker.js`);
    let debounceTimer;
    let latestRequestId = 0;

    worker.onmessage = event => {
        if (event.data.requestId !== latestRequestId) return;
        if (event.data.error) {
            status.textContent = 'Search is unavailable offline until the selected index has been opened once.';
            console.warn(event.data.error);
            return;
        }

        const query = input.value.trim().toLowerCase();
        const matches = event.data.matches;
        results.replaceChildren();
        status.textContent = `${matches.length}${matches.length === 50 ? '+' : ''} result${matches.length === 1 ? '' : 's'}.`;
        matches.forEach(entry => {
            const item = document.createElement('li');
            const link = document.createElement('a');
            link.href = `${entry.u}${entry.a || ''}`;
            link.textContent = `${entry.b} ${entry.c} · ${entry.v.toUpperCase()}`;
            const snippet = document.createElement('p');
            const pattern = new RegExp(`(${query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')})`, 'ig');
            entry.s.split(pattern).forEach(part => {
                if (part.toLowerCase() === query) {
                    const mark = document.createElement('mark');
                    mark.textContent = part;
                    snippet.appendChild(mark);
                } else {
                    snippet.appendChild(document.createTextNode(part));
                }
            });
            item.append(link, snippet);
            results.appendChild(item);
        });
    };
    const initialVersion = new URLSearchParams(window.location.search).get('version');
    if (initialVersion && versionSelect?.querySelector(`option[value="${initialVersion}"]`)) {
        versionSelect.value = initialVersion;
    }
    const initialBook = new URLSearchParams(window.location.search).get('book');
    if (initialBook && bookSelect?.querySelector(`option[value="${initialBook}"]`)) {
        bookSelect.value = initialBook;
    }
    if ('serviceWorker' in navigator) {
        navigator.serviceWorker.register(`${baseUrl}sw.js`, { scope: baseUrl })
            .catch(error => console.warn('Offline support unavailable:', error));
    }

    try {
        const render = () => {
            const query = input.value.trim().toLowerCase();
            results.replaceChildren();
            clearTimeout(debounceTimer);
            latestRequestId += 1;
            if (query.length < 2) {
                status.textContent = 'Enter at least two characters.';
                return;
            }

            const selectedVersion = versionSelect?.value || 'all';
            const selectedBook = bookSelect?.value || 'all';
            const versions = selectedVersion === 'all'
                ? Array.from(versionSelect?.options || []).map(option => option.value).filter(value => value !== 'all')
                : [selectedVersion];
            const requestId = latestRequestId;
            status.textContent = 'Searching...';
            debounceTimer = setTimeout(() => {
                worker.postMessage({ baseUrl, query, requestId, versions, book: selectedBook });
            }, 200);
        };

        input.addEventListener('input', render);
        versionSelect?.addEventListener('change', render);
        bookSelect?.addEventListener('change', render);
        const initialQuery = new URLSearchParams(window.location.search).get('q');
        if (initialQuery) {
            input.value = initialQuery;
            render();
        }
        input.addEventListener('change', () => {
            const url = new URL(window.location.href);
            url.searchParams.set('q', input.value);
            if (versionSelect?.value && versionSelect.value !== 'all') url.searchParams.set('version', versionSelect.value);
            else url.searchParams.delete('version');
            if (bookSelect?.value && bookSelect.value !== 'all') url.searchParams.set('book', bookSelect.value);
            else url.searchParams.delete('book');
            history.replaceState(null, '', url);
        });
        document.addEventListener('keydown', event => {
            if (event.key === '/' && document.activeElement !== input) {
                event.preventDefault();
                input.focus();
            }
        });
    } catch (error) {
        status.textContent = 'Search is unavailable offline until the index has been opened once.';
        console.warn(error);
    }
});
