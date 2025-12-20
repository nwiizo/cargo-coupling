// =====================================================
// UI Components Module
// =====================================================

import { state, setSelectedNode, setCenterMode } from './state.js';
import { t } from './i18n.js';
import { applyLayout, clearHighlights, centerOnNode, focusOnNode, highlightNeighbors, highlightDependencyPath, analyzeCoupling, getHealthColor } from './graph.js';
import { debounce, escapeHtml, estimateVolatility } from './utils.js';

// =====================================================
// Header & Footer Stats
// =====================================================

export function updateHeaderStats(summary, graphData) {
    const container = document.getElementById('header-stats');
    if (!container || !summary) return;

    let totalFunctions = 0;
    let totalTypes = 0;
    let totalImpls = 0;

    if (graphData?.nodes) {
        for (const node of graphData.nodes) {
            totalFunctions += node.metrics?.fn_count || 0;
            totalTypes += node.metrics?.type_count || 0;
            totalImpls += node.metrics?.impl_count || 0;
        }
    }

    container.innerHTML = `
        <span class="stat">Modules: <span class="stat-value">${summary.total_modules}</span></span>
        <span class="stat">Functions: <span class="stat-value">${totalFunctions}</span></span>
        <span class="stat">Types: <span class="stat-value">${totalTypes}</span></span>
        <span class="stat">Impls: <span class="stat-value">${totalImpls}</span></span>
        <span class="stat">Health: <span class="health-grade ${summary.health_grade}">${summary.health_grade}</span></span>
    `;
}

export function updateFooterStats(summary) {
    const container = document.getElementById('summary-stats');
    if (!container || !summary) return;

    const issueCount = Object.values(summary.issues_by_severity || {}).reduce((a, b) => a + b, 0);
    container.innerHTML = `
        <span>Issues: ${issueCount}</span>
        <span>Health Score: ${(summary.health_score * 100).toFixed(1)}%</span>
    `;
}

// =====================================================
// Filters
// =====================================================

