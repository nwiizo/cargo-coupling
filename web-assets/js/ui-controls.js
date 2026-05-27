// =====================================================
// UI Components Module
// =====================================================

import { state, setSelectedNode, setSelectedEdge, setCenterMode } from './state.js';
import { t, tf } from './i18n.js';
import { applyLayout, clearHighlights, centerOnNode, focusOnNode, highlightNeighbors, highlightDependencyPath, analyzeCoupling, getHealthColor } from './coupling-graph-2d.js';
import { refresh3dGraph, focusLink3d, focusNode3d } from './coupling-graph-3d.js';
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

    const counts = summary.issues_by_severity || {};
    const rationale = buildHealthRationale(summary, graphData);
    container.innerHTML = `
        <div class="health-header-card">
            <div class="health-grade-block">
                <span class="health-label">${t('health_grade')}</span>
                <span class="health-grade ${summary.health_grade}">${summary.health_grade}</span>
            </div>
            <div class="health-rationale">
                <strong>${(summary.health_score * 100).toFixed(1)}%</strong>
                <span>${escapeHtml(rationale)}</span>
                ${state.activeRevision ? `<span class="revision-chip">${escapeHtml(state.activeRevision)}</span>` : ''}
            </div>
            <div class="health-counts" aria-label="Issue counts">
                <button class="health-count critical" type="button" data-severity="Critical">
                    <span>${t('severity_critical')}</span><strong>${counts.critical || 0}</strong>
                </button>
                <button class="health-count high" type="button" data-severity="High">
                    <span>${t('severity_high')}</span><strong>${counts.high || 0}</strong>
                </button>
                <button class="health-count medium" type="button" data-severity="Medium">
                    <span>${t('severity_medium')}</span><strong>${counts.medium || 0}</strong>
                </button>
            </div>
            <div class="health-meta">
                <span>${summary.total_modules} ${t('modules')}</span>
                <span>${summary.total_couplings} ${t('couplings')}</span>
                <span>${totalFunctions}fn ${totalTypes}ty ${totalImpls}impl</span>
            </div>
        </div>
    `;
}

function buildHealthRationale(summary, graphData) {
    const counts = summary.issues_by_severity || {};
    const issues = graphData?.issues || [];
    const criticalHigh = issues.filter(issue => ['Critical', 'High'].includes(issue.severity));
    if ((counts.critical || 0) > 0) {
        const top = mostCommon(criticalHigh.map(issue => formatIssueType(issue.type || issue.issue_type)));
        return top ? tf('rationale_critical_top', { issue: top }) : t('rationale_critical');
    }
    if ((counts.high || 0) > 0) {
        const top = mostCommon(criticalHigh.map(issue => formatIssueType(issue.type || issue.issue_type)));
        return top ? tf('rationale_high_top', { issue: top }) : t('rationale_high');
    }
    if ((counts.medium || 0) > 0) {
        return t('rationale_medium');
    }
    if (summary.health_grade === 'S') {
        return t('rationale_s_grade');
    }
    return t('rationale_none');
}

function mostCommon(values) {
    const counts = new Map();
    for (const value of values.filter(Boolean)) {
        counts.set(value, (counts.get(value) || 0) + 1);
    }
    return [...counts.entries()].sort((a, b) => b[1] - a[1])[0]?.[0] || null;
}

export function setupHealthHeaderInteractions() {
    const container = document.getElementById('header-stats');
    if (!container || container.dataset.bound === 'true') return;
    container.dataset.bound = 'true';
    container.addEventListener('click', (event) => {
        const button = event.target.closest('.health-count[data-severity]');
        if (!button) return;
        focusIssuesBySeverity(button.dataset.severity);
    });
}

