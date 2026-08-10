document.addEventListener('DOMContentLoaded', async () => {
    const input = document.querySelector('#search-input');
    const results = document.querySelector('#search-results');
    const status = document.querySelector('#search-status');
    if (!input || !results || !status) return;

    const baseUrl = document.querySelector('html').dataset.baseUrl || '/bible/';
    if ('serviceWorker' in navigator) {
        navigator.serviceWorker.register(`${baseUrl}sw.js`, { scope: baseUrl })
            .catch(error => console.warn('Offline support unavailable:', error));
    }

    try {
        const response = await fetch(`${baseUrl}search-index.json`);
        if (!response.ok) throw new Error(`Search index: ${response.status}`);
        const index = await response.json();

        const render = () => {
            const query = input.value.trim().toLowerCase();
            results.replaceChildren();
            if (query.length < 2) {
                status.textContent = 'Enter at least two characters.';
                return;
            }

            const matches = index.filter(entry =>
                `${entry.v} ${entry.b} ${entry.c} ${entry.t}`.toLowerCase().includes(query)
            ).slice(0, 50);
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

        input.addEventListener('input', render);
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