export function setupFilters() {
    const applyFilters = () => {
        if (!state.cy) return;

        const strengths = getCheckedValues('strength-filters');
        const distances = getCheckedValues('distance-filters');
        const volatilities = getCheckedValues('volatility-filters');
        const balanceMin = parseInt(document.getElementById('balance-min')?.value || 0) / 100;
        const balanceMax = parseInt(document.getElementById('balance-max')?.value || 100) / 100;
        const issuesOnly = document.getElementById('show-issues-only')?.checked;
        const cyclesOnly = document.getElementById('show-cycles-only')?.checked;
        const hideExternal = document.getElementById('hide-external')?.checked;

        // First, determine which nodes are internal (have source file path)
        const internalNodes = new Set();
        state.cy.nodes().forEach(node => {
            const filePath = node.data('file_path');
            const isInternal = filePath && !filePath.startsWith('[external]');
            if (isInternal) {
                internalNodes.add(node.id());
            }
        });

        state.cy.edges().forEach(edge => {
            const strength = edge.data('strengthLabel') || 'Model';
            const distance = edge.data('distance') || 'DifferentModule';
            const volatility = edge.data('volatility') || 'Low';
            const balance = edge.data('balance') ?? 0.5;
            const hasIssue = edge.data('issue');
            const inCycle = edge.data('inCycle');
            const edgeType = edge.data('edgeType');

            // Skip filtering for parent/item edges
            if (edgeType === 'parent' || edgeType === 'item-dep') {
                edge.style('display', 'element');
                return;
            }

            // Check if this is an internal edge
            const sourceInternal = internalNodes.has(edge.data('source'));
            const targetInternal = internalNodes.has(edge.data('target'));
            const isInternalEdge = sourceInternal && targetInternal;

            let visible = true;
            if (!strengths.includes(strength)) visible = false;

            // Skip distance filter for internal edges when hideExternal is on
            // (internal edges may have incorrect distance values due to path resolution)
            if (!hideExternal || !isInternalEdge) {
                if (!distances.includes(distance)) visible = false;
            }

            if (!volatilities.includes(volatility)) visible = false;
            if (balance < balanceMin || balance > balanceMax) visible = false;
            if (issuesOnly && !hasIssue) visible = false;
            if (cyclesOnly && !inCycle) visible = false;

            // Hide edges to/from external nodes if hide-external is checked
            if (hideExternal && !isInternalEdge) {
                visible = false;
            }

            edge.style('display', visible ? 'element' : 'none');
        });

        // Hide nodes with no visible edges OR external nodes if hideExternal is checked
        state.cy.nodes().forEach(node => {
            const nodeType = node.data('nodeType');
            const filePath = node.data('file_path');
            const isExternal = filePath && filePath.startsWith('[external]');
            const isInternal = filePath && !filePath.startsWith('[external]');

            // Always show item nodes if their parent is visible
            if (nodeType === 'item') {
                const parentVisible = state.cy.getElementById(node.data('parentModule')).style('display') !== 'none';
                node.style('display', parentVisible ? 'element' : 'none');
                return;
            }

            // Hide external nodes if filter is on
            if (hideExternal && isExternal) {
                node.style('display', 'none');
                return;
            }

            const visibleEdges = node.connectedEdges().filter(e => e.style('display') !== 'none');
            const nodeVisible = visibleEdges.length > 0 || isInternal;
            node.style('display', nodeVisible ? 'element' : 'none');
        });

        const balanceLabel = document.getElementById('balance-value');
        if (balanceLabel) {
            balanceLabel.textContent = `${balanceMin.toFixed(1)} - ${balanceMax.toFixed(1)}`;
        }
    };

    // Attach filter listeners
    document.querySelectorAll('#strength-filters input, #distance-filters input, #volatility-filters input').forEach(cb => {
        cb.addEventListener('change', applyFilters);
    });

    ['balance-min', 'balance-max'].forEach(id => {
        document.getElementById(id)?.addEventListener('input', applyFilters);
    });

    document.getElementById('show-issues-only')?.addEventListener('change', applyFilters);
    document.getElementById('show-cycles-only')?.addEventListener('change', applyFilters);
    document.getElementById('hide-external')?.addEventListener('change', applyFilters);

    document.getElementById('reset-filters')?.addEventListener('click', () => {
        document.querySelectorAll('#strength-filters input, #volatility-filters input').forEach(cb => cb.checked = true);
        document.querySelectorAll('#distance-filters input').forEach(cb => {
            cb.checked = cb.value !== 'DifferentCrate';
        });
        document.getElementById('balance-min').value = 0;
        document.getElementById('balance-max').value = 100;
        document.getElementById('show-issues-only').checked = false;
        document.getElementById('show-cycles-only').checked = false;
        document.getElementById('hide-external').checked = true; // Default: hide external

        state.cy.elements().removeClass('hidden highlighted dimmed dependency-source dependency-target search-match');
        applyFilters();
    });

    document.getElementById('fit-graph')?.addEventListener('click', () => state.cy?.fit(undefined, 50));

    setTimeout(() => {
        applyFilters();
        state.cy?.fit(undefined, 50);
    }, 100);
}

// =====================================================
// Search
// =====================================================

export function setupSearch() {
    const input = document.getElementById('search-input');
    if (!input || !state.cy) return;

    input.addEventListener('input', (e) => {
        const query = e.target.value.toLowerCase().trim();
        state.cy.nodes().removeClass('search-match dimmed');

        if (!query) return;

        const matches = state.cy.nodes().filter(n => n.data('label').toLowerCase().includes(query));
        if (matches.length > 0) {
            state.cy.nodes().addClass('dimmed');
            matches.removeClass('dimmed').addClass('search-match');
            state.cy.fit(matches, 50);
        }
    });

    input.addEventListener('keydown', (e) => {
        if (e.key === 'Escape') {
            input.value = '';
            state.cy.nodes().removeClass('search-match dimmed');
        }
    });
}

// =====================================================
// Layout Selector
// =====================================================

export function setupLayoutSelector() {
    const select = document.getElementById('layout-select');
    if (!select) return;

    select.addEventListener('change', (e) => {
        applyLayout(e.target.value);
    });
}

// =====================================================
// Export Buttons
// =====================================================

export function setupExportButtons() {
    document.getElementById('export-png')?.addEventListener('click', () => exportGraph('png'));
    document.getElementById('export-json')?.addEventListener('click', () => exportGraph('json'));
}

