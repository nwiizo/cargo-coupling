// =====================================================
// 3D Coupling Graph and Dimension Space
// =====================================================

import { state, setGraph3d, set3dMode } from './state.js';
import { t } from './i18n.js';
import { escapeHtml } from './utils.js';

const COLORS = {
    core: '#ef4444',
    supporting: '#38bdf8',
    generic: '#94a3b8',
    unknown: '#64748b',
    intrusive: '#ef4444',
    functional: '#f97316',
    model: '#eab308',
    contract: '#22c55e',
    hidden: '#f59e0b',
    highCohesion: '#22c55e',
    looseCoupling: '#10b981',
    acceptable: '#3b82f6',
    localComplexity: '#eab308',
    globalComplexity: '#ef4444',
    selected: '#f8fafc',
    axisX: '#ef4444',
    axisY: '#38bdf8',
    axisZ: '#22c55e'
};

let onNodeSelected = null;
let onEdgeSelected = null;
let selected3dNodeId = null;
let selected3dLinkId = null;

export function initCouplingGraph3d(data, nodeHandler, edgeHandler) {
    const container = document.getElementById('graph-3d');
    if (!container) return null;

    if (typeof ForceGraph3D !== 'function') {
        container.innerHTML = '<div class="graph-error">3D graph library failed to load.</div>';
        return null;
    }

    onNodeSelected = nodeHandler;
    onEdgeSelected = edgeHandler;

    const graph = ForceGraph3D()(container)
        .backgroundColor('#0f172a')
        .showNavInfo(false)
        .nodeLabel(node => node.tooltip || node.label || node.id)
        .nodeColor(nodeColor)
        .nodeVal(nodeValue)
        .linkLabel(link => link.tooltip || `${link.source?.id || link.source} -> ${link.target?.id || link.target}`)
        .linkColor(linkColor)
        .linkWidth(linkWidth)
        .linkOpacity(0.72)
        .linkDirectionalArrowLength(link => link.isAxis ? 0 : 3)
        .linkDirectionalArrowRelPos(0.92)
        .linkDirectionalParticles(link => {
            if (link.id === selected3dLinkId) return 6;
            if (link.hidden || isGlobalComplexity(link)) return 3;
            return 0;
        })
        .linkDirectionalParticleColor(link => link.hidden ? COLORS.hidden : COLORS.globalComplexity)
        .linkDirectionalParticleWidth(link => link.hidden ? 2 : 1.5)
        .onNodeClick(handleNodeClick)
        .onLinkClick(handleLinkClick);

    setGraph3d(graph);
    render3dMode(data, state.current3dMode || 'network');
    window.addEventListener('resize', resize3dGraph);
    return graph;
}

export function render3dMode(data = state.graphData, mode = state.current3dMode || 'network') {
    if (!state.graph3d || !data) return;

    set3dMode(mode);
    selected3dNodeId = null;
    selected3dLinkId = null;

    const dimensionLabels = document.getElementById('dimension-space-labels');
    if (dimensionLabels) {
        dimensionLabels.style.display = mode === 'dimension-space' ? 'block' : 'none';
    }

    if (mode === 'dimension-space') {
        const graphData = buildDimensionSpaceData(data);
        state.graph3d
            .graphData(graphData)
            .cooldownTicks(0);
        state.graph3d.cameraPosition({ x: 260, y: 220, z: 320 }, { x: 0, y: 0, z: 0 }, 900);
        return;
    }

    state.graph3d
        .cooldownTicks(120)
        .graphData(buildNetworkData(data));
    const linkForce = state.graph3d.d3Force('link');
    if (linkForce) {
        linkForce.distance(link => linkDistance(link));
    }
    state.graph3d.cameraPosition({ x: 0, y: 0, z: 420 }, { x: 0, y: 0, z: 0 }, 900);
}

export function refresh3dGraph() {
    render3dMode(state.graphData, state.current3dMode);
}

