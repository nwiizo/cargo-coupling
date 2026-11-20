// =====================================================
// Utility Functions
// =====================================================

/**
 * Debounce function
 */
export function debounce(func, wait) {
    let timeout;
    return function executedFunction(...args) {
        const later = () => {
            clearTimeout(timeout);
            func(...args);
        };
        clearTimeout(timeout);
        timeout = setTimeout(later, wait);
    };
}

/**
 * Download a data URL as a file
 */
export function downloadDataUrl(dataUrl, filename) {
    const link = document.createElement('a');
    link.download = filename;
    link.href = dataUrl;
    link.click();
}

/**
 * Download text content as a file
 */
export function downloadText(text, filename, mimeType = 'application/json') {
    const blob = new Blob([text], { type: mimeType });
    const url = URL.createObjectURL(blob);
    downloadDataUrl(url, filename);
    URL.revokeObjectURL(url);
}

/**
 * Get health color based on score/status
 */
export function getHealthColor(health) {
    switch (health) {
        case 'good': return '#22c55e';
        case 'acceptable': return '#eab308';
        case 'needs_review': return '#f97316';
        case 'critical': return '#ef4444';
        default: return '#6b7280';
    }
}

/**
 * Get strength color
 */
export function getStrengthColor(strength) {
    switch (strength) {
        case 'Intrusive': return '#ef4444';
        case 'Functional': return '#f97316';
        case 'Model': return '#eab308';
        case 'Contract': return '#22c55e';
        default: return '#6b7280';
    }
}

/**
 * Format a module path for display
 */
export function formatModulePath(path) {
    if (!path) return '';
    const parts = path.split('::');
    return parts[parts.length - 1] || path;
}

/**
 * Escape HTML to prevent XSS
 */
export function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}
