// =====================================================
// UI Components Module
// =====================================================

import { state, setSelectedNode, setCenterMode } from './state.js';
import { t } from './i18n.js';
import { applyLayout, clearHighlights, centerOnNode, focusOnNode, highlightNeighbors, highlightDependencyPath, analyzeCoupling, getHealthColor } from './graph.js';
import { debounce, escapeHtml, estimateVolatility } from './utils.js';
import { updateUrl } from './url-router.js';

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

/**
 * Apply current filter settings to the graph
 */
export function applyFilters() {
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
}

/**
 * Setup filter event listeners
 */
export function setupFilters() {
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
    const metrics = fullNode?.metrics || {};
    const fnCount = data.fn_count || metrics.fn_count || 0;
    const typeCount = data.type_count || metrics.type_count || 0;
    const implCount = data.impl_count || metrics.impl_count || 0;
    const traitImplCount = metrics.trait_impl_count || 0;
    const inherentImplCount = metrics.inherent_impl_count || 0;
    const filePath = data.file_path || fullNode?.file_path;
    const isExternal = filePath && filePath.startsWith('[external]');
    const items = data.items || fullNode?.items || [];
    const inCycle = data.in_cycle || fullNode?.in_cycle || false;
    const volatility = data.volatility || metrics.volatility || 'Medium';

    // Group items by kind
    const types = items.filter(i => i.kind === 'type' || i.kind === 'struct' || i.kind === 'enum');
    const traits = items.filter(i => i.kind === 'trait');
    const functions = items.filter(i => i.kind === 'fn');

    const renderItemList = (itemList, kind, icon) => {
        if (itemList.length === 0) return '';
        const isPub = (vis) => vis === 'pub' || vis === 'Public';
        return `
            <div class="item-list-section" data-kind="${kind}">
                <div class="item-list-header">
                    <span class="item-list-icon">${icon}</span>
                    <span class="item-list-title">${kind}</span>
                    <span class="item-list-count">${itemList.length}</span>
                </div>
                <div class="item-list-items">
                    ${itemList.slice(0, 10).map(item => `
                        <div class="item-list-item ${isPub(item.visibility) ? 'public' : 'private'}"
                             data-module="${data.id}"
                             data-item="${item.name}">
                            <span class="item-name">${escapeHtml(item.name)}</span>
                            ${isPub(item.visibility) ? '<span class="pub-badge">pub</span>' : ''}
                        </div>
                    `).join('')}
                    ${itemList.length > 10 ? `<div class="item-list-more">+${itemList.length - 10} more</div>` : ''}
                </div>
            </div>
        `;
    };

    const getVolatilityClass = (vol) => {
        if (vol === 'High') return 'high';
        if (vol === 'Medium') return 'medium';
        return 'low';
    };

    container.innerHTML = `
        ${inCycle ? `
            <div class="warning-banner critical">
                ⚠️ This module is part of a circular dependency
            </div>
        ` : ''}
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
            <span class="detail-label">Volatility:</span>
            <span class="volatility-badge ${getVolatilityClass(volatility)}">${volatility}</span>
        </div>
        <div class="detail-row">
            <span class="detail-label">Outgoing:</span>
            <span>${data.couplings_out || 0}</span>
        </div>
        <div class="detail-row">
            <span class="detail-label">Incoming:</span>
            <span>${data.couplings_in || 0}</span>
        </div>
        ${implCount > 0 ? `
            <div class="impl-breakdown">
                <div class="impl-item">
                    <span class="label">Trait impl:</span>
                    <span class="count">${traitImplCount}</span>
                </div>
                <div class="impl-item">
                    <span class="label">Inherent impl:</span>
                    <span class="count">${inherentImplCount}</span>
                </div>
            </div>
        ` : ''}
        ${filePath && !isExternal ? `
            <div class="file-path-display">
                <span class="file-path-label">File:</span>
                <span class="file-path-value" title="${escapeHtml(filePath)}">${escapeHtml(filePath)}</span>
            </div>
            <button class="btn-view-code" data-path="${escapeHtml(filePath)}">
                <span class="icon">📄</span> View Source Code
            </button>
        ` : ''}
        <div id="source-code-panel"></div>
        ${items.length > 0 ? `
            <div class="module-items-section">
                <div class="module-items-header">
                    <span>📦 Module Contents</span>
                    <span class="hint">(click to focus in graph)</span>
                </div>
                ${renderItemList(types, 'Types', 'S')}
                ${renderItemList(traits, 'Traits', 'T')}
                ${renderItemList(functions, 'Functions', 'ƒ')}
            </div>
        ` : ''}
        <button class="btn-expand-details" data-module-id="${data.id}">
            <span>⤢</span> Full Details (Modal)
        </button>
    `;

    // Setup expand details button
    const expandBtn = container.querySelector('.btn-expand-details');
    expandBtn?.addEventListener('click', () => {
        showDetailsModal(data);
    });

    // Setup view code button
    if (filePath && !isExternal) {
        const btn = container.querySelector('.btn-view-code');
        btn?.addEventListener('click', () => {
            loadSourceCode(filePath);
        });
    }

    // Setup item click handlers
    container.querySelectorAll('.item-list-item').forEach(item => {
        item.addEventListener('click', () => {
            const moduleId = item.dataset.module;
            const itemName = item.dataset.item;
            focusOnItem(moduleId, itemName);
        });
    });
}

