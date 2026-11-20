// =====================================================
// Cytoscape Graph Module
// =====================================================

import { state, setCy, setCurrentLayout } from './state.js';
import { t } from './i18n.js';
import { moduleLabel } from './module-labels.js';
import { ScreenLabels } from './screen-labels.js';

const COLORS = {
    core: '#fb7185',
    supporting: '#38bdf8',
    generic: '#cbd5e1',
    unknown: '#8291a6',
    accidental: '#fb923c',
    good: '#34d399',
    review: '#facc15',
    critical: '#f87171',
    stable: '#60a5fa'
};

/**
 * Initialize Cytoscape graph with data
 */
export function initCytoscape(data, onNodeTap, onEdgeTap, onBackgroundTap, options = {}) {
    const elements = buildElements(data, options);

    const cy = cytoscape({
        container: document.getElementById('cy'),
        elements: elements,
        style: getCytoscapeStyle(),
        layout: { ...getLayoutConfig('dagre'), animate: false },
        minZoom: 0.05,
        maxZoom: 3,
        pixelRatio: 'auto'
    });

    setCy(cy);
    setupGraphEventHandlers(cy, onNodeTap, onEdgeTap, onBackgroundTap);
    const labels = new ScreenLabels(document.getElementById('cy'), id => onNodeTap(cy.getElementById(id)));
    const updateLabels = () => {
        if (cy.zoom() >= .65) { labels.render([]); return; }
        const nodes = cy.nodes(':visible').sort((a, b) => Number(b.id() === state.selectedNode?.id()) - Number(a.id() === state.selectedNode?.id()) || b.degree() - a.degree() || a.id().localeCompare(b.id()));
        labels.render(nodes.map(node => ({ id: node.id(), text: node.data('graph_label') || node.data('label'), ...node.renderedPosition() })));
    };
    let labelFrame;
    cy.on('pan zoom resize layoutstop data add remove', () => {
        if (!labelFrame) labelFrame = requestAnimationFrame(() => { labelFrame = undefined; updateLabels(); });
    });
    cy.on('destroy', () => { cancelAnimationFrame(labelFrame); labels.layer.remove(); });
    updateLabels();
    return cy;
}

/**
 * Build Cytoscape elements from graph data
 * @param {Object} data - Graph data from API
 * @param {Object} options - Build options
 * @param {boolean} options.showItems - Include item-level nodes (functions, types)
 */