export function updateFooterStats(summary) {
    const container = document.getElementById('summary-stats');
    if (!container || !summary) return;

    const issueCount = Object.values(summary.issues_by_severity || {}).reduce((a, b) => a + b, 0);
    container.innerHTML = `
        <span>${t('issues')}: ${issueCount}</span>
        <span>${t('health_score')}: ${(summary.health_score * 100).toFixed(1)}%</span>
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
    const showHiddenCoupling = document.getElementById('show-hidden-coupling')?.checked ?? true;

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
        const hiddenCoupling = edge.data('hiddenCoupling');

        // Skip filtering for parent/item edges
        if (edgeType === 'parent' || edgeType === 'item-dep') {
            edge.style('display', 'element');
            return;
        }

        if (hiddenCoupling && !showHiddenCoupling) {
            edge.style('display', 'none');
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
    document.getElementById('hide-external')?.addEventListener('change', () => {
        applyFilters();
        refresh3dGraph();
    });
    document.getElementById('show-hidden-coupling')?.addEventListener('change', () => {
        applyFilters();
        refresh3dGraph();
    });

    document.getElementById('reset-filters')?.addEventListener('click', () => {
        document.querySelectorAll('#strength-filters input, #volatility-filters input').forEach(cb => cb.checked = true);
        document.querySelectorAll('#distance-filters input').forEach(cb => {
            cb.checked = cb.value !== 'DifferentCrate';
        });
        document.getElementById('balance-min').value = 0;
        document.getElementById('balance-max').value = 100;
        document.getElementById('show-issues-only').checked = false;
        document.getElementById('show-cycles-only').checked = false;
        document.getElementById('show-hidden-coupling').checked = true;
        document.getElementById('hide-external').checked = true; // Default: hide external

        state.cy.elements().removeClass('hidden highlighted dimmed dependency-source dependency-target search-match');
        applyFilters();
        refresh3dGraph();
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
    const toggle = document.getElementById('legend-toggle') || document.getElementById('toggle-legend');
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
// Trust Panel
// =====================================================

export function populateTrustPanel() {
    const count = document.getElementById('blind-spot-count');
    const content = document.getElementById('trust-panel-content');
    if (!content || !state.graphData) return;

    const manifest = state.graphData.not_analyzed || {};
    const blindSpots = manifest.blind_spots || [];
    const notes = state.currentLang === 'ja' ? (manifest.notes_ja || manifest.notes || []) : (manifest.notes || []);

    if (count) count.textContent = blindSpots.length + notes.length;

    content.innerHTML = `
        ${notes.length > 0 ? `
            <div class="trust-notes">
                ${notes.map(note => `<div class="trust-note">${escapeHtml(note)}</div>`).join('')}
            </div>
        ` : ''}
        <div class="blind-spot-list">
            ${blindSpots.map(spot => `
                    <div class="blind-spot-item">
                        <div class="blind-spot-area">${escapeHtml(spot.area)}</div>
                    <div class="blind-spot-description">${escapeHtml(state.currentLang === 'ja' ? (spot.description_ja || spot.description) : spot.description)}</div>
                </div>
            `).join('')}
        </div>
    `;
}

export function setupTrustPanelToggle() {
    const toggle = document.getElementById('toggle-trust-panel');
    const content = document.getElementById('trust-panel-content');
    if (!toggle || !content) return;

    toggle.addEventListener('click', () => {
        const hidden = content.style.display === 'none';
        content.style.display = hidden ? 'block' : 'none';
        toggle.textContent = hidden ? t('hide_blind_spots') : t('show_blind_spots');
    });
}

// =====================================================
// Node/Edge Details
// =====================================================

export function showNodeDetails(data) {
    setSelectedEdge(null);
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
    const volatility = volatilityLabel(data.volatility ?? metrics.volatility ?? 'Medium');
    const subdomain = data.subdomain || fullNode?.subdomain;
    const expectedVolatility = data.expected_volatility || fullNode?.expected_volatility;
    const flags = data.flags || fullNode?.flags || [];
    const relatedIssues = (state.graphData?.issues || []).filter(issue =>
        issue.source === data.id || issue.target === data.id || (issue.focus?.node_ids || []).includes(data.id)
    );

    // Group items by kind
    const types = items.filter(i => i.kind === 'type' || i.kind === 'struct' || i.kind === 'enum');
    const traits = items.filter(i => i.kind === 'trait');
    const functions = items.filter(i => i.kind === 'fn');

    const renderItemList = (itemList, kind, icon) => {
        if (itemList.length === 0) return '';
        const isPub = (vis) => vis === 'pub' || vis === 'Public';
        const kindLabel = itemKindLabel(kind);
        return `
            <div class="item-list-section" data-kind="${kind}">
                <div class="item-list-header">
                    <span class="item-list-icon">${icon}</span>
                    <span class="item-list-title">${kindLabel}</span>
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
                    ${itemList.length > 10 ? `<div class="item-list-more">+${itemList.length - 10} ${t('more')}</div>` : ''}
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
        <div class="inspector-title">${escapeHtml(data.label)}</div>
        ${inCycle ? `
            <div class="warning-banner critical">
                ${t('module_in_cycle')}
            </div>
        ` : ''}
        <details class="inspector-section" open>
            <summary>${t('module_health')}</summary>
            <div class="detail-stats">
                <span class="stat-badge fn">${fnCount} fn</span>
                <span class="stat-badge type">${typeCount} type</span>
                <span class="stat-badge impl">${implCount} impl</span>
            </div>
            <div class="detail-row">
                <span class="detail-label">${t('health')}:</span>
                <span class="health-indicator ${data.health || 'good'}">${data.health || 'good'}</span>
            </div>
            <div class="detail-row">
                <span class="detail-label">${t('balance_score')}:</span>
                <span>${((data.balance_score || 0) * 100).toFixed(0)}%</span>
            </div>
            <div class="detail-row">
                <span class="detail-label">${t('volatility')}:</span>
                <span class="volatility-badge ${getVolatilityClass(volatility)}">${labelValue(volatility)}</span>
            </div>
        </details>
        <details class="inspector-section" open>
            <summary>${t('domain_dimensions')}</summary>
        ${subdomain ? `
        <div class="detail-row">
            <span class="detail-label">${t('subdomain')}:</span>
            <span class="subdomain-badge ${subdomain.toLowerCase()}">${escapeHtml(labelSubdomain(subdomain))}</span>
        </div>
        <div class="detail-row">
            <span class="detail-label">${t('expected_volatility')}:</span>
            <span>${escapeHtml(labelValue(expectedVolatility || 'Unknown'))}</span>
        </div>
        ` : ''}
        ${flags.includes('AccidentalVolatility') ? `
            <div class="warning-banner medium">${t('accidental_volatility_warning')}</div>
        ` : ''}
        <div class="detail-row">
            <span class="detail-label">${t('outgoing')}:</span>
            <span>${data.couplings_out || 0}</span>
        </div>
        <div class="detail-row">
            <span class="detail-label">${t('incoming')}:</span>
            <span>${data.couplings_in || 0}</span>
        </div>
        ${implCount > 0 ? `
            <div class="impl-breakdown">
                <div class="impl-item">
                    <span class="label">${t('trait_impl')}:</span>
                    <span class="count">${traitImplCount}</span>
                </div>
                <div class="impl-item">
                    <span class="label">${t('inherent_impl')}:</span>
                    <span class="count">${inherentImplCount}</span>
                </div>
            </div>
        ` : ''}
        </details>
        ${relatedIssues.length > 0 ? `
            <details class="inspector-section" open>
                <summary>${t('issues_affecting_module')}</summary>
                <div class="overview-issue-list">
                    ${relatedIssues.slice(0, 8).map(issue => `
                        <button type="button" class="overview-issue severity-${issue.severity?.toLowerCase()}" data-issue-id="${issue.id}">
                            <span>${escapeHtml(labelSeverity(issue.severity))}</span>
                            <strong>${escapeHtml(formatIssueType(issue.type || issue.issue_type))}</strong>
                            <small>${escapeHtml(issue.source)}${issue.target ? ` -> ${escapeHtml(issue.target)}` : ''}</small>
                        </button>
                    `).join('')}
                </div>
            </details>
        ` : ''}
        <details class="inspector-section" open>
            <summary>${t('source')}</summary>
        ${filePath && !isExternal ? `
            <div class="file-path-display">
                <span class="file-path-label">${t('file')}:</span>
                <span class="file-path-value" title="${escapeHtml(filePath)}">${escapeHtml(filePath)}</span>
            </div>
            <button class="btn-view-code" data-path="${escapeHtml(filePath)}">
                <span class="icon">📄</span> ${t('view_source_code')}
            </button>
        ` : `<div class="no-data">${t('no_local_source')}</div>`}
            <div id="source-code-panel"></div>
        </details>
        ${items.length > 0 ? `
            <details class="inspector-section">
            <summary>${t('module_contents')}</summary>
            <div class="module-items-section">
                <div class="module-items-header">
                    <span>${t('module_contents')}</span>
                    <span class="hint">${t('click_to_focus')}</span>
                </div>
                ${renderItemList(types, 'types', 'S')}
                ${renderItemList(traits, 'traits', 'T')}
                ${renderItemList(functions, 'functions', 'ƒ')}
            </div>
            </details>
        ` : ''}
        <button class="btn-expand-details" data-module-id="${data.id}">
            <span>⤢</span> ${t('full_details')}
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

    container.querySelectorAll('[data-issue-id]').forEach(button => {
        button.addEventListener('click', () => {
            const issue = relatedIssues.find(item => item.id === button.dataset.issueId);
            focusIssueInGraph(issue);
        });
    });

    if (filePath && !isExternal) {
        loadSourceCode(filePath, null, 20);
    }
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
    setSelectedNode(null);
    setSelectedEdge(data);
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
                ${t('edge_in_cycle')}
            </div>
        ` : ''}
        <div class="inspector-title">${t('coupling_details')}</div>
        <details class="inspector-section" open>
            <summary>${t('relationship')}</summary>
        <div class="detail-row">
            <span class="detail-label">${t('source')}:</span>
            <span>${escapeHtml(data.source)}</span>
        </div>
        <div class="detail-row">
            <span class="detail-label">${t('target')}:</span>
            <span>${escapeHtml(data.target)}</span>
        </div>
        ${data.hiddenCoupling || data.edgeType === 'hidden-coupling' ? `
        <div class="detail-row">
            <span class="detail-label">${t('co_change')}:</span>
            <span>${data.coChangeCount || 0} commits, ${Math.round((data.couplingRatio || 0) * 100)}%</span>
        </div>
        ` : ''}
        </details>
        <details class="inspector-section" open>
            <summary>${t('balanced_dimensions')}</summary>
        <div class="detail-row">
            <span class="detail-label">${t('strength')}:</span>
            <span class="strength-badge ${(data.strengthLabel || '').toLowerCase()}">${labelValue(data.strengthLabel || 'Model')}</span>
        </div>
        <div class="detail-row">
            <span class="detail-label">${t('distance')}:</span>
            <span>${labelValue(data.distance || 'DifferentModule')}</span>
        </div>
        <div class="detail-row">
            <span class="detail-label">${t('volatility')}:</span>
            <span class="volatility-badge ${effectiveVol.toLowerCase()}">${labelValue(effectiveVol)}</span>
        </div>
        <div class="detail-row">
            <span class="detail-label">${t('balance')}:</span>
            <span>${((data.balance || 0) * 100).toFixed(0)}%</span>
        </div>
        <div class="detail-row">
            <span class="detail-label">${t('classification')}:</span>
            <span>${escapeHtml(state.currentLang === 'ja' ? (data.classificationJa || data.classification || '-') : (data.classification || '-'))}</span>
        </div>
        ${connascence ? `
            <div class="connascence-info">
                <span class="connascence-type">${escapeHtml(connascence.type || connascence.connascence_type || 'Identity')}</span>
                <span class="connascence-strength">${connascence.strength || 'Weak'}</span>
            </div>
        ` : `<div class="connascence-info"><span class="connascence-type">${t('connascence_not_detected')}</span></div>`}
        </details>
        <details class="inspector-section" open>
            <summary>${t('assessment')}</summary>
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
                <div class="issue-type">${escapeHtml(formatIssueType(issue.type || issue.issue_type))}</div>
                <div class="issue-description">${escapeHtml(issue.description || issue.message || '')}</div>
            </div>
        ` : ''}
        </details>
        <details class="inspector-section" open>
            <summary>${t('source_location')}</summary>
        ${hasLocation ? `
            <button class="btn-view-code" data-path="${escapeHtml(location.file_path)}" data-line="${location.line || 0}">
                <span class="icon">📄</span> ${t('view_coupling_location')}
            </button>
        ` : `<div class="no-data">${t('no_source_location')}</div>`}
            <div id="source-code-panel"></div>
        </details>
    `;

    // Setup view code button for edge
    if (hasLocation) {
        const btn = container.querySelector('.btn-view-code');
        btn?.addEventListener('click', () => {
            loadSourceCode(location.file_path, location.line || null, 10);
        });
        loadSourceCode(location.file_path, location.line || null, 10);
    }
}

export function clearDetails() {
    setSelectedEdge(null);
    const container = document.getElementById('node-details');
    if (container) {
        renderProjectOverview(container);
    }
}

export function renderProjectOverview(container = document.getElementById('node-details')) {
    if (!container || !state.graphData) return;

    const data = state.graphData;
    const summary = data.summary || {};
    const counts = summary.issues_by_severity || {};
    const topIssues = [...(data.issues || [])]
        .sort((a, b) => getSeverityOrder(a.severity) - getSeverityOrder(b.severity))
        .slice(0, 5);
    const subdomains = countBy((data.nodes || []), node => node.subdomain || 'Unclassified');

    container.innerHTML = `
        <div class="inspector-title">
            <span>${t('project_overview')}</span>
            ${state.activeRevision ? `<span class="revision-chip">${escapeHtml(state.activeRevision)}</span>` : ''}
        </div>
        <details class="inspector-section" open>
            <summary>${t('health_summary')}</summary>
            <div class="overview-health-row">
                <span class="health-grade ${summary.health_grade || 'B'}">${summary.health_grade || '-'}</span>
                <div>
                    <div class="overview-score">${summary.health_score != null ? (summary.health_score * 100).toFixed(1) : '-'}%</div>
                    <div class="overview-rationale">${escapeHtml(buildHealthRationale(summary, data))}</div>
                </div>
            </div>
            <div class="overview-count-grid">
                <button type="button" data-severity="Critical" class="overview-count critical">${counts.critical || 0}<span>${t('severity_critical')}</span></button>
                <button type="button" data-severity="High" class="overview-count high">${counts.high || 0}<span>${t('severity_high')}</span></button>
                <button type="button" data-severity="Medium" class="overview-count medium">${counts.medium || 0}<span>${t('severity_medium')}</span></button>
            </div>
        </details>
        <details class="inspector-section" open>
            <summary>${t('subdomains')}</summary>
            <div class="subdomain-legend-list">
                ${Object.entries(subdomains).map(([name, count]) => `
                    <div class="subdomain-row">
                        <span class="subdomain-swatch ${name.toLowerCase()}"></span>
                        <span>${escapeHtml(labelSubdomain(name))}</span>
                        <strong>${count}</strong>
                    </div>
                `).join('')}
            </div>
        </details>
        <details class="inspector-section" open>
            <summary>${t('top_issues')}</summary>
            ${topIssues.length > 0 ? `
                <div class="overview-issue-list">
                    ${topIssues.map(issue => `
                        <button type="button" class="overview-issue severity-${issue.severity?.toLowerCase()}" data-issue-id="${issue.id}">
                            <span>${escapeHtml(labelSeverity(issue.severity))}</span>
                            <strong>${escapeHtml(formatIssueType(issue.type || issue.issue_type))}</strong>
                            <small>${escapeHtml(issue.source)}${issue.target ? ` -> ${escapeHtml(issue.target)}` : ''}</small>
                        </button>
                    `).join('')}
                </div>
            ` : `<div class="no-data">${t('no_issues_detected')}</div>`}
        </details>
    `;

    container.querySelectorAll('[data-severity]').forEach(button => {
        button.addEventListener('click', () => focusIssuesBySeverity(button.dataset.severity));
    });
    container.querySelectorAll('[data-issue-id]').forEach(button => {
        button.addEventListener('click', () => {
            const issue = (state.graphData.issues || []).find(item => item.id === button.dataset.issueId);
            focusIssueInGraph(issue);
        });
    });
}

export function focusIssuesBySeverity(severity) {
    if (!state.graphData || !state.cy) return;
    const issues = (state.graphData.issues || []).filter(issue => issue.severity === severity);
    if (issues.length === 0) {
        renderProjectOverview();
        return;
    }

    clearHighlights();
    state.cy.elements().addClass('dimmed');

    const focused = collectIssueElements(issues);
    focused.nodes.forEach(node => node.removeClass('dimmed').addClass('highlighted'));
    focused.edges.forEach(edge => edge.removeClass('dimmed').addClass('highlighted'));

    const eles = [...focused.nodes, ...focused.edges].reduce((acc, ele) => acc ? acc.union(ele) : ele, null);
    if (eles?.length) {
        state.cy.fit(eles, 80);
    }

    const first = issues[0];
    if (first?.focus?.edge_id) {
        focusLink3d(first.focus.edge_id, first.focus.node_ids || []);
    } else if (first?.focus?.node_ids?.[0]) {
        focusNode3d(first.focus.node_ids[0]);
    }

    const container = document.getElementById('node-details');
    if (container) {
        container.innerHTML = `
            <div class="inspector-title">${escapeHtml(tf('issue_focus', { severity: labelSeverity(severity) }))}</div>
            <div class="issue-focus-summary">
                <p>${escapeHtml(tf('issue_focus_summary', { count: issues.length, severity: labelSeverity(severity) }))}</p>
            </div>
            <div class="overview-issue-list">
                ${issues.map(issue => `
                    <button type="button" class="overview-issue severity-${issue.severity?.toLowerCase()}" data-issue-id="${issue.id}">
                        <span>${escapeHtml(labelSeverity(issue.severity))}</span>
                        <strong>${escapeHtml(formatIssueType(issue.type || issue.issue_type))}</strong>
                        <small>${escapeHtml(issue.source)}${issue.target ? ` -> ${escapeHtml(issue.target)}` : ''}</small>
                    </button>
                `).join('')}
            </div>
        `;
        container.querySelectorAll('[data-issue-id]').forEach(button => {
            button.addEventListener('click', () => {
                const issue = issues.find(item => item.id === button.dataset.issueId);
                focusIssueInGraph(issue);
            });
        });
    }
}

function focusIssueInGraph(issue) {
    if (!issue || !state.cy) return;
    const focused = collectIssueElements([issue]);
    clearHighlights();
    state.cy.elements().addClass('dimmed');
    focused.nodes.forEach(node => node.removeClass('dimmed').addClass('highlighted'));
    focused.edges.forEach(edge => edge.removeClass('dimmed').addClass('highlighted'));

    const firstEdge = focused.edges[0];
    if (firstEdge) {
        highlightDependencyPath(firstEdge);
        showEdgeDetails(firstEdge.data());
        return;
    }

    const firstNode = focused.nodes[0];
    if (firstNode) {
        focusOnNode(firstNode);
        showNodeDetails(firstNode.data());
    }
}

function collectIssueElements(issues) {
    const nodes = [];
    const edges = [];
    for (const issue of issues) {
        const focus = issue.focus || {};
        if (focus.edge_id) {
            let edge = state.cy.getElementById(focus.edge_id);
            if (!edge.length && focus.node_ids?.length >= 2) {
                edge = state.cy.edges().filter(e =>
                    e.data('source') === focus.node_ids[0] && e.data('target') === focus.node_ids[1]
                );
            }
            edge.forEach(e => edges.push(e));
        }
        (focus.node_ids || []).forEach(id => {
            const node = state.cy.getElementById(id);
            if (node.length) nodes.push(node);
        });
    }
    return { nodes: uniqueElements(nodes), edges: uniqueElements(edges) };
}

function uniqueElements(elements) {
    const seen = new Set();
    return elements.filter(element => {
        const id = element.id();
        if (seen.has(id)) return false;
        seen.add(id);
        return true;
    });
}

function countBy(items, keyFn) {
    return items.reduce((acc, item) => {
        const key = keyFn(item);
        acc[key] = (acc[key] || 0) + 1;
        return acc;
    }, {});
}

function getSeverityOrder(severity) {
    const order = { critical: 0, high: 1, medium: 2, low: 3 };
    return order[severity?.toLowerCase()] ?? 4;
}

function formatIssueType(type) {
    if (!type) return t('issue');
    const normalized = type
        .replace(/([A-Z])/g, '_$1')
        .replace(/[-\s]+/g, '_')
        .replace(/^_/, '')
        .toLowerCase();
    return t(`issue_${normalized}`) || type.replace(/([A-Z])/g, ' $1').trim() || t('issue');
}

function volatilityLabel(value) {
    if (typeof value === 'number') {
        if (value >= 0.75) return 'High';
        if (value >= 0.25) return 'Medium';
        return 'Low';
    }
    return value || 'Medium';
}

function itemKindLabel(kind) {
    return {
        types: t('types'),
        traits: t('traits'),
        functions: t('functions')
    }[kind] || kind;
}

function labelSeverity(severity) {
    return {
        Critical: t('severity_critical'),
        High: t('severity_high'),
        Medium: t('severity_medium'),
        Low: t('severity_low')
    }[severity] || severity || t('unknown');
}

function labelSubdomain(subdomain) {
    return {
        Core: t('subdomain_core'),
        Supporting: t('subdomain_supporting'),
        Generic: t('subdomain_generic'),
        Unclassified: t('subdomain_unclassified')
    }[subdomain] || subdomain || t('subdomain_not_configured');
}

function labelValue(value) {
    return {
        Intrusive: t('legend_intrusive'),
        Functional: t('legend_functional'),
        Model: t('legend_model'),
        Contract: t('legend_contract'),
        Low: t('severity_low'),
        Medium: t('severity_medium'),
        High: t('severity_high'),
        SameFunction: t('distance_same_function'),
        SameModule: t('distance_same_module'),
        DifferentModule: t('distance_different_module'),
        DifferentCrate: t('distance_different_crate'),
        Unknown: t('unknown')
    }[value] || value;
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
    panel.innerHTML = `<div class="source-loading">${t('loading_source')}</div>`;

    try {
        const params = new URLSearchParams({ path: filePath });
        if (line) params.append('line', line.toString());
        params.append('context', context.toString());
        if (state.activeRevision) params.append('ref', state.activeRevision);

        const response = await fetch(`/api/source?${params}`);
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}`);
        }

        const data = await response.json();
        renderSourceCode(panel, data);
    } catch (error) {
        panel.innerHTML = `<div class="source-error">${t('failed_to_load_source')}: ${escapeHtml(error.message)}</div>`;
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
                    <span class="line-info">${t('lines')} ${data.start_line}-${data.end_line} ${t('of')} ${data.total_lines}</span>
                </div>
                <div class="source-code-actions">
                    <button class="btn-expand-source" title="${t('expand_collapse')}">⤢</button>
                    <button class="btn-close-source" title="${t('close')}">×</button>
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
                <span class="total">${data.total_lines} ${t('lines_total')}</span>
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
