// =====================================================
// Item Graph Module (Module Internal Dependencies)
// =====================================================

import { state, setItemCy, setCurrentModuleForItemGraph } from './state.js';

let currentModuleNode = null;
let isFiltersSetup = false;

/**
 * Show item graph for a module
 */
export function showItemGraph(moduleId) {
    const panel = document.getElementById('item-graph-panel');
    const container = document.getElementById('item-graph-container');
    if (!panel || !container || !state.graphData) {
        return;
    }

    // Find the module node
    const moduleNode = state.graphData.nodes.find(n => n.id === moduleId);
    if (!moduleNode || !moduleNode.items || moduleNode.items.length === 0) {
        panel.style.display = 'none';
        return;
    }

    currentModuleNode = moduleNode;
    setCurrentModuleForItemGraph(moduleId);
    panel.style.display = 'block';

    // Update panel title
    const titleEl = panel.querySelector('h2');
    if (titleEl) {
        titleEl.innerHTML = `📦 ${escapeHtml(moduleNode.label)} Items <button id="close-item-graph" class="btn-close">×</button>`;
    }

    // Build and render item graph
    renderItemGraph(moduleNode);

    // Setup filter handlers only once
    if (!isFiltersSetup) {
        setupItemFilters();
        isFiltersSetup = true;
    }

    // Setup close button (need to re-attach since we replaced innerHTML)
    setupCloseButton();
}

/**
 * Setup close button handler
 */
function setupCloseButton() {
    const closeBtn = document.getElementById('close-item-graph');
    if (closeBtn) {
        // Remove any existing listener by cloning
        const newBtn = closeBtn.cloneNode(true);
        closeBtn.parentNode.replaceChild(newBtn, closeBtn);
        newBtn.addEventListener('click', hideItemGraph);
    }
}

/**
 * Hide item graph panel
 */
export function hideItemGraph() {
    const panel = document.getElementById('item-graph-panel');
    if (panel) panel.style.display = 'none';

    if (state.itemCy) {
        state.itemCy.destroy();
        setItemCy(null);
    }
    currentModuleNode = null;
}

/**
 * Render item graph
 */
function renderItemGraph(moduleNode) {
    const container = document.getElementById('item-graph-container');
    if (!container) return;

    // Get filter settings
    const showFn = document.getElementById('item-filter-fn')?.checked ?? true;
    const showType = document.getElementById('item-filter-type')?.checked ?? true;
    const showTrait = document.getElementById('item-filter-trait')?.checked ?? true;

    // Filter items based on checkboxes
    const items = (moduleNode.items || []).filter(item => {
        if (item.kind === 'fn' && !showFn) return false;
        if (item.kind === 'type' && !showType) return false;
        if (item.kind === 'trait' && !showTrait) return false;
        return true;
    });

    // Build elements
    const elements = buildItemElements(items, moduleNode.label);

    // Update item count
    const countEl = document.getElementById('item-count');
    if (countEl) countEl.textContent = items.length;

    // Destroy existing instance
    if (state.itemCy) {
        state.itemCy.destroy();
        setItemCy(null);
    }

    // Create new Cytoscape instance
    const itemCy = cytoscape({
        container: container,
        elements: elements,
        style: getItemCytoscapeStyle(),
        layout: {
            name: 'cose',
            animate: false,
            nodeRepulsion: 8000,
            idealEdgeLength: 80,
            padding: 20
        },
        minZoom: 0.3,
        maxZoom: 2,
        wheelSensitivity: 0.3
    });

    setItemCy(itemCy);

    // Click handler for item nodes
    itemCy.on('tap', 'node', function(evt) {
        const node = evt.target;
        showItemDetails(node.data(), moduleNode);
    });
}

/**
 * Build Cytoscape elements for item graph
 */
function buildItemElements(items, moduleName) {
    const nodes = [];
    const edges = [];
    const edgeSet = new Set(); // Prevent duplicate edges
    const itemNames = new Set(items.map(i => i.name));

    // Create set of internal module targets
    const internalTargets = new Set();

    items.forEach(item => {
        // Count internal dependencies (not external crate)
        const internalDeps = (item.dependencies || []).filter(d => d.distance !== 'DifferentCrate');
        const externalDeps = (item.dependencies || []).filter(d => d.distance === 'DifferentCrate');

        // Add node
        nodes.push({
            data: {
                id: item.name,
                label: item.name,
                kind: item.kind,
                visibility: item.visibility,
                depCount: internalDeps.length,
                externalDepCount: externalDeps.length,
                dependencies: item.dependencies || []
            }
        });

        // Add edges for internal dependencies (within same module or to other internal modules)
        internalDeps.forEach(dep => {
            const targetParts = dep.target.split('::');
            const targetName = targetParts[targetParts.length - 1];

            // Check if target is in the same module
            if (itemNames.has(targetName)) {
                const edgeId = `${item.name}->${targetName}`;
                if (!edgeSet.has(edgeId) && item.name !== targetName) {
                    edgeSet.add(edgeId);
                    edges.push({
                        data: {
                            id: edgeId,
                            source: item.name,
                            target: targetName,
                            depType: dep.dep_type,
                            strength: dep.strength,
                            expression: dep.expression,
                            internal: true
                        }
                    });
                }
            } else {
                // Add virtual node for external module dependency
                const externalModule = targetParts.length > 1 ? targetParts[0] : targetName;
                if (!internalTargets.has(externalModule)) {
                    internalTargets.add(externalModule);
                    nodes.push({
                        data: {
                            id: `ext:${externalModule}`,
                            label: externalModule,
                            kind: 'module',
                            visibility: 'external',
                            depCount: 0,
                            externalDepCount: 0,
                            isExternal: true
                        }
                    });
                }

                const edgeId = `${item.name}->ext:${externalModule}`;
                if (!edgeSet.has(edgeId)) {
                    edgeSet.add(edgeId);
                    edges.push({
                        data: {
                            id: edgeId,
                            source: item.name,
                            target: `ext:${externalModule}`,
                            depType: dep.dep_type,
                            strength: dep.strength,
                            expression: dep.expression,
                            internal: false
                        }
                    });
                }
            }
        });
    });

    return [...nodes, ...edges];
}

