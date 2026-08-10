const indexRequests = new Map();
let latestRequestId = 0;
let databasePromise;

function openDatabase() {
    if (databasePromise) return databasePromise;
    databasePromise = new Promise((resolve, reject) => {
        const request = indexedDB.open('bible-search', 1);
        request.onupgradeneeded = () => request.result.createObjectStore('indexes');
        request.onsuccess = () => resolve(request.result);
        request.onerror = () => reject(request.error);
    });
    return databasePromise;
}

async function readCachedIndex(key) {
    try {
        const database = await openDatabase();
        return await new Promise((resolve, reject) => {
            const request = database.transaction('indexes').objectStore('indexes').get(key);
            request.onsuccess = () => resolve(request.result || null);
            request.onerror = () => reject(request.error);
        });
    } catch (_) {
        return null;
    }
}

async function writeCachedIndex(key, entries) {
    try {
        const database = await openDatabase();
        await new Promise((resolve, reject) => {
            const request = database.transaction('indexes', 'readwrite').objectStore('indexes').put(entries, key);
            request.onsuccess = resolve;
            request.onerror = () => reject(request.error);
        });
    } catch (_) {
        // Network cache remains the fallback when IndexedDB is unavailable.
    }
}

async function loadIndex(baseUrl, version) {
    if (!indexRequests.has(version)) {
        const cacheKey = `${baseUrl}:${version}`;
        const request = readCachedIndex(cacheKey).then(async cached => {
            if (cached) return cached;
            const response = await fetch(`${baseUrl}search-index-${version}.json`);
            if (!response.ok) throw new Error(`Search index: ${response.status}`);
            const entries = (await response.json()).map(entry => {
                entry.searchText = `${entry.v} ${entry.b} ${entry.c} ${entry.t}`.toLowerCase();
                return entry;
            });
            await writeCachedIndex(cacheKey, entries);
            return entries;
        })
            .catch(error => {
                indexRequests.delete(version);
                throw error;
            });
        indexRequests.set(version, request);
    }
    return indexRequests.get(version);
}

self.onmessage = async event => {
    const { baseUrl, query, requestId, versions, book } = event.data;
    latestRequestId = Math.max(latestRequestId, requestId);
    const normalized = query.trim().toLowerCase();
    const matches = [];

    function fuzzyScore(text) {
        let cursor = 0;
        let score = 0;
        for (const character of normalized) {
            const found = text.indexOf(character, cursor);
            if (found < 0) return 0;
            score += 1 / (found - cursor + 1);
            cursor = found + 1;
        }
        return score;
    }

    try {
        for (const version of versions) {
            const entries = await loadIndex(baseUrl, version);
            if (requestId !== latestRequestId) return;

            for (const entry of entries) {
                if (book !== 'all' && entry.b !== book) continue;
                const position = entry.searchText.indexOf(normalized);
                const score = position >= 0 ? 1000 - position : fuzzyScore(entry.searchText);
                if (!score) continue;
                const textPosition = entry.t.toLowerCase().indexOf(normalized);
                matches.push({
                    b: entry.b,
                    c: entry.c,
                    score,
                    s: textPosition < 0
                        ? entry.t.slice(0, 180)
                        : entry.t.slice(Math.max(0, textPosition - 70), textPosition + 140),
                    u: entry.u,
                    v: entry.v,
                    j: entry.j
                });
            }
        }

        matches.sort((a, b) => b.score - a.score);
        const candidates = matches.slice(0, 50);
        await Promise.all(candidates.map(async entry => {
            const response = await fetch(entry.j);
            if (!response.ok) return;
            const data = await response.json();
            for (const [number, text] of Object.entries(data.verses || {})) {
                if (text.toLowerCase().includes(normalized)) {
                    entry.a = `#v${number}`;
                    entry.s = text;
                    break;
                }
            }
        }));

        if (requestId === latestRequestId) {
            self.postMessage({ matches: candidates, requestId });
        }
    } catch (error) {
        if (requestId === latestRequestId) {
            self.postMessage({ error: error.message, requestId });
        }
    }
};
