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
        highlightItemConnections(node);
        showItemDetails(node.data(), moduleNode);
    });

    // Click handler for edges
    itemCy.on('tap', 'edge', function(evt) {
        const edge = evt.target;
        showEdgeDetails(edge.data());
    });

    // Double-click to navigate to external module
    itemCy.on('dbltap', 'node', function(evt) {
        const node = evt.target;
        if (node.data('isExternal') && node.data('kind') === 'module') {
            // Navigate to this module in the main graph
            const moduleName = node.data('label');
            navigateToModule(moduleName);
        }
    });
}

/**
 * Highlight connections for a node
 */
function highlightItemConnections(node) {
    if (!state.itemCy) return;

    // Reset all
    state.itemCy.elements().removeClass('highlighted dimmed');

    // Highlight selected node and its connections
    state.itemCy.elements().addClass('dimmed');
    node.removeClass('dimmed').addClass('highlighted');
    node.neighborhood().removeClass('dimmed');
}

/**
 * Navigate to a module in the main graph
 */
function navigateToModule(moduleName) {
    if (!state.cy) return;

    const node = state.cy.nodes().filter(n => {
        const label = n.data('label') || '';
        return label === moduleName || label.endsWith('::' + moduleName);
    }).first();

    if (node && node.length > 0) {
        // Select and center on the node
        state.cy.elements().removeClass('highlighted dimmed');
        node.addClass('highlighted');
        state.cy.animate({
            center: { eles: node },
            zoom: 1.5,
            duration: 500
        });
    }
}

/**
 * Show edge details in the item details panel
 */
function showEdgeDetails(data) {
    const container = document.getElementById('item-details');
    if (!container) return;

    const strengthClass = (data.strength || '').toLowerCase();
    const edgeKindLabels = {
        'same-module': 'Same Module',
        'different-module': 'Different Module',
        'external-crate': 'External Crate'
    };

    container.innerHTML = `
        <div class="item-detail-header">Dependency Details</div>
        <div class="item-edge-info">
            <div class="edge-endpoints">
                <span class="edge-source">${escapeHtml(data.source)}</span>
                <span class="edge-arrow">→</span>
                <span class="edge-target">${escapeHtml(data.fullTarget || data.target)}</span>
            </div>
        </div>
        <div class="item-detail-section">
            <div class="detail-row">
                <span class="detail-label">Strength:</span>
                <span class="strength-badge ${strengthClass}">${data.strength || 'Unknown'}</span>
            </div>
            <div class="detail-row">
                <span class="detail-label">Distance:</span>
                <span class="distance-badge">${data.distance || 'Unknown'}</span>
            </div>
            <div class="detail-row">
                <span class="detail-label">Type:</span>
                <span>${edgeKindLabels[data.edgeKind] || data.edgeKind || 'Unknown'}</span>
            </div>
            ${data.depType ? `
                <div class="detail-row">
                    <span class="detail-label">Dependency Type:</span>
                    <span>${escapeHtml(data.depType)}</span>
                </div>
            ` : ''}
        </div>
        ${data.expression ? `
            <div class="item-detail-section">
                <div class="item-deps-header">Expression</div>
                <div class="dep-expression-box">
                    <code>${escapeHtml(data.expression)}</code>
                </div>
            </div>
        ` : ''}
        <div class="item-detail-hint">
            <span class="hint-icon">💡</span>
            <span>Click on nodes to see their details</span>
        </div>
    `;
}

/**
 * Build Cytoscape elements for item graph
 */
