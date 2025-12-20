// =====================================================
// cargo-coupling Web Visualization - Main Entry Point
// =====================================================

import { CONFIG, state, setGraphData, setSelectedNode, setShowItems, setCy } from './state.js';
import { setupLanguageToggle, updateUILanguage } from './i18n.js';
import { initCytoscape, buildElements, getCytoscapeStyle, getLayoutConfig, applyLayout, centerOnNode, focusOnNode, highlightNeighbors, highlightDependencyPath, clearHighlights } from './graph.js';
import {
    updateHeaderStats,
    updateFooterStats,
    setupFilters,
    applyFilters,
    setupSearch,
    setupLayoutSelector,
    setupExportButtons,
    setupKeyboardShortcuts,
    setupCenterModeToggle,
    setupLegendToggle,
    setupResizableSidebar,
    showNodeDetails,
    showEdgeDetails,
    clearDetails,
    showBlastRadius,
    clearBlastRadius,
    setupDetailsModal
} from './ui.js';
import { showItemGraph, hideItemGraph } from './item-graph.js';
import {
    populateCriticalIssues,
    populateHotspots,
    populateModuleRankings,
    setupModuleRankingSorting,
    populateIssueList,
    setupAnalysisButtons,
    enableAnalysisButtons,
    setupClusterView,
    populatePathFinderSelects,
    setupJobButtons,
    updateWhatBreaksButton,
    setSelectNodeCallback
} from './features.js';
import { graphQueue, runLayoutAsync, debounce } from './graph-queue.js';
import { getInitialSelection, updateUrl, initUrlRouter } from './url-router.js';

// =====================================================
// State Management
// =====================================================

let currentView = 'graph';
let isInitialized = false;
let pendingRebuild = null;
let isNavigatingFromUrl = false;

// =====================================================
// Initialization
// =====================================================

async function init() {
    if (isInitialized) {
        console.warn('Already initialized');
        return;
    }

    try {
        await loadConfig();
        const data = await fetchGraphData();
        setGraphData(data);

        initGraph(data);
        initUI(data);
        initJobFeatures();

        // Initialize URL router for browser navigation
        initUrlRouter(handleUrlNavigation);

        // Show sidebar on load
        const sidebar = document.getElementById('sidebar');
        if (sidebar) {
            sidebar.classList.add('visible');
        }

        isInitialized = true;

        // Handle initial URL selection after everything is ready
        handleInitialUrlSelection();

    } catch (error) {
        console.error('Failed to initialize:', error);
        document.getElementById('cy').innerHTML =
            `<div style="padding: 2rem; color: #ef4444;">Failed to load graph data: ${error.message}</div>`;
    }
}

/**
 * Handle initial URL selection after graph is loaded
 */
function handleInitialUrlSelection() {
    const selection = getInitialSelection();

    // Handle view switch
    if (selection.view === 'tree') {
        const treeBtn = document.getElementById('view-tree');
        treeBtn?.click();
    }

    // Handle module/item selection
    if (selection.module) {
        isNavigatingFromUrl = true;

        // Enable show items if needed
        if (selection.item || selection.items) {
            const showItemsToggle = document.getElementById('show-items-toggle');
            if (showItemsToggle && !showItemsToggle.checked) {
                showItemsToggle.checked = true;
                setShowItems(true);
                rebuildGraph();
            }
        }

        // Wait for graph to settle, then select
        setTimeout(() => {
            const node = state.cy?.getElementById(selection.module);
            if (node?.length) {
                selectNode(node);

                // If item is specified, focus on it after selection
                if (selection.item) {
                    setTimeout(() => {
                        focusOnItemInternal(selection.module, selection.item);
                        isNavigatingFromUrl = false;
                    }, 300);
                } else {
                    isNavigatingFromUrl = false;
                }
            } else {
                isNavigatingFromUrl = false;
            }
        }, 500);
    }
}

/**
 * Handle URL navigation (back/forward button)
 */