export function buildElements(data, options = {}) {
    const { showItems = false } = options;
    const nodes = [];
    const itemNodeIds = new Set();

    // Build module nodes
    data.nodes.forEach(node => {
        const crate = node.id.split('::')[0];
        const items = node.items || [];

        // Count items by kind (prefer API values, fallback to counting items)
        const fnCount = node.metrics?.fn_count ?? items.filter(i => i.kind === 'fn').length;
        const typeCount = node.metrics?.type_count ?? items.filter(i => i.kind === 'type' || i.kind === 'trait').length;
        const implCount = node.metrics?.impl_count ?? ((node.metrics?.trait_impl_count || 0) + (node.metrics?.inherent_impl_count || 0));

        const statsStr = `${fnCount} functions, ${typeCount} types, ${implCount} impls`;

        nodes.push({
            data: {
                id: node.id,
                label: node.label,
                graph_label: moduleLabel(node),
                label_title: node.label,
                crate: crate,
                nodeType: 'module',
                subdomain: node.subdomain,
                expected_volatility: node.expected_volatility,
                flags: node.flags || [],
                accidental_volatility: (node.flags || []).includes('AccidentalVolatility'),
                ...node.metrics,
                file_path: node.file_path,
                in_cycle: node.in_cycle,
                items: node.items,
                fn_count: fnCount,
                type_count: typeCount,
                impl_count: implCount,
                stats_label: statsStr
            }
        });

        // Add item nodes if enabled
        if (showItems && items.length > 0) {
            items.forEach(item => {
                const itemId = `${node.id}::${item.name}`;
                itemNodeIds.add(itemId);
                nodes.push({
                    data: {
                        id: itemId,
                        label: item.name,
                        nodeType: 'item',
                        itemKind: item.kind,
                        visibility: item.visibility,
                        parentModule: node.id,
                        depCount: (item.dependencies || []).length
                    }
                });
            });
        }
    });

    // Aggregate edges between same source-target pairs
    const edgeMap = new Map();

    data.edges.forEach(edge => {
        const key = `${edge.source}->${edge.target}`;
        if (!edgeMap.has(key)) {
            edgeMap.set(key, {
                representative: edge,
                inCycle: edge.in_cycle,
                count: 1,
                originalIds: [edge.id]
            });
        } else {
            const existing = edgeMap.get(key);
            if (compareObservations(edge, existing.representative) < 0) existing.representative = edge;
            existing.inCycle = existing.inCycle || edge.in_cycle;
            existing.count++;
            existing.originalIds.push(edge.id);
        }
    });

    const edges = Array.from(edgeMap.entries()).sort(([a], [b]) => compareIds(a, b)).map(([key, group]) => {
        const edge = group.representative;
        return { data: {
            ...edgeDataToDetails(edge),
            id: `edge-${sanitizeElementId(key)}`,
            inCycle: group.inCycle,
            count: group.count,
            originalIds: group.originalIds.sort(compareIds)
        } };
    });

    (data.hidden_couplings || []).forEach(hidden => {
        edges.push({ data: hiddenCouplingDetails(hidden) });
    });

    // Add item-level edges if enabled
    if (showItems) {
        let itemEdgeIdx = 0;

        // Add parent edges (item -> module)
        data.nodes.forEach(node => {
            (node.items || []).forEach(item => {
                const itemId = `${node.id}::${item.name}`;
                edges.push({
                    data: {
                        id: `ie-parent-${itemEdgeIdx++}`,
                        source: itemId,
                        target: node.id,
                        edgeType: 'parent',
                        strengthLabel: 'Parent'
                    }
                });

                // Add item dependency edges
                (item.dependencies || []).forEach(dep => {
                    // Find target - could be in same module or different module
                    const targetName = dep.target.split('::').pop();
                    let targetId = null;

                    // Check if target is an item in our graph
                    for (const n of data.nodes) {
                        const matchingItem = (n.items || []).find(i => i.name === targetName);
                        if (matchingItem) {
                            targetId = `${n.id}::${matchingItem.name}`;
                            break;
                        }
                    }

                    // If not found as item, try as module
                    if (!targetId) {
                        const moduleMatch = data.nodes.find(n => n.id === targetName || n.label === targetName);
                        if (moduleMatch) {
                            targetId = moduleMatch.id;
                        }
                    }

                    if (targetId && targetId !== itemId) {
                        edges.push({
                            data: {
                                id: `ie-dep-${itemEdgeIdx++}`,
                                source: itemId,
                                target: targetId,
                                edgeType: 'item-dep',
                                depType: dep.dep_type,
                                strengthLabel: dep.strength || 'Model',
                                distance: dep.distance || 'SameModule'
                            }
                        });
                    }
                });
            });
        });
    }

    return [...nodes, ...edges];
}

/**
 * Get Cytoscape style configuration
 */