function buildItemElements(items, moduleName) {
    const nodes = [];
    const edges = [];
    const edgeSet = new Set(); // Prevent duplicate edges
    const itemNames = new Set(items.map(i => i.name));

    // Debug: Log available items
    console.log('[Item Graph] Building graph for module:', moduleName);
    console.log('[Item Graph] Items count:', items.length);
    console.log('[Item Graph] Item names:', Array.from(itemNames));

    // Track external targets for virtual nodes
    const externalTargets = new Map(); // target -> { deps: [], module: string }

    let sameModuleEdgeCount = 0;

    items.forEach(item => {
        const allDeps = item.dependencies || [];
        const sameModuleDeps = allDeps.filter(d => d.distance === 'SameModule' || d.distance === 'SameFunction');
        const otherModuleDeps = allDeps.filter(d => d.distance === 'DifferentModule');
        const externalCrateDeps = allDeps.filter(d => d.distance === 'DifferentCrate');

        // Add node with full dependency info
        nodes.push({
            data: {
                id: item.name,
                label: item.name,
                kind: item.kind,
                visibility: item.visibility,
                depCount: sameModuleDeps.length + otherModuleDeps.length,
                externalDepCount: externalCrateDeps.length,
                sameModuleDepCount: sameModuleDeps.length,
                otherModuleDepCount: otherModuleDeps.length,
                dependencies: allDeps,
                fullItem: item
            }
        });

        // Process ALL dependencies for edges
        allDeps.forEach(dep => {
            const targetParts = dep.target.split('::');
            const targetName = targetParts[targetParts.length - 1];
            const targetModule = targetParts.length > 1 ? targetParts.slice(0, -1).join('::') : '';

            // Check if target is in the same module (same module items)
            if (itemNames.has(targetName) && (dep.distance === 'SameModule' || dep.distance === 'SameFunction')) {
                const edgeId = `${item.name}->${targetName}`;
                if (!edgeSet.has(edgeId) && item.name !== targetName) {
                    edgeSet.add(edgeId);
                    sameModuleEdgeCount++;
                    console.log('[Item Graph] Creating same-module edge:', item.name, '->', targetName);
                    edges.push({
                        data: {
                            id: edgeId,
                            source: item.name,
                            target: targetName,
                            depType: dep.dep_type,
                            strength: dep.strength,
                            distance: dep.distance,
                            expression: dep.expression,
                            fullTarget: dep.target,
                            internal: true,
                            edgeKind: 'same-module'
                        }
                    });
                }
            } else {
                // External dependency (different module or crate)
                let virtualNodeId;
                let virtualLabel;
                let edgeKind;

                if (dep.distance === 'DifferentCrate') {
                    // External crate - use crate name
                    virtualNodeId = `crate:${targetParts[0]}`;
                    virtualLabel = targetParts[0];
                    edgeKind = 'external-crate';
                } else {
                    // Different module in same crate
                    virtualNodeId = `mod:${targetModule || targetName}`;
                    virtualLabel = targetModule ? targetModule.split('::').pop() : targetName;
                    edgeKind = 'different-module';
                }

                // Track for virtual node creation
                if (!externalTargets.has(virtualNodeId)) {
                    externalTargets.set(virtualNodeId, {
                        label: virtualLabel,
                        kind: dep.distance === 'DifferentCrate' ? 'crate' : 'module',
                        deps: []
                    });
                }
                externalTargets.get(virtualNodeId).deps.push({
                    from: item.name,
                    dep: dep
                });

                const edgeId = `${item.name}->${virtualNodeId}`;
                if (!edgeSet.has(edgeId)) {
                    edgeSet.add(edgeId);
                    edges.push({
                        data: {
                            id: edgeId,
                            source: item.name,
                            target: virtualNodeId,
                            depType: dep.dep_type,
                            strength: dep.strength,
                            distance: dep.distance,
                            expression: dep.expression,
                            fullTarget: dep.target,
                            internal: false,
                            edgeKind: edgeKind
                        }
                    });
                }
            }
        });
    });

    // Create virtual nodes for external targets
    externalTargets.forEach((info, id) => {
        nodes.push({
            data: {
                id: id,
                label: info.label,
                kind: info.kind,
                visibility: 'external',
                depCount: 0,
                externalDepCount: 0,
                isExternal: true,
                externalDeps: info.deps
            }
        });
    });

    console.log('[Item Graph] Graph built - Nodes:', nodes.length, 'Edges:', edges.length, 'Same-module edges:', sameModuleEdgeCount);

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

    // Handle external node click
    if (data.isExternal) {
        showExternalNodeDetails(data, container);
        return;
    }

    const deps = data.dependencies || [];
    const sameModuleDeps = deps.filter(d => d.distance === 'SameModule' || d.distance === 'SameFunction');
    const otherModuleDeps = deps.filter(d => d.distance === 'DifferentModule');
    const externalDeps = deps.filter(d => d.distance === 'DifferentCrate');

    // Group by dependency type
    const groupByType = (depList) => {
        const groups = {};
        depList.forEach(d => {
            const type = d.dep_type || 'Other';
            if (!groups[type]) groups[type] = [];
            groups[type].push(d);
        });
        return groups;
    };

    const renderDepGroup = (deps, groupName, showExpression = true) => {
        if (deps.length === 0) return '';
        return `
            <div class="item-deps-group">
                <div class="item-deps-group-header">${groupName} (${deps.length})</div>
                ${deps.map(d => `
                    <div class="item-dep-card" data-target="${escapeHtml(d.target)}">
                        <div class="dep-card-header">
                            <span class="strength-badge ${(d.strength || '').toLowerCase()}">${d.strength || '?'}</span>
                            <span class="dep-target-name">${escapeHtml(d.target.split('::').pop())}</span>
                        </div>
                        <div class="dep-card-path">${escapeHtml(d.target)}</div>
                        ${showExpression && d.expression ? `
                            <div class="dep-card-expression">
                                <code>${escapeHtml(d.expression)}</code>
                            </div>
                        ` : ''}
                        <div class="dep-card-meta">
                            <span class="meta-tag">${d.dep_type || 'Unknown'}</span>
                            <span class="meta-tag distance">${d.distance || ''}</span>
                        </div>
                    </div>
                `).join('')}
            </div>
        `;
    };

    const kindColors = {
        fn: 'fn',
        type: 'type',
        struct: 'type',
        enum: 'type',
        trait: 'trait'
    };

    const totalDeps = deps.length;
    const incomingDeps = findIncomingDeps(data.label, moduleNode);

    container.innerHTML = `
        <div class="item-detail-header">${escapeHtml(data.label)}</div>
        <div class="item-detail-badges">
            <span class="item-kind-badge ${kindColors[data.kind] || 'type'}">${data.kind}</span>
            <span class="item-visibility-badge ${data.visibility === 'pub' ? 'public' : 'private'}">${data.visibility}</span>
        </div>
        <div class="item-stats-grid">
            <div class="item-stat">
                <span class="stat-value">${totalDeps}</span>
                <span class="stat-label">Outgoing</span>
            </div>
            <div class="item-stat">
                <span class="stat-value">${incomingDeps.length}</span>
                <span class="stat-label">Incoming</span>
            </div>
            <div class="item-stat">
                <span class="stat-value">${sameModuleDeps.length}</span>
                <span class="stat-label">Internal</span>
            </div>
            <div class="item-stat">
                <span class="stat-value">${externalDeps.length}</span>
                <span class="stat-label">External</span>
            </div>
        </div>

        ${incomingDeps.length > 0 ? `
            <div class="item-deps-section incoming">
                <div class="item-deps-header">
                    <span class="deps-icon">⬅</span>
                    Used By (${incomingDeps.length})
                </div>
                ${incomingDeps.slice(0, 5).map(d => `
                    <div class="item-dep-simple incoming">
                        <span class="dep-name">${escapeHtml(d.from)}</span>
                        <span class="strength-badge ${(d.strength || '').toLowerCase()}">${d.strength || '?'}</span>
                    </div>
                `).join('')}
                ${incomingDeps.length > 5 ? `<div class="item-dep-more">+${incomingDeps.length - 5} more</div>` : ''}
            </div>
        ` : ''}

        ${sameModuleDeps.length > 0 ? `
            <div class="item-deps-section same-module">
                <div class="item-deps-header">
                    <span class="deps-icon">🏠</span>
                    Same Module Dependencies (${sameModuleDeps.length})
                </div>
                ${renderDepGroup(sameModuleDeps, '', true)}
            </div>
        ` : ''}

        ${otherModuleDeps.length > 0 ? `
            <div class="item-deps-section other-module">
                <div class="item-deps-header">
                    <span class="deps-icon">📦</span>
                    Other Module Dependencies (${otherModuleDeps.length})
                </div>
                ${renderDepGroup(otherModuleDeps, '', true)}
            </div>
        ` : ''}

        ${externalDeps.length > 0 ? `
            <div class="item-deps-section external">
                <div class="item-deps-header">
                    <span class="deps-icon">🌐</span>
                    External Crate Dependencies (${externalDeps.length})
                </div>
                ${renderDepGroup(externalDeps.slice(0, 10), '', false)}
                ${externalDeps.length > 10 ? `<div class="item-dep-more">+${externalDeps.length - 10} more</div>` : ''}
            </div>
        ` : ''}

        ${totalDeps === 0 && incomingDeps.length === 0 ? `
            <div class="item-no-deps">
                <span class="no-deps-icon">✨</span>
                <span>No dependencies detected</span>
            </div>
        ` : ''}

        <div class="item-detail-hint">
            <span class="hint-icon">💡</span>
            <span>Double-click external modules to navigate</span>
        </div>
    `;

    // Add click handlers for dependency cards
    container.querySelectorAll('.item-dep-card').forEach(card => {
        card.addEventListener('click', () => {
            const target = card.dataset.target;
            highlightTargetInGraph(target);
        });
    });
}