/**
 * Focus on a specific item in the graph
 */
export function focusOnItem(moduleId, itemName) {
    if (!state.cy) return;

    const itemNodeId = `${moduleId}::${itemName}`;

    // Check if item node exists
    let itemNode = state.cy.getElementById(itemNodeId);

    if (!itemNode || itemNode.length === 0) {
        // Item node doesn't exist - need to enable Show Items
        const showItemsToggle = document.getElementById('show-items-toggle');
        if (showItemsToggle && !showItemsToggle.checked) {
            // Dispatch event to trigger rebuild with items
            showItemsToggle.checked = true;
            showItemsToggle.dispatchEvent(new Event('change'));

            // Wait for rebuild and then focus
            setTimeout(() => {
                itemNode = state.cy?.getElementById(itemNodeId);
                if (itemNode?.length) {
                    selectAndFocusItem(itemNode);
                    // Update URL with module and item
                    updateUrl(moduleId, itemName);
                }
            }, 800);
        }
    } else {
        selectAndFocusItem(itemNode);
        // Update URL with module and item
        updateUrl(moduleId, itemName);
    }
}

/**
 * Select and focus on an item node
 */
function selectAndFocusItem(node) {
    if (!state.cy || !node) return;

    // Clear previous highlights
    state.cy.elements().removeClass('highlighted dimmed search-match');

    // Highlight the item and its connections
    state.cy.elements().addClass('dimmed');
    node.removeClass('dimmed').addClass('highlighted search-match');
    node.neighborhood().removeClass('dimmed');

    // Center on the item
    state.cy.animate({
        center: { eles: node },
        zoom: 2,
        duration: 500,
        easing: 'ease-out-cubic'
    });
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
    const location = data.location;
    const hasLocation = location && location.file_path && !location.file_path.startsWith('[external]');
    const inCycle = data.inCycle || false;
    const issue = data.issue;
    const connascence = data.connascence;
    const interpretation = data.interpretation;

    // Determine issue severity class
    const getIssueSeverityClass = (severity) => {
        if (!severity) return 'medium';
        const s = severity.toLowerCase();
        if (s === 'critical' || s === 'high') return 'critical';
        if (s === 'medium') return 'medium';
        return 'low';
    };

    container.innerHTML = `
        ${inCycle ? `
            <div class="warning-banner critical">
                ⚠️ Part of a circular dependency chain
            </div>
        ` : ''}
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
            <span class="volatility-badge ${effectiveVol.toLowerCase()}">${effectiveVol}</span>
        </div>
        <div class="detail-row">
            <span class="detail-label">${t('balance')}:</span>
            <span>${((data.balance || 0) * 100).toFixed(0)}%</span>
        </div>
        ${connascence ? `
            <div class="connascence-info">
                <span class="connascence-type">${connascence.type || 'Identity'}</span>
                <span class="connascence-strength">${connascence.strength || 'Weak'}</span>
            </div>
        ` : ''}
        <hr class="detail-divider">
        <div class="analysis-result ${analysis.status}">
            <span class="analysis-icon">${analysis.icon}</span>
            <span class="analysis-text">${analysis.statusText}</span>
        </div>
        ${analysis.action ? `<div class="analysis-action">${analysis.action}</div>` : ''}
        ${interpretation ? `
            <div class="balance-interpretation">
                ${escapeHtml(interpretation)}
            </div>
        ` : ''}
        ${issue ? `
            <div class="issue-detail ${getIssueSeverityClass(issue.severity)}">
                <div class="issue-type">${escapeHtml(issue.type || issue.issue_type || 'Issue')}</div>
                <div class="issue-description">${escapeHtml(issue.description || issue.message || '')}</div>
            </div>
        ` : ''}
        ${data.classification ? `<div class="classification-badge">${state.currentLang === 'ja' ? data.classificationJa : data.classification}</div>` : ''}
        ${hasLocation ? `
            <button class="btn-view-code" data-path="${escapeHtml(location.file_path)}" data-line="${location.line || 0}">
                <span class="icon">📄</span> View Coupling Location
            </button>
        ` : ''}
        <div id="source-code-panel"></div>
    `;

    // Setup view code button for edge
    if (hasLocation) {
        const btn = container.querySelector('.btn-view-code');
        btn?.addEventListener('click', () => {
            loadSourceCode(location.file_path, location.line || null, 10);
        });
    }
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
        e.preventDefault();
        isResizing = true;
        startX = e.clientX;
        startWidth = sidebar.offsetWidth;
        document.body.classList.add('resizing-sidebar');
        sidebar.classList.add('resizing');
        resizer.classList.add('active');
    });

    document.addEventListener('mousemove', (e) => {
        if (!isResizing) return;
        e.preventDefault();
        const width = startWidth - (e.clientX - startX);
        if (width >= 280 && width <= 700) {
            sidebar.style.width = `${width}px`;
        }
    });

    document.addEventListener('mouseup', () => {
        if (isResizing) {
            isResizing = false;
            document.body.classList.remove('resizing-sidebar');
            sidebar.classList.remove('resizing');
            resizer.classList.remove('active');
            // Trigger graph resize after sidebar resize
            if (state.cy) {
                state.cy.resize();
                state.cy.fit(undefined, 50);
            }
        }
    });
}

