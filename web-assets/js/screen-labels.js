import { placeLabel } from './module-labels.js';

/** Shared projected text. Pointer gestures pass to the canvas; keyboard users
 * can focus a name and select it with Enter. */
export class ScreenLabels {
    constructor(container, onSelect) {
        this.onSelect = onSelect;
        this.labels = new Map();
        this.layer = document.createElement('div'); this.layer.className = 'graph-screen-labels';
        this.layer.setAttribute('aria-label', 'Module and item names'); container.append(this.layer);
    }

    render(entries) {
        const viewport = { width: this.layer.clientWidth, height: this.layer.clientHeight };
        if (!viewport.width || !viewport.height) return;
        const current = new Set(entries.map(entry => entry.id));
        for (const [id, item] of this.labels) if (!current.has(id)) { item.node.remove(); this.labels.delete(id); }
        const occupied = [];
        for (const entry of entries) {
            let item = this.labels.get(entry.id);
            if (!item) {
                const node = document.createElement('button'); node.type = 'button'; node.className = 'graph-screen-label';
                item = { node }; this.labels.set(entry.id, item); this.layer.append(node);
            }
            const label = item.node;
            label.onclick = () => this.onSelect(entry.id);
            if (label.textContent !== entry.text) { label.textContent = entry.text; item.size = undefined; }
            label.title = entry.id;
            if (!item.size) { label.hidden = false; item.size = { width: label.offsetWidth, height: label.offsetHeight }; }
            const rect = placeLabel(entry, item.size, occupied, viewport);
            label.hidden = !rect;
            if (rect) { occupied.push(rect); label.style.left = `${rect.x}px`; label.style.top = `${rect.y}px`; }
        }
    }
}