function exportGraph(format) {
    if (format === 'png' && state.cy) {
        const png = state.cy.png({ full: true, scale: 2, bg: '#0f172a' });
        const link = document.createElement('a');
        link.href = png;
        link.download = 'coupling-graph.png';
        link.click();
    } else if (format === 'json' && state.graphData) {
        const json = JSON.stringify(state.graphData, null, 2);
        const blob = new Blob([json], { type: 'application/json' });
        const link = document.createElement('a');
        link.href = URL.createObjectURL(blob);
        link.download = 'coupling-data.json';
        link.click();
    }
}

// =====================================================
// Keyboard Shortcuts
// =====================================================

export function setupKeyboardShortcuts(callbacks = {}) {
    document.addEventListener('keydown', (e) => {
        if (e.target.tagName === 'INPUT' || e.target.tagName === 'SELECT') return;

        switch (e.key) {
            case '/':
                e.preventDefault();
                document.getElementById('search-input')?.focus();
                break;
            case 'f':
                state.cy?.fit(undefined, 50);
                break;
            case 'r':
                applyLayout(state.currentLayout);
                break;
            case 'e':
                exportGraph('png');
                break;
            case 's':
                document.getElementById('sidebar-toggle')?.click();
                break;
            case 'c':
                setCenterMode(!state.centerMode);
                const toggle = document.getElementById('center-mode-toggle');
                if (toggle) toggle.checked = state.centerMode;
                break;
            case 'Escape':
                if (callbacks.onEscape) callbacks.onEscape();
                break;
            case '?':
                toggleHelpModal();
                break;
        }
    });
}

export function toggleHelpModal() {
    const modal = document.getElementById('help-modal');
    if (modal) modal.classList.toggle('visible');
}

// =====================================================
// Center Mode Toggle
// =====================================================

export function setupCenterModeToggle() {
    const toggle = document.getElementById('center-mode-toggle');
    if (toggle) {
        toggle.addEventListener('change', (e) => {
            setCenterMode(e.target.checked);
        });
    }
}

// =====================================================
// Legend Toggle
// =====================================================

export function setupLegendToggle() {
    const toggle = document.getElementById('legend-toggle');
    const content = document.getElementById('legend-content');
    if (toggle && content) {
        toggle.addEventListener('click', () => {
            const isHidden = content.style.display === 'none';
            content.style.display = isHidden ? 'block' : 'none';
            toggle.textContent = isHidden ? '▼' : '▶';
        });
    }
}

// =====================================================
// Node/Edge Details
// =====================================================

export function showNodeDetails(data) {
    const container = document.getElementById('node-details');
    if (!container) return;

    const fullNode = state.graphData?.nodes?.find(n => n.id === data.id);
    const fnCount = data.fn_count || fullNode?.metrics?.fn_count || 0;
    const typeCount = data.type_count || fullNode?.metrics?.type_count || 0;
    const implCount = data.impl_count || fullNode?.metrics?.impl_count || 0;

    container.innerHTML = `
        <div class="detail-header">${escapeHtml(data.label)}</div>
        <div class="detail-stats">
            <span class="stat-badge fn">${fnCount} fn</span>
            <span class="stat-badge type">${typeCount} type</span>
            <span class="stat-badge impl">${implCount} impl</span>
        </div>
        <div class="detail-row">
            <span class="detail-label">Health:</span>
            <span class="health-indicator ${data.health || 'good'}">${data.health || 'good'}</span>
        </div>
        <div class="detail-row">
            <span class="detail-label">Balance Score:</span>
            <span>${((data.balance_score || 0) * 100).toFixed(0)}%</span>
        </div>
        <div class="detail-row">
            <span class="detail-label">Outgoing:</span>
            <span>${data.couplings_out || 0}</span>
        </div>
        <div class="detail-row">
            <span class="detail-label">Incoming:</span>
            <span>${data.couplings_in || 0}</span>
        </div>
        ${data.file_path ? `<div class="detail-row file-path">${escapeHtml(data.file_path)}</div>` : ''}
    `;
}

