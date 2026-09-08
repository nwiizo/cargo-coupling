/** Stable, shared labels for 2D and 3D. Full item names remain in the inspector. */
export function moduleLabel(node, previewCount = 3) {
    const items = [...(node.items || [])].sort((a, b) => {
        const visibility = item => ['pub', 'Public'].includes(item.visibility) ? 0 : 1;
        return visibility(a) - visibility(b) || a.name.localeCompare(b.name);
    });
    const lines = [node.label || node.id];
    for (const item of items.slice(0, previewCount)) lines.push(`${item.kind === 'fn' ? 'fn ' : ''}${item.name}`);
    if (items.length > previewCount) lines.push(`+${items.length - previewCount}`);
    return lines.join('\n');
}

/** Place screen labels without intersecting previously occupied rectangles. */
export function placeLabel(point, size, occupied, viewport) {
    if (![point.x, point.y, size.width, size.height].every(Number.isFinite)) return null;
    for (const [dx, dy] of [[12, -size.height / 2], [-size.width - 12, -size.height / 2], [12, 16], [12, -size.height - 16], [-size.width / 2, 24], [-size.width / 2, -size.height - 24]]) {
        const rect = { x: point.x + dx, y: point.y + dy, ...size };
        if (rect.x < 4 || rect.y < 4 || rect.x + rect.width > viewport.width - 4 || rect.y + rect.height > viewport.height - 4) continue;
        if (occupied.some(other => rect.x < other.x + other.width + 6 && rect.x + rect.width + 6 > other.x && rect.y < other.y + other.height + 6 && rect.y + rect.height + 6 > other.y)) continue;
        return rect;
    }
    return null;
}