export function focusNode3d(nodeId) {
    if (!state.graph3d || !nodeId) return;
    selected3dNodeId = nodeId;
    selected3dLinkId = null;
    state.graph3d.nodeColor(nodeColor).linkWidth(linkWidth).linkDirectionalParticles(link => {
        if (touchesNode(link, nodeId)) return 3;
        return link.hidden ? 2 : 0;
    });

    const graphData = state.graph3d.graphData();
    const node = graphData.nodes.find(n => n.id === nodeId || n.sourceNodeId === nodeId || n.targetNodeId === nodeId);
    if (node) {
        const distance = 120;
        const ratio = 1 + distance / Math.hypot(node.x || 1, node.y || 1, node.z || 1);
        state.graph3d.cameraPosition(
            { x: (node.x || 0) * ratio, y: (node.y || 0) * ratio, z: (node.z || 0) * ratio },
            node,
            700
        );
    }
}

export function focusLink3d(linkId, nodeIds = []) {
    if (!state.graph3d) return;
    selected3dNodeId = null;
    selected3dLinkId = linkId;
    state.graph3d.linkWidth(linkWidth).linkDirectionalParticles(link => link.id === linkId ? 8 : (link.hidden ? 2 : 0));
    if (nodeIds.length > 0) {
        focusNode3d(nodeIds[0]);
        selected3dLinkId = linkId;
    }
}

export function clear3dFocus() {
    selected3dNodeId = null;
    selected3dLinkId = null;
    if (state.graph3d) {
        state.graph3d.nodeColor(nodeColor).linkWidth(linkWidth).linkDirectionalParticles(link => link.hidden ? 2 : 0);
    }
}

function buildNetworkData(data) {
    const hideExternal = document.getElementById('hide-external')?.checked ?? true;
    const showHidden = document.getElementById('show-hidden-coupling')?.checked ?? true;
    const nodeIds = new Set();
    const nodes = data.nodes
        .filter(node => !hideExternal || !node.file_path?.startsWith('[external]'))
        .map(node => {
            nodeIds.add(node.id);
            return {
                id: node.id,
                label: node.label,
                tooltip: nodeTooltip(node),
                subdomain: node.subdomain,
                flags: node.flags || [],
                volatility: node.metrics?.volatility || 0,
                balance: node.metrics?.balance_score ?? 1,
                couplings: (node.metrics?.couplings_in || 0) + (node.metrics?.couplings_out || 0),
                sourceNode: node
            };
        });

    const links = [];
    for (const edge of data.edges || []) {
        if (!nodeIds.has(edge.source) || !nodeIds.has(edge.target)) continue;
        links.push(edgeToLink(edge, false));
    }
    if (showHidden) {
        for (const hidden of data.hidden_couplings || []) {
            if (!nodeIds.has(hidden.source) || !nodeIds.has(hidden.target)) continue;
            links.push(edgeToLink(hidden, true));
        }
    }

    return { nodes, links };
}

function buildDimensionSpaceData(data) {
    const showHidden = document.getElementById('show-hidden-coupling')?.checked ?? true;
    const nodes = [
        axisNode('axis-origin', 'Origin', 0, 0, 0),
        axisNode('axis-x', 'Strength', 180, 0, 0, COLORS.axisX),
        axisNode('axis-y', 'Distance', 0, 180, 0, COLORS.axisY),
        axisNode('axis-z', 'Volatility', 0, 0, 180, COLORS.axisZ)
    ];
    const links = [
        axisLink('axis-link-x', 'axis-origin', 'axis-x', COLORS.axisX),
        axisLink('axis-link-y', 'axis-origin', 'axis-y', COLORS.axisY),
        axisLink('axis-link-z', 'axis-origin', 'axis-z', COLORS.axisZ)
    ];

    const couplingEdges = [...(data.edges || [])];
    if (showHidden) couplingEdges.push(...(data.hidden_couplings || []));

    couplingEdges.forEach((edge, index) => {
        const dims = edge.dimensions || {};
        const strength = dims.strength?.value ?? 0.5;
        const distance = dims.distance?.value ?? 0.5;
        const volatility = dims.volatility?.value ?? 0.5;
        const id = `space-${edge.id || index}`;
        nodes.push({
            id,
            label: `${edge.source} -> ${edge.target}`,
            tooltip: dimensionTooltip(edge),
            fx: strength * 180,
            fy: distance * 180,
            fz: volatility * 180,
            sourceNodeId: edge.source,
            targetNodeId: edge.target,
            edgeData: edge,
            hidden: edge.issue?.type === 'HiddenCoupling' || edge.issue?.issue_type === 'HiddenCoupling',
            classification: dims.balance?.classification || '',
            strengthLabel: dims.strength?.label || 'Model',
            balance: dims.balance?.value ?? 0.5,
            isCouplingPoint: true
        });
    });

    return { nodes, links };
}

