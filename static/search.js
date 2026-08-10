document.addEventListener('DOMContentLoaded', async () => {
    const input = document.querySelector('#search-input');
    const results = document.querySelector('#search-results');
    const status = document.querySelector('#search-status');
    const versionSelect = document.querySelector('#search-version');
    if (!input || !results || !status) return;

    const baseUrl = document.querySelector('html').dataset.baseUrl || '/bible/';
    const worker = new Worker(`${baseUrl}static/search-worker.js`);
    worker.onmessage = event => {
        const query = input.value.trim().toLowerCase();
        const matches = event.data;
        status.textContent = `${matches.length}${matches.length === 50 ? '+' : ''} result${matches.length === 1 ? '' : 's'}.`;
        matches.forEach(entry => {
            const item = document.createElement('li');
            const link = document.createElement('a');
            link.href = entry.u;
            link.textContent = `${entry.b} ${entry.c} · ${entry.v.toUpperCase()}`;
            const snippet = document.createElement('p');
            const position = entry.t.toLowerCase().indexOf(query);
            snippet.textContent = position < 0 ? entry.t.slice(0, 180) : entry.t.slice(Math.max(0, position - 70), position + 140);
            item.append(link, snippet);
            results.appendChild(item);
        });
    };
    const initialVersion = new URLSearchParams(window.location.search).get('version');
    if (initialVersion && versionSelect?.querySelector(`option[value="${initialVersion}"]`)) {
        versionSelect.value = initialVersion;
    }
    if ('serviceWorker' in navigator) {
        navigator.serviceWorker.register(`${baseUrl}sw.js`, { scope: baseUrl })
            .catch(error => console.warn('Offline support unavailable:', error));
    }

    try {
        let index = [];
        const loaded = new Map();
        const loadVersion = async version => {
            if (!loaded.has(version)) {
                const response = await fetch(`${baseUrl}search-index-${version}.json`);
                if (!response.ok) throw new Error(`Search index: ${response.status}`);
                loaded.set(version, await response.json());
            }
            return loaded.get(version);
        };

        const render = async () => {
            const query = input.value.trim().toLowerCase();
            results.replaceChildren();
            if (query.length < 2) {
                status.textContent = 'Enter at least two characters.';
                return;
            }

            const selectedVersion = versionSelect?.value || 'all';
            const versions = selectedVersion === 'all'
                ? Array.from(versionSelect?.options || []).map(option => option.value).filter(value => value !== 'all')
                : [selectedVersion];
            index = (await Promise.all(versions.map(loadVersion))).flat();
            worker.postMessage({ index, query });
        };

        input.addEventListener('input', render);
        versionSelect?.addEventListener('change', render);
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
