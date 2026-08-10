const indexRequests = new Map();
let latestRequestId = 0;

async function loadIndex(baseUrl, version) {
    if (!indexRequests.has(version)) {
        const request = fetch(`${baseUrl}search-index-${version}.json`)
            .then(response => {
                if (!response.ok) throw new Error(`Search index: ${response.status}`);
                return response.json();
            })
            .then(entries => entries.map(entry => {
                entry.searchText = `${entry.v} ${entry.b} ${entry.c} ${entry.t}`.toLowerCase();
                return entry;
            }))
            .catch(error => {
                indexRequests.delete(version);
                throw error;
            });
        indexRequests.set(version, request);
    }
    return indexRequests.get(version);
}

self.onmessage = async event => {
    const { baseUrl, query, requestId, versions } = event.data;
    latestRequestId = Math.max(latestRequestId, requestId);
    const normalized = query.trim().toLowerCase();
    const matches = [];

    try {
        for (const version of versions) {
            const entries = await loadIndex(baseUrl, version);
            if (requestId !== latestRequestId) return;

            for (const entry of entries) {
                const position = entry.searchText.indexOf(normalized);
                if (position < 0) continue;
                const textPosition = entry.t.toLowerCase().indexOf(normalized);
                matches.push({
                    b: entry.b,
                    c: entry.c,
                    s: textPosition < 0
                        ? entry.t.slice(0, 180)
                        : entry.t.slice(Math.max(0, textPosition - 70), textPosition + 140),
                    u: entry.u,
                    v: entry.v
                });
                if (matches.length === 50) break;
            }
            if (matches.length === 50) break;
        }

        if (requestId === latestRequestId) {
            self.postMessage({ matches, requestId });
        }
    } catch (error) {
        if (requestId === latestRequestId) {
            self.postMessage({ error: error.message, requestId });
        }
    }
};