function edgeToLink(edge, hidden) {
    const dims = edge.dimensions || {};
    return {
        id: edge.id,
        source: edge.source,
        target: edge.target,
        hidden,
        edgeData: edge,
        strength: dims.strength?.value ?? 0.5,
        strengthLabel: dims.strength?.label || 'Model',
        distance: dims.distance?.value ?? 0.5,
        distanceLabel: dims.distance?.label || 'DifferentModule',
        volatility: dims.volatility?.label || 'Low',
        balance: dims.balance?.value ?? 0.5,
        classification: dims.balance?.classification || '',
        tooltip: edgeTooltip(edge, hidden)
    };
}

function handleNodeClick(node) {
    if (node.isAxis) return;
    if (node.isCouplingPoint) {
        selected3dLinkId = node.edgeData?.id;
        state.graph3d?.nodeColor(nodeColor);
        onEdgeSelected?.(node.edgeData);
        return;
    }
    selected3dNodeId = node.id;
    state.graph3d?.nodeColor(nodeColor);
    onNodeSelected?.(node.id);
}

function handleLinkClick(link) {
    if (link.isAxis) return;
    selected3dLinkId = link.id;
    state.graph3d?.linkWidth(linkWidth);
    onEdgeSelected?.(link.edgeData);
}

function nodeColor(node) {
    if (node.id === selected3dNodeId || node.id === selected3dLinkId) return COLORS.selected;
    if (node.isAxis) return node.color;
    if (node.isCouplingPoint) {
        if (node.hidden) return COLORS.hidden;
        return classificationColor(node.classification);
    }
    if ((node.flags || []).includes('AccidentalVolatility')) return '#f97316';
    const subdomain = (node.subdomain || '').toLowerCase();
    return COLORS[subdomain] || heatColor(node.volatility);
}

function nodeValue(node) {
    if (node.isAxis) return 3;
    if (node.isCouplingPoint) return node.hidden ? 5 : 3 + (1 - (node.balance ?? 0.5)) * 5;
    return 5 + Math.min(22, node.couplings || 0) + (node.volatility || 0) * 8;
}

function linkColor(link) {
    if (link.isAxis) return link.color;
    if (link.id === selected3dLinkId) return COLORS.selected;
    if (link.hidden) return COLORS.hidden;
    if (isGlobalComplexity(link)) return COLORS.globalComplexity;
    return strengthColor(link.strengthLabel);
}

function linkWidth(link) {
    if (link.isAxis) return 1.5;
    if (link.id === selected3dLinkId) return 7;
    if (link.hidden) return 2.5 + (link.edgeData?.coupling_ratio || 0.5) * 3;
    return 1 + (link.strength || 0.5) * 5;
}

function linkDistance(link) {
    if (link.isAxis) return 180;
    return 45 + (link.distance || 0.5) * 190;
}

function touchesNode(link, nodeId) {
    const source = typeof link.source === 'object' ? link.source.id : link.source;
    const target = typeof link.target === 'object' ? link.target.id : link.target;
    return source === nodeId || target === nodeId;
}