/**
 * Get Cytoscape style for item graph
 */
function getItemCytoscapeStyle() {
    return [
        {
            selector: 'node',
            style: {
                'label': 'data(label)',
                'text-valign': 'center',
                'text-halign': 'center',
                'font-size': '9px',
                'color': '#f8fafc',
                'text-outline-color': '#0f172a',
                'text-outline-width': 1,
                'width': node => 30 + (node.data('depCount') || 0) * 3,
                'height': node => 30 + (node.data('depCount') || 0) * 3,
                'background-color': node => {
                    if (node.data('isExternal')) return '#475569';
                    const kind = node.data('kind');
                    if (kind === 'fn') return '#3b82f6';
                    if (kind === 'trait') return '#22c55e';
                    return '#8b5cf6';
                },
                'border-width': node => {
                    if (node.data('isExternal')) return 2;
                    return node.data('visibility') === 'pub' ? 2 : 1;
                },
                'border-color': node => {
                    if (node.data('isExternal')) return '#64748b';
                    return node.data('visibility') === 'pub' ? '#fbbf24' : '#475569';
                },
                'shape': node => node.data('isExternal') ? 'rectangle' : 'ellipse'
            }
        },
        {
            selector: 'edge',
            style: {
                'width': 2,
                'line-color': edge => {
                    if (!edge.data('internal')) return '#64748b';
                    const strength = edge.data('strength');
                    if (strength === 'Intrusive') return '#ef4444';
                    if (strength === 'Functional') return '#f97316';
                    return '#6b7280';
                },
                'target-arrow-color': edge => {
                    if (!edge.data('internal')) return '#64748b';
                    const strength = edge.data('strength');
                    if (strength === 'Intrusive') return '#ef4444';
                    if (strength === 'Functional') return '#f97316';
                    return '#6b7280';
                },
                'target-arrow-shape': 'triangle',
                'arrow-scale': 1,
                'curve-style': 'bezier',
                'opacity': edge => edge.data('internal') ? 0.8 : 0.5,
                'line-style': edge => edge.data('internal') ? 'solid' : 'dashed'
            }
        },
        {
            selector: 'node:selected',
            style: {
                'border-width': 3,
                'border-color': '#3b82f6'
            }
        },
        {
            selector: '.highlighted',
            style: {
                'border-width': 3,
                'border-color': '#3b82f6'
            }
        },
        {
            selector: '.dimmed',
            style: {
                'opacity': 0.2
            }
        }
    ];
}

/**
 * Setup item filter handlers (only called once)
 */
function setupItemFilters() {
    const filterIds = ['item-filter-fn', 'item-filter-type', 'item-filter-trait'];
    filterIds.forEach(id => {
        const el = document.getElementById(id);
        if (el) {
            el.addEventListener('change', () => {
                if (currentModuleNode) {
                    renderItemGraph(currentModuleNode);
                }
            });
        }
    });
}

/**
 * Show item details
 */
function showItemDetails(data, moduleNode) {
    const container = document.getElementById('item-details');
    if (!container) return;

    const deps = data.dependencies || [];
    const internalDeps = deps.filter(d => d.distance !== 'DifferentCrate');
    const externalDeps = deps.filter(d => d.distance === 'DifferentCrate');

    container.innerHTML = `
        <div class="item-detail-header">${escapeHtml(data.label)}</div>
        <div class="item-detail-row">
            <span class="item-kind-badge ${data.kind}">${data.kind}</span>
            <span class="item-visibility-badge">${data.visibility}</span>
        </div>
        ${internalDeps.length > 0 ? `
            <div class="item-deps-section">
                <div class="item-deps-header">Internal Dependencies (${internalDeps.length})</div>
                ${internalDeps.map(d => `
                    <div class="item-dep">
                        <span class="strength-badge ${(d.strength || '').toLowerCase()}">${d.strength || 'Unknown'}</span>
                        ${escapeHtml(d.target.split('::').pop())}
                    </div>
                `).join('')}
            </div>
        ` : ''}
        ${externalDeps.length > 0 ? `
            <div class="item-deps-section">
                <div class="item-deps-header">External Dependencies (${externalDeps.length})</div>
                ${externalDeps.slice(0, 5).map(d => `
                    <div class="item-dep external">${escapeHtml(d.target)}</div>
                `).join('')}
                ${externalDeps.length > 5 ? `<div class="item-dep more">... and ${externalDeps.length - 5} more</div>` : ''}
            </div>
        ` : ''}
    `;
}

/**
 * Escape HTML
 */
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text || '';
    return div.innerHTML;
}