export function getCytoscapeStyle() {
    return [
        // Module node styles
        {
            selector: 'node[nodeType="module"]',
            style: {
                'label': node => {
                    return node.data('graph_label') || node.data('label') || '';
                },
                'text-valign': 'center',
                'text-halign': 'center',
                'text-wrap': 'wrap',
                'text-max-width': '240px',
                'background-color': node => getNodeColor(node.data()),
                'border-width': 2.5,
                'border-color': node => node.data('accidental_volatility') ? '#f97316' : '#475569',
                'color': '#f8fafc',
                'font-size': '12px',
                'font-weight': 700,
                'line-height': 1.18,
                'text-outline-color': '#020617',
                'text-outline-width': 3,
                'min-zoomed-font-size': 7.8,
                'width': node => moduleNodeWidth(node),
                'height': node => moduleNodeHeight(node),
                'shape': 'roundrectangle'
            }
        },
        // Item node styles (functions, types, traits)
        {
            selector: 'node[nodeType="item"]',
            style: {
                'label': 'data(label)',
                'text-valign': 'center',
                'text-halign': 'center',
                'font-size': '11px',
                'text-wrap': 'wrap',
                'text-max-width': 200,
                'color': '#f8fafc',
                'text-outline-color': '#0f172a',
                'text-outline-width': 1,
                'width': node => Math.min(220, Math.max(80, node.data('label').length * 7)),
                'height': 40,
                'shape': 'roundrectangle',
                'background-color': node => {
                    const kind = node.data('itemKind');
                    if (kind === 'fn') return '#3b82f6';
                    if (kind === 'trait') return '#22c55e';
                    return '#8b5cf6';
                },
                'border-width': node => node.data('visibility') === 'pub' ? 2 : 1,
                'border-color': node => node.data('visibility') === 'pub' ? '#fbbf24' : '#475569'
            }
        },
        // Fallback for nodes without nodeType
        {
            selector: 'node[!nodeType]',
            style: {
                'label': 'data(label)',
                'text-valign': 'center',
                'text-halign': 'center',
                'background-color': '#64748b',
                'border-width': 2,
                'border-color': '#475569',
                'color': '#f8fafc',
                'font-size': '11px',
                'font-weight': 700,
                'text-outline-color': '#0f172a',
                'text-outline-width': 2,
                'width': 40,
                'height': 40
            }
        },
        // Cluster coloring
        {
            selector: 'node[clusterColor]',
            style: {
                'background-color': 'data(clusterColor)',
                'border-color': 'data(clusterColor)'
            }
        },
        // Edge styles - base
        {
            selector: 'edge',
            style: {
                'width': edge => 1.5 + edge.data('strength') * 4.5,
                'line-color': edge => getEdgeColorByAnalysis(edge.data()),
                'target-arrow-color': edge => getEdgeColorByAnalysis(edge.data()),
                'target-arrow-shape': 'triangle',
                'arrow-scale': 1.25,
                'curve-style': 'bezier',
                'opacity': 0.78,
                'line-style': edge => getDistanceStyle(edge.data('distance'))
            }
        },
        // Critical coupling edges
        {
            selector: 'edge[strengthLabel="Intrusive"][distance="DifferentCrate"], edge[strengthLabel="Intrusive"][distance="DifferentModule"], edge[strengthLabel="Functional"][distance="DifferentCrate"]',
            style: {
                'line-color': '#ef4444',
                'target-arrow-color': '#ef4444',
                'width': edge => 2 + edge.data('strength') * 5,
                'opacity': 0.9
            }
        },
        // Good coupling edges
        {
            selector: 'edge[strengthLabel="Intrusive"][distance="SameModule"], edge[strengthLabel="Functional"][distance="SameModule"], edge[strengthLabel="Contract"][distance="DifferentModule"], edge[strengthLabel="Contract"][distance="DifferentCrate"]',
            style: {
                'line-color': '#22c55e',
                'target-arrow-color': '#22c55e',
                'opacity': 0.6
            }
        },
        // Edges with issues
        {
            selector: 'edge[issue]',
            style: {
                'width': edge => 3 + edge.data('strength') * 4,
                'opacity': 0.85
            }
        },
        // Hidden temporal coupling
        {
            selector: 'edge[?hiddenCoupling]',
            style: {
                'line-color': '#f59e0b',
                'target-arrow-color': '#f59e0b',
                'line-style': 'dashed',
                'width': edge => 2 + (edge.data('couplingRatio') || 0.5) * 4,
                'opacity': 0.95,
                'target-arrow-shape': 'triangle'
            }
        },
        // Cycle edges
        {
            selector: 'edge[?inCycle]',
            style: {
                'line-color': '#dc2626',
                'target-arrow-color': '#dc2626',
                'width': 3,
                'line-style': 'solid'
            }
        },
        {
            selector: 'edge[clusterColor]',
            style: {
                'line-color': 'data(clusterColor)',
                'target-arrow-color': 'data(clusterColor)',
                'opacity': 0.82
            }
        },
        // Parent edges (item -> module)
        {
            selector: 'edge[edgeType="parent"]',
            style: {
                'line-color': '#64748b',
                'target-arrow-color': '#64748b',
                'width': 1,
                'opacity': 0.3,
                'line-style': 'dotted',
                'target-arrow-shape': 'none'
            }
        },
        // Item dependency edges
        {
            selector: 'edge[edgeType="item-dep"]',
            style: {
                'width': 1.5,
                'opacity': 0.6,
                'line-color': edge => {
                    const strength = edge.data('strengthLabel');
                    if (strength === 'Intrusive') return '#ef4444';
                    if (strength === 'Functional') return '#f97316';
                    return '#6b7280';
                },
                'target-arrow-color': edge => {
                    const strength = edge.data('strengthLabel');
                    if (strength === 'Intrusive') return '#ef4444';
                    if (strength === 'Functional') return '#f97316';
                    return '#6b7280';
                },
                'target-arrow-shape': 'triangle',
                'arrow-scale': 0.8,
                'curve-style': 'bezier'
            }
        },
        // Highlighted state
        {
            selector: '.highlighted',
            style: {
                'opacity': 1,
                'border-width': 4,
                'border-color': '#60a5fa',
                'z-index': 20
            }
        },
        // Dimmed state
        {
            selector: '.dimmed',
            style: { 'opacity': 0.16 }
        },
        // Hidden state
        {
            selector: '.hidden',
            style: { 'display': 'none' }
        },
        // Dependency highlighting
        {
            selector: '.dependency-source',
            style: {
                'border-color': '#34d399',
                'border-width': 4,
                'z-index': 22
            }
        },
        {
            selector: '.dependency-target',
            style: {
                'border-color': '#f87171',
                'border-width': 4,
                'z-index': 22
            }
        },
        // Hover state
        {
            selector: '.hover',
            style: {
                'border-color': '#93c5fd',
                'border-width': 3,
                'z-index': 18
            }
        },
        // Search match
        {
            selector: '.search-match',
            style: {
                'border-color': '#facc15',
                'border-width': 4,
                'background-color': '#ca8a04'
            }
        }
    ];
}