function isGlobalComplexity(link) {
    return link.classification === 'Global Complexity'
        || link.classification === 'Needs Refactoring'
        || (link.strength >= 0.75 && link.distance >= 0.5 && link.volatility === 'High');
}

function classificationColor(classification) {
    if (classification === 'High Cohesion') return COLORS.highCohesion;
    if (classification === 'Loose Coupling') return COLORS.looseCoupling;
    if (classification === 'Acceptable') return COLORS.acceptable;
    if (classification === 'Local Complexity') return COLORS.localComplexity;
    if (classification === 'Global Complexity' || classification === 'Needs Refactoring') return COLORS.globalComplexity;
    return COLORS.model;
}

function strengthColor(strength) {
    if (strength === 'Intrusive') return COLORS.intrusive;
    if (strength === 'Functional') return COLORS.functional;
    if (strength === 'Contract') return COLORS.contract;
    return COLORS.model;
}

function heatColor(volatility) {
    if (volatility >= 0.9) return '#ef4444';
    if (volatility >= 0.45) return '#eab308';
    return '#64748b';
}

function axisNode(id, label, x, y, z, color = '#64748b') {
    return { id, label, tooltip: label, fx: x, fy: y, fz: z, color, isAxis: true };
}

function axisLink(id, source, target, color) {
    return { id, source, target, color, isAxis: true };
}

function nodeTooltip(node) {
    return `
        <div class="graph-tooltip">
            <strong>${escapeHtml(node.label)}</strong><br>
            ${t('subdomain')}: ${escapeHtml(labelSubdomain(node.subdomain))}<br>
            ${t('volatility')}: ${Number(node.metrics?.volatility || 0).toFixed(2)}<br>
            ${t('balance')}: ${Math.round((node.metrics?.balance_score || 0) * 100)}%
        </div>
    `;
}

function edgeTooltip(edge, hidden) {
    const dims = edge.dimensions || {};
    return `
        <div class="graph-tooltip">
            <strong>${hidden ? t('hidden_coupling') : t('code_coupling')}</strong><br>
            ${escapeHtml(edge.source)} -> ${escapeHtml(edge.target)}<br>
            ${t('strength')}: ${escapeHtml(labelValue(dims.strength?.label || 'Model'))}<br>
            ${t('distance')}: ${escapeHtml(labelValue(dims.distance?.label || 'DifferentModule'))}<br>
            ${t('volatility')}: ${escapeHtml(labelValue(dims.volatility?.label || 'Low'))}<br>
            ${t('classification')}: ${escapeHtml(state.currentLang === 'ja' ? (dims.balance?.classification_ja || dims.balance?.classification || '') : (dims.balance?.classification || ''))}
        </div>
    `;
}

function dimensionTooltip(edge) {
    const dims = edge.dimensions || {};
    return `
        <div class="graph-tooltip">
            <strong>${escapeHtml(edge.source)} -> ${escapeHtml(edge.target)}</strong><br>
            ${t('legend_x_axis')}: ${dims.strength?.value ?? 0}<br>
            ${t('legend_y_axis')}: ${dims.distance?.value ?? 0}<br>
            ${t('legend_z_axis')}: ${dims.volatility?.value ?? 0}<br>
            ${escapeHtml(state.currentLang === 'ja' ? (dims.balance?.classification_ja || dims.balance?.classification || '') : (dims.balance?.classification || ''))}
        </div>
    `;
}

function labelSubdomain(subdomain) {
    return {
        Core: t('subdomain_core'),
        Supporting: t('subdomain_supporting'),
        Generic: t('subdomain_generic')
    }[subdomain] || t('subdomain_not_configured');
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
        DifferentCrate: t('distance_different_crate')
    }[value] || value;
}

function resize3dGraph() {
    const container = document.getElementById('graph-3d');
    if (!state.graph3d || !container) return;
    state.graph3d.width(container.clientWidth).height(container.clientHeight);
}