function handleUrlNavigation(moduleId, itemName) {
    if (!state.cy) return;

    isNavigatingFromUrl = true;

    if (moduleId) {
        const node = state.cy.getElementById(moduleId);
        if (node?.length) {
            selectNode(node);
            if (itemName) {
                setTimeout(() => {
                    focusOnItemInternal(moduleId, itemName);
                    isNavigatingFromUrl = false;
                }, 300);
            } else {
                isNavigatingFromUrl = false;
            }
        } else {
            isNavigatingFromUrl = false;
        }
    } else {
        clearSelection();
        isNavigatingFromUrl = false;
    }
}

function initGraph(data, options = {}) {
    initCytoscape(
        data,
        // Node tap handler
        (node) => selectNode(node),
        // Edge tap handler
        (edge) => {
            highlightDependencyPath(edge);
            showEdgeDetails(edge.data());
        },
        // Background tap handler
        () => clearSelection(),
        // Options
        options
    );
}

/**
 * Rebuild graph with new options (e.g., toggle items)
 * Uses operation queue to prevent race conditions
 */
const rebuildGraph = debounce(function() {
    if (!state.graphData || !state.cy) return;

    graphQueue.enqueue('rebuild', async () => {
        const elements = buildElements(state.graphData, { showItems: state.showItems });

        // Remove existing elements and add new ones
        state.cy.elements().remove();
        state.cy.add(elements);

        // Run layout and wait for completion
        await runLayoutAsync(state.cy, getLayoutConfig(state.currentLayout));

        // Apply filters after layout is complete
        applyFilters();
    }, { cancelPending: true });
}, 100);

/**
 * Setup item view toggle
 */
function setupItemToggle() {
    const toggle = document.getElementById('show-items-toggle');
    if (toggle) {
        toggle.addEventListener('change', (e) => {
            setShowItems(e.target.checked);
            rebuildGraph();
        });
    }
}

function initUI(data) {
    setupLanguageToggle(() => {
        // Re-populate dynamic content on language change
        populateCriticalIssues();
    });

    updateHeaderStats(data.summary, data);
    updateFooterStats(data.summary);

    setupFilters();
    setupSearch();
    setupLayoutSelector();
    setupExportButtons();
    setupAnalysisButtons();
    setupClusterView();
    populateIssueList();
    setupResizableSidebar();
    setupDetailsModal();

    setupKeyboardShortcuts({
        onEscape: () => clearSelection()
    });

    setupAutoHideTriggers();
    setupViewToggle();
    setupLegendToggle();
    setupCenterModeToggle();
    setupItemToggle();

    // Set up callback for feature module
    setSelectNodeCallback((node) => selectNode(node));
}

function initJobFeatures() {
    populateCriticalIssues();
    populateHotspots();
    populateModuleRankings();
    setupModuleRankingSorting();
    setupJobButtons();
    populatePathFinderSelects();
}

// =====================================================
// Data Loading
// =====================================================

async function loadConfig() {
    try {
        const response = await fetch(CONFIG.configPath);
        if (response.ok) {
            const serverConfig = await response.json();
            if (serverConfig.api_endpoint) {
                CONFIG.apiEndpoint = serverConfig.api_endpoint;
            }
        }
    } catch (e) {
        console.log('Using default config');
    }
}

async function fetchGraphData() {
    const url = CONFIG.apiEndpoint + CONFIG.graphPath;
    const response = await fetch(url);
    if (!response.ok) {
        throw new Error(`HTTP ${response.status} from ${url}`);
    }
    return response.json();
}

// =====================================================
// Node Selection
// =====================================================

function selectNode(node) {
    // Use queue to prevent selection conflicts during animations
    graphQueue.enqueue('select', async () => {
        setSelectedNode(node);

        if (state.centerMode) {
            await new Promise(resolve => {
                centerOnNode(node, true);
                // Wait for animation
                setTimeout(resolve, 600);
            });
        } else {
            focusOnNode(node);
        }

        highlightNeighbors(node);
        showNodeDetails(node.data());
        enableAnalysisButtons(true);
        showBlastRadius(node);
        updateWhatBreaksButton();
        showItemGraph(node.id());

        // Update URL (skip if navigating from URL to avoid loops)
        if (!isNavigatingFromUrl) {
            updateUrl(node.id());
        }
    }, { cancelPending: true });
}