function moduleNodeWidth(node) {
    const lines = (node.data('graph_label') || node.data('label') || '').split('\n');
    return Math.min(260, Math.max(120, ...lines.map(line => line.length * 7 + 28)));
}

function moduleNodeHeight(node) {
    const lines = (node.data('graph_label') || '').split('\n');
    return 30 + lines.reduce((height, line) => height + Math.max(1, Math.ceil(line.length / 32)) * 16, 0);
}

/**
 * Get layout configuration by name
 */
export function getLayoutConfig(name) {
    const configs = {
        cose: {
            name: 'cose',
            animate: true,
            animationDuration: 420,
            fit: true,
            padding: 70,
            nodeRepulsion: 11000,
            nodeDimensionsIncludeLabels: true,
            nodeOverlap: 30,
            componentSpacing: 100,
            idealEdgeLength: 140,
            edgeElasticity: 100,
            gravity: 0.25,
            numIter: 1000
        },
        dagre: {
            name: 'dagre',
            rankDir: 'LR',
            nodeDimensionsIncludeLabels: true,
            nodeSep: 50,
            rankSep: 140,
            edgeSep: 20,
            animate: true,
            animationDuration: 420,
            fit: true,
            padding: 70
        },
        concentric: {
            name: 'concentric',
            animate: true,
            animationDuration: 420,
            fit: true,
            padding: 70,
            concentric: node => node.data('couplings_in') || 0,
            levelWidth: () => 2
        },
        grid: { name: 'grid', animate: true, animationDuration: 420, fit: true, padding: 70, rows: 5 }
    };
    return configs[name] || configs.cose;
}