/**
 * Find items that depend on this item
 */
function findIncomingDeps(itemName, moduleNode) {
    if (!moduleNode || !moduleNode.items) return [];

    const incoming = [];
    moduleNode.items.forEach(item => {
        if (item.name === itemName) return;
        const deps = item.dependencies || [];
        deps.forEach(dep => {
            if (dep.target.endsWith('::' + itemName) || dep.target === itemName) {
                incoming.push({
                    from: item.name,
                    strength: dep.strength,
                    expression: dep.expression
                });
            }
        });
    });
    return incoming;
}

/**
 * Highlight target in the item graph
 */
function highlightTargetInGraph(targetPath) {
    if (!state.itemCy) return;

    const targetName = targetPath.split('::').pop();
    const node = state.itemCy.getElementById(targetName);

    if (node && node.length > 0) {
        state.itemCy.elements().removeClass('highlighted dimmed');
        state.itemCy.elements().addClass('dimmed');
        node.removeClass('dimmed').addClass('highlighted');
        node.neighborhood().removeClass('dimmed');

        state.itemCy.animate({
            center: { eles: node },
            duration: 300
        });
    }
}

/**
 * Show details for external nodes
 */
function showExternalNodeDetails(data, container) {
    const externalDeps = data.externalDeps || [];
    const kindLabel = data.kind === 'crate' ? 'External Crate' : 'External Module';

    container.innerHTML = `
        <div class="item-detail-header">${escapeHtml(data.label)}</div>
        <div class="item-detail-badges">
            <span class="item-kind-badge module">${kindLabel}</span>
        </div>
        <div class="item-deps-section">
            <div class="item-deps-header">
                <span class="deps-icon">⬅</span>
                Referenced By (${externalDeps.length})
            </div>
            ${externalDeps.map(d => `
                <div class="item-dep-card">
                    <div class="dep-card-header">
                        <span class="dep-target-name">${escapeHtml(d.from)}</span>
                        <span class="strength-badge ${(d.dep.strength || '').toLowerCase()}">${d.dep.strength || '?'}</span>
                    </div>
                    <div class="dep-card-path">${escapeHtml(d.dep.target)}</div>
                    ${d.dep.expression ? `
                        <div class="dep-card-expression">
                            <code>${escapeHtml(d.dep.expression)}</code>
                        </div>
                    ` : ''}
                </div>
            `).join('')}
        </div>
        <div class="item-detail-hint">
            <span class="hint-icon">💡</span>
            <span>Double-click to navigate to this module</span>
        </div>
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
