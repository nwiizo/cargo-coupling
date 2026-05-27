// =====================================================
// Markdown Report View
// =====================================================

import { CONFIG } from './state.js';

const POLL_INTERVAL_MS = 4000;

let isActive = false;
let pollTimer = null;
let inFlightRequest = null;
let lastFingerprint = '';

export function setupReportView() {
    const reportView = document.getElementById('report-view');
    const reportContent = document.getElementById('report-content');

    if (!reportView || !reportContent) return;

    if (window.marked?.setOptions) {
        window.marked.setOptions({
            gfm: true,
            breaks: false
        });
    }
}

export function showReportView() {
    const reportView = document.getElementById('report-view');
    if (!reportView) return;

    isActive = true;
    reportView.style.display = 'block';
    fetchAndRenderReport();
    startPolling();
}

export function hideReportView() {
    const reportView = document.getElementById('report-view');
    isActive = false;
    stopPolling();

    if (reportView) {
        reportView.style.display = 'none';
    }
}

async function fetchAndRenderReport() {
    if (!isActive) return;
    if (inFlightRequest) return inFlightRequest;

    setReportStatus('Refreshing report...');

    inFlightRequest = fetchReport()
        .then(markdown => {
            const nextFingerprint = contentFingerprint(markdown);
            if (nextFingerprint !== lastFingerprint) {
                lastFingerprint = nextFingerprint;
                renderMarkdown(markdown);
            }
            setReportStatus(`Updated ${new Date().toLocaleTimeString()}`);
        })
        .catch(error => {
            console.error('Failed to refresh report:', error);
            renderReportError(error);
            setReportStatus('Report refresh failed');
        })
        .finally(() => {
            inFlightRequest = null;
        });

    return inFlightRequest;
}

async function fetchReport() {
    const response = await fetch(`${CONFIG.apiEndpoint}${CONFIG.reportPath}`, {
        cache: 'no-store',
        headers: {
            Accept: 'text/markdown'
        }
    });

    if (!response.ok) {
        throw new Error(`HTTP ${response.status}`);
    }

    return response.text();
}

function renderMarkdown(markdown) {
    const reportContent = document.getElementById('report-content');
    if (!reportContent) return;

    if (window.marked?.parse) {
        reportContent.innerHTML = window.marked.parse(markdown);
    } else {
        reportContent.innerHTML = `<pre>${escapeHtml(markdown)}</pre>`;
    }

    decorateHeadings(reportContent);
    buildTableOfContents(reportContent);
}

function decorateHeadings(container) {
    const headings = container.querySelectorAll('h1, h2, h3');
    const seen = new Map();

    headings.forEach(heading => {
        const baseSlug = slugify(heading.textContent || 'section');
        const count = (seen.get(baseSlug) || 0) + 1;
        seen.set(baseSlug, count);
        heading.id = count === 1 ? baseSlug : `${baseSlug}-${count}`;

        const anchor = document.createElement('a');
        anchor.className = 'report-heading-anchor';
        anchor.href = `#${heading.id}`;
        anchor.setAttribute('aria-label', `Link to ${heading.textContent}`);
        anchor.textContent = '#';
        heading.appendChild(anchor);
    });
}

function buildTableOfContents(container) {
    const toc = document.getElementById('report-toc-list');
    if (!toc) return;

    const headings = Array.from(container.querySelectorAll('h1, h2, h3'));
    if (headings.length === 0) {
        toc.innerHTML = '<span class="report-toc-empty">No sections</span>';
        return;
    }

    toc.innerHTML = headings
        .map(heading => {
            const depth = heading.tagName.toLowerCase();
            return `
                <a class="report-toc-link ${depth}" href="#${heading.id}">
                    ${escapeHtml(headingText(heading))}
                </a>
            `;
        })
        .join('');
}

function renderReportError(error) {
    const reportContent = document.getElementById('report-content');
    const toc = document.getElementById('report-toc-list');
    if (toc) toc.innerHTML = '';
    if (!reportContent) return;

    reportContent.innerHTML = `
        <div class="report-error">
            <strong>Could not load report.</strong>
            <span>${escapeHtml(error.message)}</span>
        </div>
    `;
}

function startPolling() {
    if (pollTimer) return;
    pollTimer = window.setInterval(fetchAndRenderReport, POLL_INTERVAL_MS);
}

function stopPolling() {
    if (!pollTimer) return;
    window.clearInterval(pollTimer);
    pollTimer = null;
}

function setReportStatus(message) {
    const status = document.getElementById('report-status');
    if (status) status.textContent = message;
}

function contentFingerprint(text) {
    let hash = 0;
    for (let i = 0; i < text.length; i += 1) {
        hash = ((hash << 5) - hash + text.charCodeAt(i)) | 0;
    }
    return `${text.length}:${hash >>> 0}`;
}

function slugify(text) {
    const slug = text
        .toLowerCase()
        .replace(/[^a-z0-9\s-]/g, '')
        .trim()
        .replace(/\s+/g, '-')
        .replace(/-+/g, '-');

    return slug || 'section';
}

function headingText(heading) {
    return Array.from(heading.childNodes)
        .filter(node => !node.classList?.contains('report-heading-anchor'))
        .map(node => node.textContent || '')
        .join('')
        .trim();
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text || '';
    return div.innerHTML;
}