/**
 * Apply layout to graph
 */
export function applyLayout(name) {
    if (!state.cy) return;
    setCurrentLayout(name);
    state.cy.elements(':visible').layout(getLayoutConfig(name)).run();
}

/**
 * Center view on a specific node with optional re-layout
 */
export function centerOnNode(node, useRelayout = false) {
    if (!state.cy) return;

    if (useRelayout) {
        // Re-layout with selected node at center
        const layout = state.cy.layout({
            name: 'concentric',
            concentric: function(n) {
                if (n.id() === node.id()) return 10;
                if (node.neighborhood().contains(n)) return 5;
                return 1;
            },
            levelWidth: function() { return 2; },
            animate: true,
            animationDuration: 500
        });
        layout.run();
    } else {
        // Just animate to center the node
        state.cy.animate({
            center: { eles: node },
            zoom: Math.max(state.cy.zoom(), 1),
            duration: 400,
            easing: 'ease-out-cubic'
        });
    }
}

/**
 * Focus on node with zoom
 */
export function focusOnNode(node) {
    if (!state.cy) return;
    const neighborhood = node.closedNeighborhood().filter(':visible');
    state.cy.animate({
        fit: { eles: neighborhood, padding: 70 },
        duration: 400,
        easing: 'ease-out-cubic'
    });
}

/**
 * Highlight neighbors of a node
 */
export function highlightNeighbors(node) {
    clearHighlights();
    state.cy.elements().addClass('dimmed');
    node.removeClass('dimmed').addClass('highlighted');
    node.neighborhood().removeClass('dimmed').addClass('highlighted');
}

/**
 * Highlight dependency path for an edge
 */
export function highlightDependencyPath(edge) {
    clearHighlights();
    state.cy.elements().addClass('dimmed');

    const source = state.cy.getElementById(edge.data('source'));
    const target = state.cy.getElementById(edge.data('target'));

    source.removeClass('dimmed').addClass('dependency-source');
    target.removeClass('dimmed').addClass('dependency-target');
    edge.removeClass('dimmed').addClass('highlighted');

    state.cy.fit(source.union(target).union(edge), 100);
}

/**
 * Clear all highlights
 */
export function clearHighlights() {
    if (state.cy) {
        state.cy.elements().removeClass('highlighted dimmed dependency-source dependency-target search-match');
    }
}

/**
 * Setup graph event handlers
 */
function setupGraphEventHandlers(cy, onNodeTap, onEdgeTap, onBackgroundTap) {
    cy.on('tap', 'node', function(evt) {
        if (onNodeTap) onNodeTap(evt.target);
    });

    cy.on('tap', 'edge', function(evt) {
        if (onEdgeTap) onEdgeTap(evt.target);
    });

    cy.on('tap', function(evt) {
        if (evt.target === cy && onBackgroundTap) {
            onBackgroundTap();
        }
    });

    // Hover effects
    cy.on('mouseover', 'node', function(evt) {
        evt.target.addClass('hover');
    });

    cy.on('mouseout', 'node', function(evt) {
        evt.target.removeClass('hover');
    });
}

// =====================================================
// Helper Functions
// =====================================================

export function getHealthColor(health) {
    const colors = { good: COLORS.good, needs_review: COLORS.review, critical: COLORS.critical };
    return colors[health] || COLORS.unknown;
}

export function getNodeColor(data) {
    if (data.accidental_volatility) return COLORS.accidental;
    const subdomain = (data.subdomain || '').toLowerCase();
    if (subdomain === 'core') return COLORS.core;
    if (subdomain === 'supporting') return COLORS.supporting;
    if (subdomain === 'generic') return COLORS.generic;
    return getHealthColor(data.health);
}