// =====================================================
// Source Code Viewer
// =====================================================

let currentSourcePath = null;
let sourceExpanded = false;

/**
 * Load and display source code for a file
 */
async function loadSourceCode(filePath, line = null, context = 15) {
    const panel = document.getElementById('source-code-panel');
    if (!panel) return;

    // Toggle if same file
    if (currentSourcePath === filePath && panel.innerHTML !== '') {
        panel.innerHTML = '';
        currentSourcePath = null;
        return;
    }

    currentSourcePath = filePath;

    // Show loading state
    panel.innerHTML = '<div class="source-loading">Loading source code...</div>';

    try {
        const params = new URLSearchParams({ path: filePath });
        if (line) params.append('line', line.toString());
        params.append('context', context.toString());

        const response = await fetch(`/api/source?${params}`);
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}`);
        }

        const data = await response.json();
        renderSourceCode(panel, data);
    } catch (error) {
        panel.innerHTML = `<div class="source-error">Failed to load source: ${escapeHtml(error.message)}</div>`;
    }
}

/**
 * Render source code in the panel
 */
function renderSourceCode(panel, data) {
    const fileName = data.file_name || 'unknown';
    const lines = data.lines || [];

    panel.innerHTML = `
        <div class="source-code-container">
            <div class="source-code-header">
                <div class="file-info">
                    <span class="file-icon">🦀</span>
                    <span class="file-name">${escapeHtml(fileName)}</span>
                    <span class="line-info">Lines ${data.start_line}-${data.end_line} of ${data.total_lines}</span>
                </div>
                <div class="source-code-actions">
                    <button class="btn-expand-source" title="Expand/Collapse">⤢</button>
                    <button class="btn-close-source" title="Close">×</button>
                </div>
            </div>
            <div class="source-code-content ${sourceExpanded ? 'expanded' : ''}">
                ${lines.map(line => `
                    <div class="source-line ${line.is_highlight ? 'highlight' : ''}">
                        <span class="line-number">${line.number}</span>
                        <span class="line-content">${escapeHtml(line.content)}</span>
                    </div>
                `).join('')}
            </div>
            <div class="source-code-footer">
                <span class="path">${escapeHtml(data.file_path)}</span>
                <span class="total">${data.total_lines} lines total</span>
            </div>
        </div>
    `;

    // Setup button handlers
    panel.querySelector('.btn-close-source')?.addEventListener('click', () => {
        panel.innerHTML = '';
        currentSourcePath = null;
    });

    panel.querySelector('.btn-expand-source')?.addEventListener('click', () => {
        sourceExpanded = !sourceExpanded;
        const content = panel.querySelector('.source-code-content');
        if (content) {
            content.classList.toggle('expanded', sourceExpanded);
        }
    });
}

/**
 * Load source code for an edge (coupling location)
 */
export async function loadEdgeSourceCode(location) {
    if (!location || !location.file_path) return;

    const panel = document.getElementById('source-code-panel');
    if (!panel) return;

    await loadSourceCode(location.file_path, location.line || 1, 10);
}

// =====================================================
// Details Modal
// =====================================================

let currentModalData = null;

/**
 * Setup the details modal
 */
export function setupDetailsModal() {
    const modal = document.getElementById('details-modal');
    const closeBtn = document.getElementById('close-details-modal');
    const backdrop = modal?.querySelector('.details-modal-backdrop');
    const tabs = modal?.querySelectorAll('.details-tabs .tab');

    if (!modal) return;

    // Close button
    closeBtn?.addEventListener('click', hideDetailsModal);

    // Backdrop click
    backdrop?.addEventListener('click', hideDetailsModal);

    // Escape key
    document.addEventListener('keydown', (e) => {
        if (e.key === 'Escape' && !modal.classList.contains('hidden')) {
            hideDetailsModal();
        }
    });

    // Tab switching
    tabs?.forEach(tab => {
        tab.addEventListener('click', () => {
            const tabName = tab.dataset.tab;
            switchModalTab(tabName);
        });
    });
}

/**
 * Show the details modal for a module
 */
export function showDetailsModal(nodeData) {
    const modal = document.getElementById('details-modal');
    if (!modal) return;

    currentModalData = nodeData;

    // Find full node data
    const fullNode = state.graphData?.nodes?.find(n => n.id === nodeData.id);
    const data = { ...nodeData, ...fullNode };

    // Update title
    const title = document.getElementById('details-modal-title');
    if (title) {
        title.textContent = `📦 ${data.label || data.id}`;
    }

    // Populate tabs
    populateOverviewTab(data);
    populateCouplingsTab(data);
    populateItemsTab(data);
    populateSourceTab(data);

    // Show modal
    modal.classList.remove('hidden');
    switchModalTab('overview');
}

/**
 * Hide the details modal
 */
export function hideDetailsModal() {
    const modal = document.getElementById('details-modal');
    if (modal) {
        modal.classList.add('hidden');
    }
    currentModalData = null;
}

/**
 * Switch modal tab
 */
function switchModalTab(tabName) {
    const modal = document.getElementById('details-modal');
    if (!modal) return;

    // Update tab buttons
    modal.querySelectorAll('.details-tabs .tab').forEach(tab => {
        tab.classList.toggle('active', tab.dataset.tab === tabName);
    });

    // Update tab content
    modal.querySelectorAll('.tab-content').forEach(content => {
        content.classList.toggle('active', content.id === `tab-${tabName}`);
    });
}

/**
 * Populate Overview tab
 */
function populateOverviewTab(data) {
    const container = document.getElementById('tab-overview');
    if (!container) return;

    const metrics = data.metrics || {};
    const fnCount = metrics.fn_count || 0;
    const typeCount = metrics.type_count || 0;
    const implCount = metrics.impl_count || 0;
    const traitImplCount = metrics.trait_impl_count || 0;
    const inherentImplCount = metrics.inherent_impl_count || 0;
    const inCycle = data.in_cycle || false;
    const volatility = metrics.volatility || 'Medium';
    const health = data.health || 'good';
    const balanceScore = data.balance_score || 0;

    const getHealthClass = (h) => {
        if (h === 'critical') return 'critical';
        if (h === 'warning') return 'warning';
        return 'good';
    };

    container.innerHTML = `
        ${inCycle ? `
            <div class="warning-banner critical">
                ⚠️ This module is part of a circular dependency
            </div>
        ` : ''}
        <div class="modal-section">
            <div class="modal-section-title">Statistics</div>
            <div class="modal-stat-grid">
                <div class="modal-stat-card">
                    <div class="modal-stat-value">${fnCount}</div>
                    <div class="modal-stat-label">Functions</div>
                </div>
                <div class="modal-stat-card">
                    <div class="modal-stat-value">${typeCount}</div>
                    <div class="modal-stat-label">Types</div>
                </div>
                <div class="modal-stat-card">
                    <div class="modal-stat-value">${implCount}</div>
                    <div class="modal-stat-label">Implementations</div>
                </div>
                <div class="modal-stat-card">
                    <div class="modal-stat-value ${getHealthClass(health)}">${health}</div>
                    <div class="modal-stat-label">Health</div>
                </div>
                <div class="modal-stat-card">
                    <div class="modal-stat-value">${(balanceScore * 100).toFixed(0)}%</div>
                    <div class="modal-stat-label">Balance</div>
                </div>
                <div class="modal-stat-card">
                    <div class="modal-stat-value volatility-badge ${volatility.toLowerCase()}">${volatility}</div>
                    <div class="modal-stat-label">Volatility</div>
                </div>
            </div>
        </div>
        ${implCount > 0 ? `
            <div class="modal-section">
                <div class="modal-section-title">Implementation Breakdown</div>
                <div class="impl-breakdown">
                    <div class="impl-item">
                        <span class="label">Trait impl:</span>
                        <span class="count">${traitImplCount}</span>
                    </div>
                    <div class="impl-item">
                        <span class="label">Inherent impl:</span>
                        <span class="count">${inherentImplCount}</span>
                    </div>
                </div>
            </div>
        ` : ''}
        <div class="modal-section">
            <div class="modal-section-title">File Location</div>
            <div class="file-path-display">
                <span class="file-path-value">${escapeHtml(data.file_path || 'Unknown')}</span>
            </div>
        </div>
    `;
}

/**
 * Populate Couplings tab
 */
function populateCouplingsTab(data) {
    const container = document.getElementById('tab-couplings');
    if (!container || !state.cy) return;

    const node = state.cy.getElementById(data.id);
    if (!node || node.length === 0) {
        container.innerHTML = '<p class="placeholder">No coupling data available</p>';
        return;
    }

    const outgoing = node.outgoers('edge').map(e => ({
        target: e.data('target'),
        strength: e.data('strengthLabel') || 'Model',
        balance: e.data('balance') || 0
    }));

    const incoming = node.incomers('edge').map(e => ({
        source: e.data('source'),
        strength: e.data('strengthLabel') || 'Model',
        balance: e.data('balance') || 0
    }));

    container.innerHTML = `
        <div class="modal-section">
            <div class="modal-section-title">Outgoing Couplings (${outgoing.length})</div>
            ${outgoing.length > 0 ? `
                <div class="coupling-list">
                    ${outgoing.map(c => `
                        <div class="coupling-item outgoing" data-target="${c.target}">
                            <span class="coupling-target">→ ${escapeHtml(c.target)}</span>
                            <span class="coupling-meta">
                                <span class="strength-badge ${c.strength.toLowerCase()}">${c.strength}</span>
                                <span>${(c.balance * 100).toFixed(0)}%</span>
                            </span>
                        </div>
                    `).join('')}
                </div>
            ` : '<p class="placeholder">No outgoing couplings</p>'}
        </div>
        <div class="modal-section">
            <div class="modal-section-title">Incoming Couplings (${incoming.length})</div>
            ${incoming.length > 0 ? `
                <div class="coupling-list">
                    ${incoming.map(c => `
                        <div class="coupling-item incoming" data-source="${c.source}">
                            <span class="coupling-target">← ${escapeHtml(c.source)}</span>
                            <span class="coupling-meta">
                                <span class="strength-badge ${c.strength.toLowerCase()}">${c.strength}</span>
                                <span>${(c.balance * 100).toFixed(0)}%</span>
                            </span>
                        </div>
                    `).join('')}
                </div>
            ` : '<p class="placeholder">No incoming couplings</p>'}
        </div>
    `;
}

/**
 * Populate Items tab
 */
function populateItemsTab(data) {
    const container = document.getElementById('tab-items');
    if (!container) return;

    const items = data.items || [];
    const types = items.filter(i => i.kind === 'type' || i.kind === 'struct' || i.kind === 'enum');
    const traits = items.filter(i => i.kind === 'trait');
    const functions = items.filter(i => i.kind === 'fn');

    const renderItems = (itemList, kind, icon) => {
        if (itemList.length === 0) return '<p class="placeholder">None</p>';
        return `
            <div class="modal-item-list">
                ${itemList.map(item => `
                    <div class="modal-item" data-module="${data.id}" data-item="${item.name}">
                        <span class="modal-item-icon ${kind}">${icon}</span>
                        <span class="modal-item-name">${escapeHtml(item.name)}</span>
                        ${item.visibility === 'pub' || item.visibility === 'Public' ?
                            '<span class="modal-item-vis">pub</span>' : ''}
                    </div>
                `).join('')}
            </div>
        `;
    };

    container.innerHTML = `
        <div class="modal-section">
            <div class="modal-section-title">Types (${types.length})</div>
            ${renderItems(types, 'type', 'S')}
        </div>
        <div class="modal-section">
            <div class="modal-section-title">Traits (${traits.length})</div>
            ${renderItems(traits, 'trait', 'T')}
        </div>
        <div class="modal-section">
            <div class="modal-section-title">Functions (${functions.length})</div>
            ${renderItems(functions, 'fn', 'ƒ')}
        </div>
    `;

    // Setup click handlers
    container.querySelectorAll('.modal-item').forEach(item => {
        item.addEventListener('click', () => {
            const moduleId = item.dataset.module;
            const itemName = item.dataset.item;
            hideDetailsModal();
            focusOnItem(moduleId, itemName);
        });
    });
}

/**
 * Populate Source tab
 */
async function populateSourceTab(data) {
    const container = document.getElementById('tab-source');
    if (!container) return;

    const filePath = data.file_path;
    if (!filePath || filePath.startsWith('[external]')) {
        container.innerHTML = '<p class="placeholder">Source code not available for external modules</p>';
        return;
    }

    container.innerHTML = '<div class="source-loading">Loading source code...</div>';

    try {
        const response = await fetch(`/api/source?path=${encodeURIComponent(filePath)}&context=50`);
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}`);
        }

        const sourceData = await response.json();
        const lines = sourceData.lines || [];

        container.innerHTML = `
            <div class="modal-source-code">
                <div class="modal-source-header">
                    <span class="modal-source-path">${escapeHtml(sourceData.file_path)}</span>
                    <span>${sourceData.total_lines} lines</span>
                </div>
                <div class="modal-source-content source-code-content">
                    ${lines.map(line => `
                        <div class="source-line">
                            <span class="line-number">${line.number}</span>
                            <span class="line-content">${escapeHtml(line.content)}</span>
                        </div>
                    `).join('')}
                </div>
            </div>
        `;
    } catch (error) {
        container.innerHTML = `<div class="source-error">Failed to load source: ${escapeHtml(error.message)}</div>`;
    }
}

// =====================================================
// Utilities
// =====================================================

function getCheckedValues(containerId) {
    return Array.from(document.querySelectorAll(`#${containerId} input:checked`)).map(cb => cb.value);
}
