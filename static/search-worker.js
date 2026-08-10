self.onmessage = event => {
    const { index, query } = event.data;
    const normalized = query.trim().toLowerCase();
    self.postMessage(index.filter(entry =>
        `${entry.v} ${entry.b} ${entry.c} ${entry.t}`.toLowerCase().includes(normalized)
    ).slice(0, 50));
};