function clearSelection() {
    graphQueue.enqueue('clearSelection', async () => {
        setSelectedNode(null);

        if (state.cy) {
            state.cy.elements().removeClass('hidden highlighted dimmed dependency-source dependency-target search-match');
            state.cy.fit(undefined, 50);
        }

        clearDetails();
        enableAnalysisButtons(false);
        hideItemGraph();
        clearBlastRadius();
        updateWhatBreaksButton();

        // Clear URL (skip if navigating from URL)
        if (!isNavigatingFromUrl) {
            updateUrl(null);
        }

        const jobResult = document.getElementById('job-result');
        if (jobResult) jobResult.innerHTML = '';
    }, { cancelPending: true });
}

/**
 * Focus on an item internally (used by URL navigation)
 */
function focusOnItemInternal(moduleId, itemName) {
    if (!state.cy) return;

    const itemNodeId = `${moduleId}::${itemName}`;
    let itemNode = state.cy.getElementById(itemNodeId);

    if (!itemNode || itemNode.length === 0) {
        // Item node doesn't exist - need to enable Show Items
        const showItemsToggle = document.getElementById('show-items-toggle');
        if (showItemsToggle && !showItemsToggle.checked) {
            showItemsToggle.checked = true;
            setShowItems(true);
            rebuildGraph();

            // Wait for rebuild and then focus
            setTimeout(() => {
                itemNode = state.cy?.getElementById(itemNodeId);
                if (itemNode?.length) {
                    selectAndFocusItemInternal(itemNode);
                    if (!isNavigatingFromUrl) {
                        updateUrl(moduleId, itemName);
                    }
                }
            }, 800);
        }
    } else {
        selectAndFocusItemInternal(itemNode);
        if (!isNavigatingFromUrl) {
            updateUrl(moduleId, itemName);
        }
    }
}

/**
 * Select and focus on an item node
 */