export function getBalanceColor(balance) {
    if (balance >= 0.8) return COLORS.good;
    if (balance >= 0.4) return COLORS.review;
    return COLORS.critical;
}

function getDistanceStyle(distance) {
    if (distance === 'SameModule' || distance === 'SameFunction') return 'solid';
    if (distance === 'DifferentModule') return 'dashed';
    return 'dotted';
}

function sanitizeElementId(value) {
    return value.replace(/[^a-zA-Z0-9_-]/g, '-');
}

function compareIds(left, right) {
    return left < right ? -1 : left > right ? 1 : 0;
}

// Keep all dimensions and labels from one observed relationship. Combining a
// maximum strength with another observation's balance invents an assessment.
function compareObservations(left, right) {
    for (const [axis, direction] of [['balance', 1], ['strength', -1], ['distance', -1], ['volatility', -1]]) {
        const a = left.dimensions?.[axis]?.value, b = right.dimensions?.[axis]?.value;
        if (Number.isFinite(a) !== Number.isFinite(b)) return Number.isFinite(a) ? -1 : 1;
        if (Number.isFinite(a) && a !== b) return (a - b) * direction;
    }
    return compareIds(left.id, right.id);
}

export function edgeDataToDetails(edge) {
    if (Number.isFinite(edge.coupling_ratio)) return hiddenCouplingDetails(edge);
    const dims = edge.dimensions || {};
    return {
        id: edge.id, source: edge.source, target: edge.target, edgeType: 'module',
        strength: dims.strength?.value ?? null,
        strengthLabel: dims.strength?.label ?? 'Unknown',
        distance: dims.distance?.label ?? 'Unknown',
        volatility: dims.volatility?.label ?? 'Unknown',
        balance: dims.balance?.value ?? null,
        balanceLabel: dims.balance?.label ?? 'Unknown',
        classification: dims.balance?.classification ?? '',
        classificationJa: dims.balance?.classification_ja ?? '',
        interpretation: dims.balance?.interpretation ?? '',
        connascence: dims.connascence ?? null,
        issue: edge.issue,
        location: edge.location, inCycle: edge.in_cycle, count: 1
    };
}

// Old API snapshots may contain invented dimensions on co-change edges. Ignore
// those fields as well as null dimensions from current servers.
export function hiddenCouplingDetails(edge) {
    return {
        id: edge.id, source: edge.source, target: edge.target,
        edgeType: 'hidden-coupling', hiddenCoupling: true,
        strength: null, strengthLabel: 'Unknown', distance: 'Unknown', volatility: 'Unknown',
        balance: null, balanceLabel: 'Unknown', classification: '', classificationJa: '',
        issue: edge.issue, coChangeCount: edge.co_change_count, couplingRatio: edge.coupling_ratio, count: 1
    };
}

/** Render the server assessment; do not infer another model from syntax labels. */
export function analyzeCoupling(data) {
    if (data.hiddenCoupling || data.edgeType === 'hidden-coupling') {
        return { status: 'unknown', icon: '?', statusText: t('temporal_evidence_only'), action: null };
    }
    const status = ['Critical', 'High'].includes(data.issue?.severity) ? 'critical'
        : data.issue?.severity === 'Medium' ? 'acceptable'
        : { Balanced: 'good', Acceptable: 'acceptable', NeedsReview: 'acceptable', NeedsRefactoring: 'critical', Critical: 'critical' }[data.balanceLabel] || 'unknown';
    const classification = state.currentLang === 'ja' ? data.classificationJa || data.classification : data.classification;
    return { status, icon: { good: '✅', acceptable: '⚠️', critical: '❌', unknown: '?' }[status], statusText: classification || t('unknown'), action: null };
}

function getEdgeColorByAnalysis(data) {
    const analysis = analyzeCoupling(data);

    switch (analysis.status) {
        case 'good': return COLORS.good;
        case 'acceptable': return COLORS.review;
        case 'critical': return COLORS.critical;
        default: return COLORS.unknown;
    }
}
