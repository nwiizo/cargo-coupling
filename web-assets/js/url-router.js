// =====================================================
// URL Router - URL-based Navigation
// =====================================================

/**
 * Get the initial selection from URL parameters
 * @returns {{ module?: string, item?: string, view?: string, items?: boolean }} Initial selection
 */
export function getInitialSelection() {
    const params = new URLSearchParams(window.location.search);

    return {
        module: params.get('module') || null,
        item: params.get('item') || null,
        view: params.get('view') || null,
        items: params.get('items') === 'true'
    };
}

/**
 * Update the URL with the current selection
 * Uses replaceState to avoid polluting browser history
 * @param {string|null} moduleId - Module ID to include in URL
 * @param {string|null} itemName - Item name to include in URL (optional)
 */
export function updateUrl(moduleId, itemName = null) {
    const params = new URLSearchParams(window.location.search);

    if (moduleId) {
        params.set('module', moduleId);
        if (itemName) {
            params.set('item', itemName);
        } else {
            params.delete('item');
        }
    } else {
        params.delete('module');
        params.delete('item');
    }

    // Preserve other params like view
    const newUrl = params.toString()
        ? `${window.location.pathname}?${params.toString()}`
        : window.location.pathname;

    window.history.replaceState({ module: moduleId, item: itemName }, '', newUrl);
}

/**
 * Push a new URL state (creates history entry)
 * Use this when user explicitly navigates
 * @param {string|null} moduleId - Module ID
 * @param {string|null} itemName - Item name
 */
export function pushUrl(moduleId, itemName = null) {
    const params = new URLSearchParams();

    if (moduleId) {
        params.set('module', moduleId);
        if (itemName) {
            params.set('item', itemName);
        }
    }

    // Preserve view param if present
    const currentView = new URLSearchParams(window.location.search).get('view');
    if (currentView) {
        params.set('view', currentView);
    }

    const newUrl = params.toString()
        ? `${window.location.pathname}?${params.toString()}`
        : window.location.pathname;

    window.history.pushState({ module: moduleId, item: itemName }, '', newUrl);
}

/**
 * Initialize URL router with navigation callback
 * @param {Function} onNavigate - Callback when URL changes (module, item) => void
 */
export function initUrlRouter(onNavigate) {
    if (!onNavigate) return;

    window.addEventListener('popstate', (event) => {
        const state = event.state;
        if (state) {
            onNavigate(state.module, state.item);
        } else {
            // Parse from URL if no state
            const selection = getInitialSelection();
            onNavigate(selection.module, selection.item);
        }
    });
}

/**
 * Build a shareable URL for a module/item
 * @param {string} moduleId - Module ID
 * @param {string|null} itemName - Item name (optional)
 * @returns {string} Full URL
 */
export function buildShareUrl(moduleId, itemName = null) {
    const params = new URLSearchParams();
    params.set('module', moduleId);
    if (itemName) {
        params.set('item', itemName);
    }
    return `${window.location.origin}${window.location.pathname}?${params.toString()}`;
}

/**
 * Copy shareable URL to clipboard
 * @param {string} moduleId - Module ID
 * @param {string|null} itemName - Item name
 * @returns {Promise<boolean>} Success status
 */
export async function copyShareUrl(moduleId, itemName = null) {
    try {
        const url = buildShareUrl(moduleId, itemName);
        await navigator.clipboard.writeText(url);
        return true;
    } catch (e) {
        console.error('Failed to copy URL:', e);
        return false;
    }
}