export function showEdgeDetails(data) {
    const container = document.getElementById('node-details');
    if (!container) return;

    const analysis = analyzeCoupling(
        data.strengthLabel || 'Model',
        data.distance || 'DifferentModule',
        data.volatility || 'Medium',
        data.target || ''
    );

    const effectiveVol = estimateVolatility(data.target || '', data.volatility || 'Medium');

    container.innerHTML = `
        <div class="detail-header">Coupling Details</div>
        <div class="detail-row">
            <span class="detail-label">Source:</span>
            <span>${escapeHtml(data.source)}</span>
        </div>
        <div class="detail-row">
            <span class="detail-label">Target:</span>
            <span>${escapeHtml(data.target)}</span>
        </div>
        <hr class="detail-divider">
        <div class="detail-row">
            <span class="detail-label">${t('strength')}:</span>
            <span class="strength-badge ${(data.strengthLabel || '').toLowerCase()}">${data.strengthLabel || 'Model'}</span>
        </div>
        <div class="detail-row">
            <span class="detail-label">${t('distance')}:</span>
            <span>${data.distance || 'DifferentModule'}</span>
        </div>
        <div class="detail-row">
            <span class="detail-label">${t('volatility')}:</span>
            <span>${effectiveVol}</span>
        </div>
        <div class="detail-row">
            <span class="detail-label">${t('balance')}:</span>
            <span>${((data.balance || 0) * 100).toFixed(0)}%</span>
        </div>
        <hr class="detail-divider">
        <div class="analysis-result ${analysis.status}">
            <span class="analysis-icon">${analysis.icon}</span>
            <span class="analysis-text">${analysis.statusText}</span>
        </div>
        ${analysis.action ? `<div class="analysis-action">${analysis.action}</div>` : ''}
        ${data.classification ? `<div class="classification-badge">${state.currentLang === 'ja' ? data.classificationJa : data.classification}</div>` : ''}
    `;
}

export function clearDetails() {
    const container = document.getElementById('node-details');
    if (container) {
        container.innerHTML = '<div class="detail-placeholder">Select a node or edge to view details</div>';
    }
}

// =====================================================
// Blast Radius
// =====================================================

export function showBlastRadius(node) {
    if (!state.cy || !node) return;

    highlightNeighbors(node);

    const incoming = node.incomers('node').length;
    const outgoing = node.outgoers('node').length;
    const total = incoming + outgoing;

    const container = document.getElementById('blast-radius');
    if (container) {
        container.innerHTML = `
            <div class="blast-header">Impact Radius</div>
            <div class="blast-stats">
                <span class="blast-stat">
                    <span class="blast-label">Affects:</span>
                    <span class="blast-value">${outgoing}</span>
                </span>
                <span class="blast-stat">
                    <span class="blast-label">Affected by:</span>
                    <span class="blast-value">${incoming}</span>
                </span>
                <span class="blast-stat total">
                    <span class="blast-label">Total:</span>
                    <span class="blast-value">${total}</span>
                </span>
            </div>
        `;
    }
}

export function clearBlastRadius() {
    const container = document.getElementById('blast-radius');
    if (container) {
        container.innerHTML = '';
    }
    clearHighlights();
}

// =====================================================
// Sidebar Resize
// =====================================================

export function setupResizableSidebar() {
    const sidebar = document.querySelector('.sidebar');
    const resizer = document.getElementById('sidebar-resizer');
    if (!sidebar || !resizer) return;

    let isResizing = false;
    let startX, startWidth;

    resizer.addEventListener('mousedown', (e) => {
        isResizing = true;
        startX = e.clientX;
        startWidth = sidebar.offsetWidth;
        document.body.style.cursor = 'col-resize';
        document.body.style.userSelect = 'none';
    });

    document.addEventListener('mousemove', (e) => {
        if (!isResizing) return;
        const width = startWidth - (e.clientX - startX);
        if (width >= 250 && width <= 600) {
            sidebar.style.width = `${width}px`;
        }
    });

    document.addEventListener('mouseup', () => {
        if (isResizing) {
            isResizing = false;
            document.body.style.cursor = '';
            document.body.style.userSelect = '';
        }
    });
}

// =====================================================
// Utilities
// =====================================================

function getCheckedValues(containerId) {
    return Array.from(document.querySelectorAll(`#${containerId} input:checked`)).map(cb => cb.value);
}