function selectAndFocusItemInternal(node) {
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

// =====================================================
// View Toggle (Graph / Tree)
// =====================================================

function setupViewToggle() {
    const graphBtn = document.getElementById('view-graph');
    const treeBtn = document.getElementById('view-tree');

    if (!graphBtn || !treeBtn) return;

    graphBtn.addEventListener('click', () => {
        if (currentView === 'graph') return;

        currentView = 'graph';
        document.getElementById('cy').style.display = 'block';
        document.getElementById('tree-view')?.style.setProperty('display', 'none');

        // Update button states
        graphBtn.classList.add('active');
        treeBtn.classList.remove('active');

        // Ensure graph is properly sized
        if (state.cy) {
            state.cy.resize();
            state.cy.fit(undefined, 50);
        }
    });

    treeBtn.addEventListener('click', () => {
        if (currentView === 'tree') return;

        currentView = 'tree';
        document.getElementById('cy').style.display = 'none';
        const treeView = document.getElementById('tree-view');
        if (treeView) {
            treeView.style.display = 'block';
            populateTreeView();
        }

        // Update button states
        treeBtn.classList.add('active');
        graphBtn.classList.remove('active');
    });
}

// =====================================================
// Tree View
// =====================================================

// Cache for tree filter state
let treeFilterState = {
    pubOnly: true,
    showTypes: true,
    showTraits: true,
    showFns: false
};

function populateTreeView() {
    const container = document.getElementById('tree-view');
    if (!container || !state.graphData) return;

    // Filter out external nodes
    const internalNodes = state.graphData.nodes.filter(node => {
        const filePath = node.file_path;
        return filePath && !filePath.startsWith('[external]');
    });

    // Group by directory structure
    const groups = {
        'src/web/': [],
        'src/': []
    };

    internalNodes.forEach(node => {
        const filePath = node.file_path || '';
        if (filePath.includes('/web/')) {
            groups['src/web/'].push(node);
        } else {
            groups['src/'].push(node);
        }
    });

    // Sort modules alphabetically within each group
    Object.values(groups).forEach(nodes => {
        nodes.sort((a, b) => a.label.localeCompare(b.label));
    });

    // Calculate edge counts for each node
    const edgeCounts = {};
    state.graphData.edges.forEach(edge => {
        edgeCounts[edge.source] = edgeCounts[edge.source] || { out: 0, in: 0 };
        edgeCounts[edge.target] = edgeCounts[edge.target] || { out: 0, in: 0 };
        edgeCounts[edge.source].out++;
        edgeCounts[edge.target].in++;
    });

    const getItemIcon = (kind) => {
        switch (kind) {
            case 'fn': return 'ƒ';
            case 'type': return 'S';
            case 'struct': return 'S';
            case 'enum': return 'E';
            case 'trait': return 'T';
            case 'impl': return 'I';
            case 'const': return 'C';
            case 'static': return 'St';
            case 'macro': return 'M';
            default: return '•';
        }
    };

    const getItemKindLabel = (kind) => {
        switch (kind) {
            case 'fn': return 'fn';
            case 'type': return 'struct';
            case 'struct': return 'struct';
            case 'enum': return 'enum';
            case 'trait': return 'trait';
            case 'impl': return 'impl';
            case 'const': return 'const';
            case 'static': return 'static';
            case 'macro': return 'macro';
            default: return kind;
        }
    };

    const getVisibilityClass = (vis) => {
        if (vis === 'pub' || vis === 'Public') return 'vis-public';
        if (vis === 'pub(crate)' || vis === 'Crate') return 'vis-crate';
        return 'vis-private';
    };

    // Filter controls
    const filterHTML = `
        <div class="tree-filters">
            <div class="tree-filter-title">Filter Items</div>
            <div class="tree-filter-options">
                <label><input type="checkbox" id="tree-filter-pub" ${treeFilterState.pubOnly ? 'checked' : ''}> pub only</label>
                <label><input type="checkbox" id="tree-filter-types" ${treeFilterState.showTypes ? 'checked' : ''}> Structs/Enums</label>
                <label><input type="checkbox" id="tree-filter-traits" ${treeFilterState.showTraits ? 'checked' : ''}> Traits</label>
                <label><input type="checkbox" id="tree-filter-fns" ${treeFilterState.showFns ? 'checked' : ''}> Functions</label>
            </div>
        </div>
    `;

    container.innerHTML = filterHTML + Object.entries(groups)
        .filter(([_, nodes]) => nodes.length > 0)
        .map(([groupName, nodes]) => `
        <div class="tree-group">
            <div class="tree-group-header">
                <span class="tree-group-icon">📁</span>
                <span class="tree-group-name">${escapeHtml(groupName)}</span>
                <span class="tree-group-count">${nodes.length} modules</span>
            </div>
            <div class="tree-modules">
                ${nodes.map(n => {
                    const counts = edgeCounts[n.id] || { out: 0, in: 0 };
                    const items = n.items || [];
                    const fnCount = n.metrics?.fn_count || items.filter(i => i.kind === 'fn').length;
                    const typeCount = n.metrics?.type_count || items.filter(i => i.kind === 'type' || i.kind === 'trait').length;
                    const hasItems = items.length > 0;

                    // Group items by kind
                    const structs = items.filter(i => i.kind === 'type' && !i.is_trait);
                    const enums = items.filter(i => i.kind === 'enum');
                    const traits = items.filter(i => i.kind === 'trait' || (i.kind === 'type' && i.is_trait));
                    const functions = items.filter(i => i.kind === 'fn');
                    const consts = items.filter(i => i.kind === 'const' || i.kind === 'static');

                    const isPublic = (vis) => vis === 'pub' || vis === 'Public';

                    const renderItem = (item, category) => `
                        <div class="tree-item ${getVisibilityClass(item.visibility)}"
                             data-module="${n.id}"
                             data-item="${item.name}"
                             data-kind="${item.kind}"
                             data-category="${category}"
                             data-visibility="${item.visibility}">
                            <span class="tree-item-icon">${getItemIcon(item.kind)}</span>
                            <span class="tree-item-kind">${getItemKindLabel(item.kind)}</span>
                            <span class="tree-item-name">${escapeHtml(item.name)}</span>
                            ${isPublic(item.visibility) ? '<span class="tree-item-pub">pub</span>' : ''}
                        </div>
                    `;

                    return `
                    <div class="tree-module-container">
                        <div class="tree-module" data-node-id="${n.id}" data-has-items="${hasItems}">
                            <div class="tree-module-main">
                                <span class="tree-expand-icon">${hasItems ? '▶' : ' '}</span>
                                <span class="tree-module-icon">📄</span>
                                <span class="tree-module-name">${escapeHtml(n.label)}.rs</span>
                            </div>
                            <div class="tree-module-info">
                                <span class="tree-module-deps" title="Dependencies: ${counts.out} out, ${counts.in} in">
                                    <span class="dep-out">↑${counts.out}</span>
                                    <span class="dep-in">↓${counts.in}</span>
                                </span>
                                <span class="tree-module-items" title="Functions and Types">
                                    ${fnCount}fn ${typeCount}ty
                                </span>
                            </div>
                        </div>
                        ${hasItems ? `
                        <div class="tree-items" style="display: none;">
                            ${structs.length > 0 ? `
                            <div class="tree-item-group" data-group="types">
                                <div class="tree-item-group-header">Structs (${structs.length})</div>
                                ${structs.map(item => renderItem(item, 'types')).join('')}
                            </div>
                            ` : ''}
                            ${enums.length > 0 ? `
                            <div class="tree-item-group" data-group="types">
                                <div class="tree-item-group-header">Enums (${enums.length})</div>
                                ${enums.map(item => renderItem(item, 'types')).join('')}
                            </div>
                            ` : ''}
                            ${traits.length > 0 ? `
                            <div class="tree-item-group" data-group="traits">
                                <div class="tree-item-group-header">Traits (${traits.length})</div>
                                ${traits.map(item => renderItem(item, 'traits')).join('')}
                            </div>
                            ` : ''}
                            ${functions.length > 0 ? `
                            <div class="tree-item-group" data-group="fns">
                                <div class="tree-item-group-header">Functions (${functions.length})</div>
                                ${functions.map(item => renderItem(item, 'fns')).join('')}
                            </div>
                            ` : ''}
                            ${consts.length > 0 ? `
                            <div class="tree-item-group" data-group="consts">
                                <div class="tree-item-group-header">Constants (${consts.length})</div>
                                ${consts.map(item => renderItem(item, 'consts')).join('')}
                            </div>
                            ` : ''}
                        </div>
                        ` : ''}
                    </div>
                `}).join('')}
            </div>
        </div>
    `).join('');

    // Setup tree event handlers
    setupTreeEventHandlers(container);
}

/**
 * Setup tree event handlers (separated to prevent duplication)
 */
function setupTreeEventHandlers(container) {
    // Module click - go to graph with items shown
    container.querySelectorAll('.tree-module').forEach(item => {
        item.addEventListener('click', handleTreeModuleClick);
    });

    // Expand icon click - toggle tree expansion
    container.querySelectorAll('.tree-expand-icon').forEach(icon => {
        icon.addEventListener('click', handleTreeExpandClick);
    });

    // Item click - show item in graph
    container.querySelectorAll('.tree-item').forEach(item => {
        item.addEventListener('click', handleTreeItemClick);
    });

    // Filter functionality
    ['tree-filter-pub', 'tree-filter-types', 'tree-filter-traits', 'tree-filter-fns'].forEach(id => {
        const el = document.getElementById(id);
        if (el) {
            el.addEventListener('change', handleTreeFilterChange);
        }
    });

    // Apply initial filter
    applyTreeFilters(container);
}

function handleTreeModuleClick(e) {
    const item = e.currentTarget;
    const nodeId = item.dataset.nodeId;
    const hasItems = item.dataset.hasItems === 'true';

    // Enable Show Items to display functions/types
    const showItemsToggle = document.getElementById('show-items-toggle');
    if (hasItems && showItemsToggle && !showItemsToggle.checked) {
        showItemsToggle.checked = true;
        setShowItems(true);
        rebuildGraph();
    }

    // Switch to graph view and select the module
    switchToGraphView();

    // Wait for view switch and potential rebuild
    setTimeout(() => {
        const node = state.cy?.getElementById(nodeId);
        if (node?.length) {
            selectNode(node);
        }
    }, 300);
}

function handleTreeExpandClick(e) {
    e.stopPropagation();
    const icon = e.currentTarget;
    const moduleEl = icon.closest('.tree-module');
    const container = moduleEl?.closest('.tree-module-container');
    const itemsContainer = container?.querySelector('.tree-items');

    if (itemsContainer) {
        const isExpanded = itemsContainer.style.display !== 'none';
        itemsContainer.style.display = isExpanded ? 'none' : 'block';
        icon.textContent = isExpanded ? '▶' : '▼';
        moduleEl.classList.toggle('expanded', !isExpanded);
    }
}

function handleTreeItemClick(e) {
    e.stopPropagation();
    const item = e.currentTarget;
    const moduleId = item.dataset.module;
    const itemName = item.dataset.item;

    // Enable Show Items if not already
    const showItemsToggle = document.getElementById('show-items-toggle');
    if (showItemsToggle && !showItemsToggle.checked) {
        showItemsToggle.checked = true;
        setShowItems(true);
        rebuildGraph();
    }

    // Switch to graph and try to find the item node
    switchToGraphView();

    setTimeout(() => {
        const itemNodeId = `${moduleId}::${itemName}`;
        const itemNode = state.cy?.getElementById(itemNodeId);
        if (itemNode?.length) {
            selectNode(itemNode);
        } else {
            // Fallback to module
            const moduleNode = state.cy?.getElementById(moduleId);
            if (moduleNode?.length) {
                selectNode(moduleNode);
            }
        }
    }, 300);
}

function handleTreeFilterChange() {
    // Update filter state
    treeFilterState.pubOnly = document.getElementById('tree-filter-pub')?.checked ?? true;
    treeFilterState.showTypes = document.getElementById('tree-filter-types')?.checked ?? true;
    treeFilterState.showTraits = document.getElementById('tree-filter-traits')?.checked ?? true;
    treeFilterState.showFns = document.getElementById('tree-filter-fns')?.checked ?? false;

    const container = document.getElementById('tree-view');
    if (container) {
        applyTreeFilters(container);
    }
}

function applyTreeFilters(container) {
    const { pubOnly, showTypes, showTraits, showFns } = treeFilterState;

    container.querySelectorAll('.tree-item').forEach(item => {
        const visibility = item.dataset.visibility;
        const category = item.dataset.category;
        const isPublic = visibility === 'pub' || visibility === 'Public';

        let visible = true;

        // Visibility filter
        if (pubOnly && !isPublic) {
            visible = false;
        }

        // Category filter
        if (visible) {
            if (category === 'types' && !showTypes) visible = false;
            if (category === 'traits' && !showTraits) visible = false;
            if (category === 'fns' && !showFns) visible = false;
        }

        item.style.display = visible ? '' : 'none';
    });

    // Hide empty groups
    container.querySelectorAll('.tree-item-group').forEach(group => {
        const visibleItems = group.querySelectorAll('.tree-item:not([style*="display: none"])');
        group.style.display = visibleItems.length > 0 ? '' : 'none';
    });
}

/**
 * Switch to graph view
 */
function switchToGraphView() {
    if (currentView === 'graph') return;

    const graphBtn = document.getElementById('view-graph');
    const treeBtn = document.getElementById('view-tree');

    currentView = 'graph';
    document.getElementById('cy').style.display = 'block';
    document.getElementById('tree-view')?.style.setProperty('display', 'none');

    graphBtn?.classList.add('active');
    treeBtn?.classList.remove('active');

    // Ensure graph is properly sized
    if (state.cy) {
        state.cy.resize();
    }
}

// =====================================================
// Auto-hide Triggers
// =====================================================

function setupAutoHideTriggers() {
    const sidebar = document.querySelector('.sidebar');
    const toggleBtn = document.getElementById('sidebar-toggle');

    if (toggleBtn && sidebar) {
        toggleBtn.addEventListener('click', () => {
            sidebar.classList.toggle('collapsed');
            toggleBtn.textContent = sidebar.classList.contains('collapsed') ? '◀' : '▶';
        });
    }
}

// =====================================================
// Utilities
// =====================================================

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text || '';
    return div.innerHTML;
}

// =====================================================
// Start Application
// =====================================================

document.addEventListener('DOMContentLoaded', init);

// Export for debugging
window.cargoCoupling = {
    state,
    selectNode,
    clearSelection,
    applyLayout,
    graphQueue
};
